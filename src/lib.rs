//! A parser for Manycore System XML configuration files

mod cores;
mod graph;
mod router;
mod utils;

use std::collections::HashMap;

pub use crate::cores::*;
pub use crate::graph::*;
pub use crate::router::*;
use getset::CopyGetters;
use getset::{Getters, MutGetters, Setters};
use serde::{Deserialize, Serialize};

pub trait WithXMLAttributes {
    fn id(&self) -> Option<&u8>;
    fn other_attributes(&self) -> &Option<HashMap<String, String>>;
}

#[derive(Default, Debug, PartialEq, Getters, Setters)]
#[getset(get = "pub", set = "pub")]
pub struct Neighbours {
    top: Option<usize>,
    right: Option<usize>,
    bottom: Option<usize>,
    left: Option<usize>,
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
pub struct ConfigurableAttributes {
    core: HashMap<String, AttributeType>,
    router: HashMap<String, AttributeType>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Getters, Setters, MutGetters)]
#[serde(rename_all = "PascalCase")]
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
    rows: u8,
    #[serde(rename = "@columns")]
    #[getset(get = "pub")]
    columns: u8,
    #[serde(rename = "@routing_algo")]
    #[getset(get = "pub")]
    routing_algo: String,
    #[getset(get = "pub", set = "pub", get_mut = "pub")]
    task_graph: TaskGraph,
    #[getset(get = "pub", set = "pub", get_mut = "pub")]
    cores: Cores,
    #[serde(skip)]
    #[getset(get = "pub", set = "pub", get_mut = "pub")]
    connections: HashMap<usize, Neighbours>,
    #[serde(skip)]
    #[getset(get = "pub")]
    configurable_attributes: ConfigurableAttributes,
}

impl ManycoreSystem {
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
            .cores
            .list_mut()
            .sort_by(|me, other| me.id().cmp(&other.id()));

