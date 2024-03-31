//! A parser for Manycore System XML configuration files

mod borders;
mod channels;
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
use getset::{Getters, MutGetters, Setters};
use serde::{Deserialize, Serialize};

pub trait WithXMLAttributes {
    fn id(&self) -> &u8;
    fn other_attributes(&self) -> &Option<BTreeMap<String, String>>;
    fn variant(&self) -> &'static str;
}

// This will be serialised as JSON
#[derive(Serialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub enum AttributeType {
    Text,
    Number,
}

/// A struct containing information about what customisation
/// parameters to provide the user with.
/// This will be serialised as JSON
#[derive(Serialize, Debug, PartialEq, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConfigurableAttributes {
    /// Core parameters. The key is the parameter name, value is parameter type.
    core: BTreeMap<String, AttributeType>,
    /// Router parameters. The key is the parameter name, value is parameter type.
    router: BTreeMap<String, AttributeType>,
    /// A list of supported routing algorithms.
    algorithms: Vec<RoutingAlgorithms>,
    /// Denotes the presence of an observed routing outcome.
    observed_algorithm: Option<String>,
    /// Denotes the presence of edge routers.
    sinks_sources: bool,
}

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
    /// Retrieves all available attributes and their type, and inserts them in the given map.
    fn populate_attribute_map<T: WithXMLAttributes>(
        item: &T,
        map: &mut BTreeMap<String, AttributeType>,
    ) {
        // Are there any attributes we can inspect?
        if let Some(other_attributes) = item.other_attributes() {
            for (key, value) in other_attributes {
                // It's worth inspecting the attribute only if missing in the map.
                if !map.contains_key(key) {
                    // If parsing the attribute value as a number fails, the attribute must
                    // be a string.
                    let attribute_type = match value.parse::<u64>() {
                        Ok(_) => AttributeType::Number,
                        Err(_) => AttributeType::Text,
                    };

                    map.insert(key.clone(), attribute_type);
                }
            }
        }
    }

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
        let mut core_attributes: BTreeMap<String, AttributeType> = BTreeMap::new();
        let mut router_attributes: BTreeMap<String, AttributeType> = BTreeMap::new();

        // Manually insert core attributes that are not part of the "other_attributes" map.
        core_attributes.insert("@id".to_string(), AttributeType::Text);
        core_attributes.insert("@coordinates".to_string(), AttributeType::Text);

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
            Self::populate_attribute_map(core, &mut core_attributes);
            Self::populate_attribute_map(core.router(), &mut router_attributes);
        }

        // Store task->core map
        manycore.task_core_map = task_core_map;

        // Populate core -> source map
        let Borders {
            ref sources,
            ref mut core_source_map,
            ..
        } = manycore.borders_mut();
        for source in sources.values() {
            core_source_map.insert(*source.core_id(), *source.task_id());
        }

        manycore.configurable_attributes = ConfigurableAttributes {
            core: core_attributes,
            router: router_attributes,
            algorithms: Vec::from(&SUPPORTED_ALGORITHMS),
            observed_algorithm: manycore.routing_algo.clone(),
            sinks_sources: !manycore.borders.should_skip_serialize(),
        };

        Ok(manycore)
    }
}
