//! A parser for Manycore System XML configuration files

mod borders;
mod channels;
mod cores;
mod graph;
mod router;
mod routing;
mod tests;
mod utils;

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;

pub use crate::borders::*;
pub use crate::channels::*;
pub use crate::cores::*;
pub use crate::graph::*;
pub use crate::router::*;
pub use crate::routing::*;
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
    borders: Borders,
    #[serde(skip)]
    #[getset(get = "pub")]
    /// This is not part of the XML and is used to provided the frontend with a list of attributes that can be requested for rendering.
    configurable_attributes: ConfigurableAttributes,
}

/// A struct to wrap element (core, router) information retrieval errors.
#[derive(Debug, Clone)]
pub struct InfoError {
    /// The error's root cause.
    reason: &'static str,
}

impl Display for InfoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.reason)
    }
}

impl Error for InfoError {}

impl ManycoreSystem {
    /// Gets all available info for specific core or router.
    /// group_id looks something like "r1" or "c20", where r (router) and c (core) symbolise the variant,
    /// and the number is the element's index.
    pub fn get_core_router_specific_info(
        &self,
        mut group_id: String,
    ) -> Result<Option<BTreeMap<String, String>>, InfoError> {
        if group_id.len() == 0 {
            return Err(InfoError {
                reason: "Empty group_id",
            });
        };

        let variant_string = group_id.remove(0).to_string();

        // let core: &Core = self
        //     .cores()
        //     .list()
        //     .get(group_id.parse::<usize>().map_err(|_| InfoError {
        //         reason: "Invalid group_id",
        //     })?)
        //     .ok_or(InfoError {
        //         reason: "Invalid index",
        //     })?;

        let core = self.cores().core_map().core_by_id(group_id);

        // id and allocated_task are not part of the core "other_attributes" field so we shall
        // add them manually.
        let insert_core_default = |mut tree: BTreeMap<String, String>| {
            tree.insert("@id".into(), core.id().to_string());

            if let Some(task_id) = core.allocated_task() {
                tree.insert("@allocated_task".into(), task_id.to_string());
            }

            tree
        };

        match variant_string.as_str() {
            "r" => {
                // All relevant router info is already stored in the "other_attributes" map.
                let attributes_clone = core.router().other_attributes().clone();

                Ok(attributes_clone)
            }
            "c" => {
                let attributes_clone = core.other_attributes().clone();

                // We clone the core's map and insert missing fields.
                match attributes_clone {
                    Some(attributes) => Ok(Some(insert_core_default(attributes))),
                    None => Ok(Some(insert_core_default(BTreeMap::new()))),
                }
            }
            _ => Err(InfoError {
                reason: "Invalid variant",
            }),
        }
    }

    /// Deserialises an XML file into a ManycoreSystem struct.
    pub fn parse_file(path: &str) -> Result<ManycoreSystem, Box<dyn std::error::Error>> {
        let file_content = std::fs::read_to_string(path)?;

        let mut manycore: ManycoreSystem = quick_xml::de::from_str(&file_content)?;

        // Populate neighbour connections, task -> core map and router IDs
        let usize_columns = usize::from(manycore.columns);

        manycore.configurable_attributes = ConfigurableAttributes {
            core: manycore.cores.core_map().core_attributes().clone(),
            router: manycore.cores.core_map().router_attributes().clone(),
            algorithms: Vec::from(&SUPPORTED_ALGORITHMS),
            observed_algorithm: manycore.routing_algo.clone(),
            sinks_sources: !manycore.borders.should_skip_serialize(),
        };

        Ok(manycore)
    }
}
