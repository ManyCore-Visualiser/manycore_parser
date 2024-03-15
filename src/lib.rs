//! A parser for Manycore System XML configuration files

mod channels;
mod cores;
mod graph;
mod router;
mod routing;
mod sink_source;
mod utils;

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt::Display;

pub use crate::channels::*;
pub use crate::cores::*;
pub use crate::graph::*;
pub use crate::router::*;
pub use crate::routing::*;
pub use crate::sink_source::*;
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

// This will be serialised as JSON
#[derive(Serialize, Debug, PartialEq, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConfigurableAttributes {
    core: HashMap<String, AttributeType>,
    router: HashMap<String, AttributeType>,
    algorithms: Vec<RoutingAlgorithms>,
    observed_algorithm: Option<String>,
    sinks_sources: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Getters, Setters, MutGetters)]
#[serde(rename_all = "PascalCase")]
/// This struct represents the many core system that was provided as input via XML
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
    /// Rows in the cores matrix
    rows: u8,
    #[serde(rename = "@columns")]
    #[getset(get = "pub")]
    /// Columns in the cores matrix
    columns: u8,
    #[serde(rename = "@routing_algo", skip_serializing_if = "Option::is_none")]
    #[getset(get = "pub")]
    /// Algorithm used in the observed routing (Channels data)
    routing_algo: Option<String>,
    /// Sources
    #[serde(
        rename = "Source",
        skip_serializing_if = "BTreeMap::is_empty",
        serialize_with = "Source::serialize_btreemap_vector",
        deserialize_with = "Source::deserialize_btreemap_vector"
    )]
    #[getset(get = "pub")]
    sources: BTreeMap<usize, Source>,
    /// Sinks
    #[serde(
        rename = "Sink",
        skip_serializing_if = "BTreeMap::is_empty",
        serialize_with = "Sink::serialize_btreemap_vector",
        deserialize_with = "Sink::deserialize_btreemap_vector"
    )]
    #[getset(get = "pub")]
    sinks: BTreeMap<usize, Sink>,
    #[getset(get = "pub", set = "pub", get_mut = "pub")]
    /// The provided task graph
    task_graph: TaskGraph,
    #[getset(get = "pub", set = "pub", get_mut = "pub")]
    /// The system's cores
    cores: Cores,
    #[serde(skip)]
    #[getset(get = "pub", set = "pub", get_mut = "pub")]
    /// This is not part of the XML and is used in the routing logic. It is a map with the core IDs as key and the core (router) connections as value.
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

#[derive(Debug, Clone)]
pub struct InfoError {
    reason: String,
}

