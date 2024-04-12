//! A parser for Manycore System XML configuration files

mod borders;
mod channels;
mod configurable_attributes;
mod cores;
mod error;
mod graph;
mod info;
mod router;
mod routing;
mod tests;
mod utils;

use std::collections::BTreeMap;
use std::collections::HashMap;

pub use crate::borders::*;
pub use crate::channels::*;
pub use crate::cores::*;
pub use crate::error::*;
pub use crate::graph::*;
pub use crate::router::*;
pub use crate::routing::*;
pub use crate::utils::*;
pub use configurable_attributes::*;
use getset::{Getters, MutGetters, Setters};
use serde::{Deserialize, Serialize};

pub static ID_KEY: &'static str = "@id";
pub static COORDINATES_KEY: &'static str = "@coordinates";
pub static BORDER_ROUTERS_KEY: &'static str = "@borderRouters";
pub static ROUTING_KEY: &'static str = "@routingAlgorithm";

#[derive(Serialize, Deserialize, Debug, PartialEq, Getters, Setters, MutGetters)]
#[serde(rename_all = "PascalCase")]
/// This struct represents the many core system that was provided as input via XML.
pub struct ManycoreSystem {
    #[serde(rename = "@xmlns")]
    xmlns: String,
    #[serde(rename = "@xmlns:xsi")]
    xmlns_si: String,
    // Not sure why deserialisation fails for xsi:schemaLocation but serialisation succeeds.
    // Either way, this works and I guess it's just a quick-xml quirk.
    #[serde(rename(serialize = "@xsi:schemaLocation", deserialize = "@schemaLocation"))]
    xsi_schema_location: String,
    #[getset(get = "pub")]
    #[serde(rename = "@rows")]
    /// Rows in the cores matrix.
    rows: u8,
    #[serde(rename = "@columns")]
    #[getset(get = "pub")]
    /// Columns in the cores matrix.
    columns: u8,
    #[serde(rename = "@routingAlgo", skip_serializing_if = "Option::is_none")]
    #[getset(get = "pub")]
    /// Algorithm used in the observed routing (Channels data), if any.
    routing_algo: Option<String>,
    #[getset(get = "pub", set = "pub", get_mut = "pub")]
    /// The provided task graph.
    task_graph: TaskGraph,
    #[getset(get = "pub", set = "pub", get_mut = "pub")]
    /// The system's cores.
    cores: Cores,
    /// Borders (edge routers).
    #[serde(skip_serializing_if = "Borders::should_skip_serialize")]
    #[getset(get = "pub", get_mut = "pub")]
    borders: Borders,
    #[serde(skip)]
    #[getset(get = "pub", set = "pub", get_mut = "pub")]
    /// This is not part of the XML and is used in the routing logic. It maps a task ID (key) to the corresponding core ID (value, the core upon which the task is allocated to).
    task_core_map: HashMap<u16, usize>,
    #[serde(skip)]
    #[getset(get = "pub")]
    /// This is not part of the XML and is used to provided the frontend with a list of attributes that can be requested for rendering.
    configurable_attributes: ConfigurableAttributes,
}

fn generation_error(reason: String) -> ManycoreError {
    ManycoreError::new(error::ManycoreErrorKind::GenerationError(reason))
}

impl ManycoreSystem {
    /// Deserialises an XML file into a ManycoreSystem struct.
    pub fn parse_file(path: &str) -> Result<ManycoreSystem, ManycoreError> {
        let file_content =
            std::fs::read_to_string(path).map_err(|e| generation_error(e.to_string()))?;

        let mut manycore: ManycoreSystem =
            quick_xml::de::from_str(&file_content).map_err(|e| generation_error(e.to_string()))?;

        // Sort cores by id. This is potentially unnecessary if the file contains,
        // cores in an ordered manner but that is not a guarantee.
        manycore
            .cores_mut()
            .list_mut()
            .sort_by(|me, other| me.id().cmp(&other.id()));

        // Configurable attributes storage maps
        let mut core_attributes: BTreeMap<String, ProcessedAttribute> = BTreeMap::new();
        let mut router_attributes: BTreeMap<String, ProcessedAttribute> = BTreeMap::new();
        let mut channel_attributes: BTreeMap<String, ProcessedAttribute> = BTreeMap::new();

        // Manually insert core attributes that are not part of the "other_attributes" map.
        core_attributes.insert_manual(ID_KEY, AttributeType::Text);
        core_attributes.insert_manual(COORDINATES_KEY, AttributeType::Coordinates);
        // Manually insert channel attributes that are not part of the "other_attributes" map.
        channel_attributes.insert_manual(ROUTING_KEY, AttributeType::Routing);
        channel_attributes.insert_manual(BORDER_ROUTERS_KEY, AttributeType::Boolean);

        let last = manycore.cores.list().len() - 1;
        let mut task_core_map = HashMap::new();
        for i in 0..=last {
            let core = manycore
                .cores_mut()
                .list_mut()
                .get_mut(i)
                .ok_or(generation_error(
                    "Something went wrong inspecting Core data.".into(),
                ))?;

            // task -> core map
            if let Some(task_id) = core.allocated_task().as_ref() {
                task_core_map.insert(*task_id, i);
            }

            // router ID
            core.router_mut().set_id(i as u8);

            // Populate attribute maps
            core_attributes.extend_from_element(core);
            router_attributes.extend_from_element(core.router());
            for channel in core.channels().channel().values() {
                channel_attributes.extend_from_element(channel);
            }
        }

        // Store task->core map
        manycore.task_core_map = task_core_map;

        // Populate core -> border map
        manycore.borders_mut().compute_core_border_map();

        manycore.configurable_attributes = ConfigurableAttributes::new(
            core_attributes,
            router_attributes,
            manycore.routing_algo.clone(),
            Vec::from(&SUPPORTED_ALGORITHMS),
            channel_attributes,
        );

        Ok(manycore)
    }
}