        // Populate neighbour connections
        let usize_columns = usize::from(manycore.columns);
        let last = manycore.cores.list().len() - 1;
        for i in 0..=last {
            let right = i + 1;
            let top = i >= usize_columns;
            let bottom = i + usize_columns;
            let mut neighbours = Neighbours::default();

            // Right
            if right % usize_columns != 0 {
                neighbours.set_right(Some(right));
            }

            // Left
            if i % usize_columns != 0 {
                neighbours.set_left(Some(i - 1));
            }

            // Top
            if top {
                neighbours.set_top(Some(i - usize_columns));
            }

            // Bottom
            if bottom <= last {
                neighbours.set_bottom(Some(bottom));
            }

            manycore.connections.insert(i, neighbours);
        }

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
        };

        Ok(manycore)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        AttributeType, ConfigurableAttributes, Core, Cores, Edge, FIFOs, ManycoreSystem,
        Neighbours, Router, Task, TaskGraph,
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
                Router::new(Some(HashMap::from([
                    ("@age".to_string(), "30".to_string()),
                    ("@temperature".to_string(), "30".to_string()),
                    ("@status".to_string(), "Normal".to_string()),
                ]))),
                None,
                Some(FIFOs {}),
                Some(HashMap::from([
                    ("@age".to_string(), "238".to_string()),
                    ("@temperature".to_string(), "45".to_string()),
                    ("@status".to_string(), "High".to_string()),
                ])),
            ),
            Core::new(
                1,
                Router::new(Some(HashMap::from([
                    ("@age".to_string(), "30".to_string()),
                    ("@temperature".to_string(), "30".to_string()),
                    ("@status".to_string(), "Normal".to_string()),
                ]))),
                Some(3),
                None,
                Some(HashMap::from([
                    ("@age".to_string(), "394".to_string()),
                    ("@temperature".to_string(), "30".to_string()),
                    ("@status".to_string(), "High".to_string()),
                ])),
            ),
            Core::new(
                2,
                Router::new(Some(HashMap::from([
                    ("@age".to_string(), "30".to_string()),
                    ("@temperature".to_string(), "30".to_string()),
                    ("@status".to_string(), "Normal".to_string()),
                ]))),
                None,
                None,
                Some(HashMap::from([
                    ("@age".to_string(), "157".to_string()),
                    ("@temperature".to_string(), "30".to_string()),
                    ("@status".to_string(), "High".to_string()),
                ])),
            ),
            Core::new(
                3,
                Router::new(Some(HashMap::from([
                    ("@age".to_string(), "30".to_string()),
                    ("@temperature".to_string(), "30".to_string()),
                    ("@status".to_string(), "Normal".to_string()),
                ]))),
                None,
                None,
                Some(HashMap::from([
                    ("@age".to_string(), "225".to_string()),
                    ("@temperature".to_string(), "30".to_string()),
                    ("@status".to_string(), "High".to_string()),
                ])),
            ),
            Core::new(
                4,
                Router::new(Some(HashMap::from([
                    ("@age".to_string(), "30".to_string()),
                    ("@temperature".to_string(), "30".to_string()),
                    ("@status".to_string(), "Normal".to_string()),
                ]))),
                Some(1),
                None,
                Some(HashMap::from([
                    ("@age".to_string(), "478".to_string()),
                    ("@temperature".to_string(), "30".to_string()),
                    ("@status".to_string(), "High".to_string()),
                ])),
            ),
            Core::new(
                5,
                Router::new(Some(HashMap::from([
                    ("@age".to_string(), "30".to_string()),
                    ("@temperature".to_string(), "30".to_string()),
                    ("@status".to_string(), "Normal".to_string()),
                ]))),
                None,
                None,
                Some(HashMap::from([
                    ("@age".to_string(), "105".to_string()),
                    ("@temperature".to_string(), "30".to_string()),
                    ("@status".to_string(), "High".to_string()),
                ])),
            ),
            Core::new(
                6,
                Router::new(Some(HashMap::from([
                    ("@age".to_string(), "30".to_string()),
                    ("@temperature".to_string(), "30".to_string()),
                    ("@status".to_string(), "Normal".to_string()),
                ]))),
                Some(0),
                None,
                Some(HashMap::from([
                    ("@age".to_string(), "18".to_string()),
                    ("@temperature".to_string(), "30".to_string()),
                    ("@status".to_string(), "High".to_string()),
                ])),
            ),
            Core::new(
                7,
                Router::new(Some(HashMap::from([
                    ("@age".to_string(), "30".to_string()),
                    ("@temperature".to_string(), "30".to_string()),
                    ("@status".to_string(), "Normal".to_string()),
                ]))),
                Some(2),
                None,
                Some(HashMap::from([
                    ("@age".to_string(), "15".to_string()),
                    ("@temperature".to_string(), "30".to_string()),
                    ("@status".to_string(), "High".to_string()),
                ])),
            ),
            Core::new(
                8,
                Router::new(Some(HashMap::from([
                    ("@age".to_string(), "30".to_string()),
                    ("@temperature".to_string(), "30".to_string()),
                    ("@status".to_string(), "Normal".to_string()),
                ]))),
                None,
                None,
                Some(HashMap::from([
                    ("@age".to_string(), "10".to_string()),
                    ("@temperature".to_string(), "30".to_string()),
                    ("@status".to_string(), "High".to_string()),
                ])),
            ),
        ];

        let expected_connections: HashMap<usize, Neighbours> = HashMap::from([
            (
                0,
                Neighbours {
                    top: None,
                    right: Some(1),
                    bottom: Some(3),
                    left: None,
                },
            ),
            (
                1,
                Neighbours {
                    top: None,
                    right: Some(2),
                    bottom: Some(4),
                    left: Some(0),
                },
            ),
            (
                2,
                Neighbours {
                    top: None,
                    right: None,
                    bottom: Some(5),
                    left: Some(1),
                },
            ),
            (
                3,
                Neighbours {
                    top: Some(0),
                    right: Some(4),
                    bottom: Some(6),
                    left: None,
                },
            ),
            (
                4,
                Neighbours {
                    top: Some(1),
                    right: Some(5),
                    bottom: Some(7),
                    left: Some(3),
                },
            ),
            (
                5,
                Neighbours {
                    top: Some(2),
                    right: None,
                    bottom: Some(8),
                    left: Some(4),
                },
            ),
            (
                6,
                Neighbours {
                    top: Some(3),
                    right: Some(7),
                    bottom: None,
                    left: None,
                },
            ),
            (
                7,
                Neighbours {
                    top: Some(4),
                    right: Some(8),
                    bottom: None,
                    left: Some(6),
                },
            ),
            (
                8,
                Neighbours {
                    top: Some(5),
                    right: None,
                    bottom: None,
                    left: Some(7),
                },
            ),
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
        };

        let expected_manycore = ManycoreSystem {
            xmlns: String::from(
                "https://www.york.ac.uk/physics-engineering-technology/ManycoreSystems",
            ),
            xmlns_si: String::from("http://www.w3.org/2001/XMLSchema-instance"),
            xsi_schema_location: String::from("https://www.york.ac.uk/physics-engineering-technology/ManycoreSystems https://gist.githubusercontent.com/joe2k01/718e437790047ca14447af3b8309ef76/raw/a8e362dd5250ccdcb517a82774303dee2b0ab8d9/manycore_schema.xsd"),
            columns: 3,
            rows: 3,
            routing_algo: String::from("RowFirst"),
            cores: Cores::new(expected_cores),
            task_graph: expected_graph,
            connections: expected_connections,
            configurable_attributes: expected_configurable_attributes
        };

        let manycore = ManycoreSystem::parse_file("tests/VisualiserOutput1.xml")
            .expect("Could not read input test file \"tests/VisualiserOutput1.xml\"");

        assert_eq!(manycore, expected_manycore)
    }
}
