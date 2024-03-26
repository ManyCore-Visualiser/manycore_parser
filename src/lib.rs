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
    #[getset(get = "pub")]
    borders: Borders,
    #[serde(skip)]
    #[getset(get = "pub", set = "pub", get_mut = "pub")]
    /// This is not part of the XML and is used in the routing logic. It is a map with the core IDs as key and the core (router) possible connections as value.
    connections: HashMap<usize, Neighbours>,
    #[serde(skip)]
    #[getset(get = "pub", set = "pub", get_mut = "pub")]
    /// This is not part of the XML and is used in the routing logic. It maps a task ID (key) to the corresponding core ID (value, the core upon which the task is allocated to).
    task_core_map: HashMap<u16, usize>,
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

        let core: &Core = self
            .cores()
            .list()
            .get(group_id.parse::<usize>().map_err(|_| InfoError {
                reason: "Invalid group_id",
            })?)
            .ok_or(InfoError {
                reason: "Invalid index",
            })?;

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
    pub fn parse_file(path: &str) -> Result<ManycoreSystem, Box<dyn std::error::Error>> {
        let file_content = std::fs::read_to_string(path)?;

        let mut manycore: ManycoreSystem = quick_xml::de::from_str(&file_content)?;

        // Sort cores by id. This is potentially unnecessary if the file contains,
        // cores in an ordered manner but that is not a guarantee.
        manycore
            .cores_mut()
            .list_mut()
            .sort_by(|me, other| me.id().cmp(&other.id()));

        // Populate neighbour connections, task -> core map and router IDs
        let usize_columns = usize::from(manycore.columns);
        let last = manycore.cores.list().len() - 1;
        let mut task_core_map = HashMap::new();
        for i in 0..=last {
            // Neighbour resolving logic START
            // Here we workout all the possible neighbours a node might
            // have based on its index. Note that this works only for a
            // 2D matrix.
            let mut neighbours = Neighbours::default();

            let right = i + 1;
            // If i is greater or equal to the number of columns, we are on the
            // second row (i is 0 indexed).
            let top = i >= usize_columns;
            let bottom = i + usize_columns;

            // Right
            if right % usize_columns != 0 {
                neighbours.set_right(Neighbour::new(Some(right)));
            }

            // Left
            if i % usize_columns != 0 {
                neighbours.set_left(Neighbour::new(Some(i - 1)));
            }

            // Top
            if top {
                neighbours.set_top(Neighbour::new(Some(i - usize_columns)));
            }

            // Bottom
            // bottom is defined as i + usize_columns. Effectively, we are checking
            // if a row exists past the current i.
            if bottom <= last {
                neighbours.set_bottom(Neighbour::new(Some(bottom)));
            }

            manycore.connections_mut().insert(i, neighbours);

            // Neighbour resolving logic END

            // task -> core map
            if let Some(task_id) = manycore.cores().list()[i].allocated_task().as_ref() {
                task_core_map.insert(*task_id, i);
            }

            // router ID
            manycore.cores_mut().list_mut()[i]
                .router_mut()
                .set_id(i as u8);
        }

        // Store map
        manycore.task_core_map = task_core_map;

        // Workout configurable attributes
        let mut core_attributes: BTreeMap<String, AttributeType> = BTreeMap::new();
        // Manually insert core attributes that are not part of the "other_attributes" map.
        core_attributes.insert("@id".to_string(), AttributeType::Text);
        core_attributes.insert("@coordinates".to_string(), AttributeType::Text);
        let mut router_attributes: BTreeMap<String, AttributeType> = BTreeMap::new();
        for core in manycore.cores.list().iter() {
            Self::populate_attribute_map(core, &mut core_attributes);
            Self::populate_attribute_map(core.router(), &mut router_attributes);
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