impl Display for InfoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.reason)
    }
}

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
                reason: "Empty group_id".into(),
            });
        };

        let variant_string = group_id.remove(0).to_string();

        let core: &Core = self
            .cores()
            .list()
            .get(group_id.parse::<usize>().map_err(|_| InfoError {
                reason: "Invalid group_id".into(),
            })?)
            .ok_or(InfoError {
                reason: "Invalid index".into(),
            })?;

        let insert_core_default = |mut tree: BTreeMap<String, String>| {
            tree.insert("@id".into(), core.id().to_string());

            if let Some(task_id) = core.allocated_task() {
                tree.insert("@allocated_task".into(), task_id.to_string());
            }

            tree
        };

        match variant_string.as_str() {
            "r" => {
                let attributes_clone = core.router().other_attributes().clone();

                Ok(attributes_clone)
            }
            "c" => {
                let attributes_clone = core.other_attributes().clone();

                match attributes_clone {
                    Some(attributes) => Ok(Some(insert_core_default(attributes))),
                    None => Ok(Some(insert_core_default(BTreeMap::new()))),
                }
            }
            _ => Err(InfoError {
                reason: "Invalid variant".into(),
            }),
        }
    }

    fn populate_attribute_map<T: WithXMLAttributes>(
        item: &T,
        map: &mut HashMap<String, AttributeType>,
    ) {
        if let Some(other_attributes) = item.other_attributes() {
            for (key, value) in other_attributes {
                if !map.contains_key(key) {
                    let attribute_type = match value.parse::<u64>() {
                        Ok(_) => AttributeType::Number,
                        Err(_) => AttributeType::Text,
                    };

                    map.insert(key.clone(), attribute_type);
                }
            }
        }
    }
    pub fn parse_file(path: &str) -> Result<ManycoreSystem, Box<dyn std::error::Error>> {
        let file_content = std::fs::read_to_string(path)?;

        let mut manycore: ManycoreSystem = quick_xml::de::from_str(&file_content)?;

        // Sort cores by id
        manycore
            .cores_mut()
            .list_mut()
            .sort_by(|me, other| me.id().cmp(&other.id()));

        // Populate neighbour connections, task -> core map and router IDs
        let usize_columns = usize::from(manycore.columns);
        let last = manycore.cores.list().len() - 1;
        let mut task_core_map = HashMap::new();
        for i in 0..=last {
            // Neighbours
            let right = i + 1;
            let top = i >= usize_columns;
            let bottom = i + usize_columns;
            let mut neighbours = Neighbours::default();

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
            if bottom <= last {
                neighbours.set_bottom(Neighbour::new(Some(bottom)));
            }

            manycore.connections_mut().insert(i, neighbours);

            // task -> core map
            if let Some(task_id) = manycore.cores().list()[i].allocated_task().as_ref() {
                task_core_map.insert(*task_id, i);
            }

            // router ID
            manycore
                .cores_mut()
                .list_mut()
                .get_mut(i)
                .unwrap()
                .router_mut()
                .set_id(i as u8);
        }

        // Store map
        manycore.task_core_map = task_core_map;

        // Workout attributes
        let mut core_attributes: HashMap<String, AttributeType> = HashMap::new();
        core_attributes.insert("@id".to_string(), AttributeType::Text);
        core_attributes.insert("@coordinates".to_string(), AttributeType::Text);
        let mut router_attributes: HashMap<String, AttributeType> = HashMap::new();
        for core in manycore.cores.list().iter() {
            Self::populate_attribute_map(core, &mut core_attributes);
            Self::populate_attribute_map(core.router(), &mut router_attributes);
        }

        manycore.configurable_attributes = ConfigurableAttributes {
            core: core_attributes,
            router: router_attributes,
            algorithms: Vec::from(&SUPPORTED_ALGORITHMS),
            observed_algorithm: manycore.routing_algo.clone(),
            sinks_sources: !manycore.sinks.is_empty() || !manycore.sources.is_empty(),
        };

        Ok(manycore)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{BTreeMap, HashMap},
        fs::read_to_string,
    };

    use crate::{
        sink_source::{Sink, SinkSourceDirection, Source},
        AttributeType, Channel, Channels, ConfigurableAttributes, Core, Cores, Directions, Edge,
        ChannelStatus, ManycoreSystem, Neighbours, Router, Task, TaskGraph, SUPPORTED_ALGORITHMS,
    };

    #[test]
    fn can_parse() {
        let expected_tasks = vec![
            Task::new(0, 40),
            Task::new(1, 80),
            Task::new(2, 60),
            Task::new(3, 40),
        ];

        let expected_edges = vec![
            Edge::new(0, 1, 3),
            Edge::new(0, 2, 2),
            Edge::new(1, 3, 3),
            Edge::new(2, 3, 1),
        ];

        let expected_graph = TaskGraph::new(expected_tasks, expected_edges);

        let expected_cores = vec![
            Core::new(
                0,
                Router::new(
                    0,
                    Some(BTreeMap::from([
                        ("@age".to_string(), "30".to_string()),
                        ("@temperature".to_string(), "30".to_string()),
                        ("@status".to_string(), "Normal".to_string()),
                    ])),
                ),
                None,
                Some(Channels::new(BTreeMap::from([
                    (
                        Directions::North,
                        Channel::new(Directions::North, 30, 0, None, ChannelStatus::Normal, 400),
                    ),
                    (
                        Directions::South,
                        Channel::new(Directions::South, 30, 1, Some(0), ChannelStatus::Normal, 400),
                    ),
                    (
                        Directions::East,
                        Channel::new(Directions::East, 30, 0, None, ChannelStatus::Normal, 400),
                    ),
                    (
                        Directions::West,
                        Channel::new(Directions::West, 30, 0, None, ChannelStatus::Normal, 400),
                    ),
                ]))),
                Some(BTreeMap::from([
                    ("@age".to_string(), "238".to_string()),
                    ("@temperature".to_string(), "45".to_string()),
                    ("@status".to_string(), "High".to_string()),
                ])),
            ),
            Core::new(
                1,
                Router::new(
                    1,
                    Some(BTreeMap::from([
                        ("@age".to_string(), "30".to_string()),
                        ("@temperature".to_string(), "30".to_string()),
                        ("@status".to_string(), "Normal".to_string()),
                    ])),
                ),
                Some(3),
                None,
                Some(BTreeMap::from([
                    ("@age".to_string(), "394".to_string()),
                    ("@temperature".to_string(), "30".to_string()),
                    ("@status".to_string(), "High".to_string()),
                ])),
            ),
            Core::new(
                2,
                Router::new(
                    2,
                    Some(BTreeMap::from([
                        ("@age".to_string(), "30".to_string()),
                        ("@temperature".to_string(), "30".to_string()),
                        ("@status".to_string(), "Normal".to_string()),
                    ])),
                ),
                None,
                None,
                Some(BTreeMap::from([
                    ("@age".to_string(), "157".to_string()),
                    ("@temperature".to_string(), "30".to_string()),
                    ("@status".to_string(), "High".to_string()),
                ])),
            ),
            Core::new(
                3,
                Router::new(
                    3,
                    Some(BTreeMap::from([
                        ("@age".to_string(), "30".to_string()),
                        ("@temperature".to_string(), "30".to_string()),
                        ("@status".to_string(), "Normal".to_string()),
                    ])),
                ),
                None,
                None,
                Some(BTreeMap::from([
                    ("@age".to_string(), "225".to_string()),
                    ("@temperature".to_string(), "30".to_string()),
                    ("@status".to_string(), "High".to_string()),
                ])),
            ),
            Core::new(
                4,
                Router::new(
                    4,
                    Some(BTreeMap::from([
                        ("@age".to_string(), "30".to_string()),
                        ("@temperature".to_string(), "30".to_string()),
                        ("@status".to_string(), "Normal".to_string()),
                    ])),
                ),
                Some(1),
                None,
                Some(BTreeMap::from([
                    ("@age".to_string(), "478".to_string()),
                    ("@temperature".to_string(), "30".to_string()),
                    ("@status".to_string(), "High".to_string()),
                ])),
            ),
            Core::new(
                5,
                Router::new(
                    5,
                    Some(BTreeMap::from([
                        ("@age".to_string(), "30".to_string()),
                        ("@temperature".to_string(), "30".to_string()),
                        ("@status".to_string(), "Normal".to_string()),
                    ])),
                ),
                None,
                None,
                Some(BTreeMap::from([
                    ("@age".to_string(), "105".to_string()),
                    ("@temperature".to_string(), "30".to_string()),
                    ("@status".to_string(), "High".to_string()),
                ])),
            ),
            Core::new(
                6,
                Router::new(
                    6,
                    Some(BTreeMap::from([
                        ("@age".to_string(), "30".to_string()),
                        ("@temperature".to_string(), "30".to_string()),
                        ("@status".to_string(), "Normal".to_string()),
                    ])),
                ),
                Some(0),
                None,
                Some(BTreeMap::from([
                    ("@age".to_string(), "18".to_string()),
                    ("@temperature".to_string(), "30".to_string()),
                    ("@status".to_string(), "High".to_string()),
                ])),
            ),
            Core::new(
                7,
                Router::new(
                    7,
                    Some(BTreeMap::from([
                        ("@age".to_string(), "30".to_string()),
                        ("@temperature".to_string(), "30".to_string()),
                        ("@status".to_string(), "Normal".to_string()),
                    ])),
                ),
                Some(2),
                None,
                Some(BTreeMap::from([
                    ("@age".to_string(), "15".to_string()),
                    ("@temperature".to_string(), "30".to_string()),
                    ("@status".to_string(), "High".to_string()),
                ])),
            ),
            Core::new(
                8,
                Router::new(
                    8,
                    Some(BTreeMap::from([
                        ("@age".to_string(), "30".to_string()),
                        ("@temperature".to_string(), "30".to_string()),
                        ("@status".to_string(), "Normal".to_string()),
                    ])),
                ),
                None,
                None,
                Some(BTreeMap::from([
                    ("@age".to_string(), "10".to_string()),
                    ("@temperature".to_string(), "30".to_string()),
                    ("@status".to_string(), "High".to_string()),
                ])),
            ),
        ];

        let expected_connections: HashMap<usize, Neighbours> = HashMap::from([
            (0, Neighbours::new(None, Some(1), Some(3), None)),
            (1, Neighbours::new(None, Some(2), Some(4), Some(0))),
            (2, Neighbours::new(None, None, Some(5), Some(1))),
            (3, Neighbours::new(Some(0), Some(4), Some(6), None)),
            (4, Neighbours::new(Some(1), Some(5), Some(7), Some(3))),
            (5, Neighbours::new(Some(2), None, Some(8), Some(4))),
            (6, Neighbours::new(Some(3), Some(7), None, None)),
            (7, Neighbours::new(Some(4), Some(8), None, Some(6))),
            (8, Neighbours::new(Some(5), None, None, Some(7))),
        ]);

        let expected_configurable_attributes = ConfigurableAttributes {
            core: HashMap::from([
                ("@id".to_string(), AttributeType::Text),
                ("@coordinates".to_string(), AttributeType::Text),
                ("@age".to_string(), AttributeType::Number),
                ("@temperature".to_string(), AttributeType::Number),
                ("@status".to_string(), AttributeType::Text),
            ]),
            router: HashMap::from([
                ("@age".to_string(), AttributeType::Number),
                ("@temperature".to_string(), AttributeType::Number),
                ("@status".to_string(), AttributeType::Text),
            ]),
            algorithms: Vec::from(&SUPPORTED_ALGORITHMS),
            observed_algorithm: Some(String::from("RowFirst")),
            sinks_sources: true,
        };

        let expected_task_core_map = HashMap::from([
            (0u16, 6usize),
            (1u16, 4usize),
            (2u16, 7usize),
            (3u16, 1usize),
        ]);

        let expected_manycore = ManycoreSystem {
            xmlns: String::from(
                "https://www.york.ac.uk/physics-engineering-technology/ManycoreSystems",
            ),
            xmlns_si: String::from("http://www.w3.org/2001/XMLSchema-instance"),
            xsi_schema_location: String::from("https://www.york.ac.uk/physics-engineering-technology/ManycoreSystems https://gist.githubusercontent.com/joe2k01/718e437790047ca14447af3b8309ef76/raw/a13c07cb6052fb16dea4d1b311a0bb0db48519b2/manycore_schema.xsd"),
            columns: 3,
            rows: 3,
            routing_algo: Some(String::from("RowFirst")),
            // sources: Some(Vec::from([Source::new(1, SinkSourceDirection::North)])),
            sources: BTreeMap::from([(1, Source::new(1, SinkSourceDirection::North))]),
            sinks: BTreeMap::from([(5, Sink::new(5, SinkSourceDirection::East))]),
            cores: Cores::new(expected_cores),
            task_graph: expected_graph,
            connections: expected_connections,
            task_core_map: expected_task_core_map,
            configurable_attributes: expected_configurable_attributes
        };

        let manycore = ManycoreSystem::parse_file("tests/VisualiserOutput1.xml")
            .expect("Could not read input test file \"tests/VisualiserOutput1.xml\"");

        assert_eq!(manycore, expected_manycore)
    }

    #[test]
    fn can_serialize() {
        let manycore = ManycoreSystem::parse_file("tests/VisualiserOutput1.xml")
            .expect("Could not read input test file \"tests/VisualiserOutput1.xml\"");

        let res = quick_xml::se::to_string(&manycore).expect("Could not serialize ManyCore");

        let expected = read_to_string("tests/serialized.xml")
            .expect("Could not read input test file \"tests/serialized.xml\"");

        assert_eq!(res, expected)
    }
}
