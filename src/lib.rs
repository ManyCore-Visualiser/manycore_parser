//! A parser for Manycore System XML configuration files

use std::{collections::HashMap, hash::Hash};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Edge {
    #[serde(rename = "@from")]
    from: u16,
    #[serde(rename = "@to")]
    to: u16,
    #[serde(rename = "@communicationCost")]
    communication_cost: u8,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Task {
    #[serde(rename = "@id")]
    id: u16,
    #[serde(rename = "@computationCost")]
    computation_cost: u8,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct TaskGraph {
    #[serde(rename = "Task")]
    tasks: Vec<Task>,
    #[serde(rename = "Edge")]
    edges: Vec<Edge>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum RouterStatus {
    Broken,
    Normal,
}

impl Default for RouterStatus {
    fn default() -> Self {
        RouterStatus::Broken
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Router {
    #[serde(rename = "@age")]
    age: u8,
    #[serde(rename = "@temperature")]
    temperature: u8,
    #[serde(rename = "@status")]
    status: RouterStatus,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum CoreStatus {
    Broken,
    Low,
    Mid,
    High,
}

impl Default for CoreStatus {
    fn default() -> Self {
        CoreStatus::Broken
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Core {
    #[serde(rename = "@id")]
    id: u8,
    #[serde(rename = "@age")]
    age: u16,
    #[serde(rename = "@temperature")]
    temperature: u8,
    #[serde(rename = "@status")]
    status: CoreStatus,
    #[serde(rename = "Router")]
    router: Router,
    #[serde(rename = "@allocated_task", skip_serializing_if = "Option::is_none")]
    allocated_task: Option<u16>,
}

impl Hash for Core {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // We can take a shortcut here as IDs are unique for our cores
        self.id.hash(state);
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Cores {
    #[serde(rename = "Core")]
    pub list: Vec<Core>,
}

#[derive(Default, Debug, PartialEq)]
pub struct Neighbours {
    top: Option<usize>,
    right: Option<usize>,
    bottom: Option<usize>,
    left: Option<usize>,
}

impl Neighbours {
    pub fn top(&self) -> Option<usize> {
        self.top
    }

    pub fn set_top(&mut self, top: Option<usize>) {
        self.top = top;
    }

    pub fn right(&self) -> Option<usize> {
        self.right
    }

    pub fn set_right(&mut self, right: Option<usize>) {
        self.right = right;
    }

    pub fn bottom(&self) -> Option<usize> {
        self.bottom
    }

    pub fn set_bottom(&mut self, bottom: Option<usize>) {
        self.bottom = bottom;
    }

    pub fn left(&self) -> Option<usize> {
        self.left
    }

    pub fn set_left(&mut self, left: Option<usize>) {
        self.left = left;
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
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
    #[serde(rename = "@rows")]
    pub rows: u8,
    #[serde(rename = "@columns")]
    pub columns: u8,
    #[serde(rename = "@routing_algo")]
    routing_algo: String,
    task_graph: TaskGraph,
    pub cores: Cores,
    #[serde(skip)]
    pub connections: HashMap<usize, Neighbours>,
}

impl ManycoreSystem {
    pub fn parse_file(path: &str) -> Result<ManycoreSystem, Box<dyn std::error::Error>> {
        let file_content = std::fs::read_to_string(path)?;

        let mut manycore: ManycoreSystem = quick_xml::de::from_str(&file_content)?;

        // Sort cores by id
        manycore
            .cores
            .list
            .sort_by(|me, other| me.id.cmp(&other.id));

        let usize_columns = usize::from(manycore.columns);
        let last = manycore.cores.list.len() - 1;
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

        Ok(manycore)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        Core, CoreStatus, Cores, Edge, ManycoreSystem, Neighbours, Router, RouterStatus, Task,
        TaskGraph,
    };

    #[test]
    fn can_parse() {
        let expected_tasks = vec![
            Task {
                id: 0,
                computation_cost: 40,
            },
            Task {
                id: 1,
                computation_cost: 80,
            },
            Task {
                id: 2,
                computation_cost: 60,
            },
            Task {
                id: 3,
                computation_cost: 40,
            },
        ];

        let expected_edges = vec![
            Edge {
                from: 0,
                to: 1,
                communication_cost: 3,
            },
            Edge {
                from: 0,
                to: 2,
                communication_cost: 2,
            },
            Edge {
                from: 1,
                to: 3,
                communication_cost: 3,
            },
            Edge {
                from: 2,
                to: 3,
                communication_cost: 1,
            },
        ];

        let expected_graph = TaskGraph {
            tasks: expected_tasks,
            edges: expected_edges,
        };

        let expected_cores = vec![
            Core {
                id: 0,
                age: 238,
                status: CoreStatus::High,
                temperature: 45,
                allocated_task: None,
                router: Router {
                    age: 30,
                    status: RouterStatus::Normal,
                    temperature: 30,
                },
            },
            Core {
                id: 1,
                age: 394,
                status: CoreStatus::High,
                temperature: 30,
                allocated_task: Some(3),
                router: Router {
                    age: 30,
                    status: RouterStatus::Normal,
                    temperature: 30,
                },
            },
            Core {
                id: 2,
                age: 157,
                status: CoreStatus::High,
                temperature: 30,
                allocated_task: None,
                router: Router {
                    age: 30,
                    status: RouterStatus::Normal,
                    temperature: 30,
                },
            },
            Core {
                id: 3,
                age: 225,
                status: CoreStatus::High,
                temperature: 30,
                allocated_task: None,
                router: Router {
                    age: 30,
                    status: RouterStatus::Normal,
                    temperature: 30,
                },
            },
            Core {
                id: 4,
                age: 478,
                status: CoreStatus::High,
                temperature: 30,
                allocated_task: Some(1),
                router: Router {
                    age: 30,
                    status: RouterStatus::Normal,
                    temperature: 30,
                },
            },
            Core {
                id: 5,
                age: 105,
                status: CoreStatus::High,
                temperature: 30,
                allocated_task: None,
                router: Router {
                    age: 30,
                    status: RouterStatus::Normal,
                    temperature: 30,
                },
            },
            Core {
                id: 6,
                age: 18,
                status: CoreStatus::High,
                temperature: 30,
                allocated_task: Some(0),
                router: Router {
                    age: 30,
                    status: RouterStatus::Normal,
                    temperature: 30,
                },
            },
            Core {
                id: 7,
                age: 15,
                status: CoreStatus::High,
                temperature: 30,
                allocated_task: Some(2),
                router: Router {
                    age: 30,
                    status: RouterStatus::Normal,
                    temperature: 30,
                },
            },
            Core {
                id: 8,
                age: 10,
                status: CoreStatus::High,
                temperature: 30,
                allocated_task: None,
                router: Router {
                    age: 30,
                    status: RouterStatus::Normal,
                    temperature: 30,
                },
            },
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

        let expected_manycore = ManycoreSystem {
            xmlns: String::from(
                "https://www.york.ac.uk/physics-engineering-technology/ManycoreSystems",
            ),
            xmlns_si: String::from("http://www.w3.org/2001/XMLSchema-instance"),
            xsi_schema_location: String::from("https://www.york.ac.uk/physics-engineering-technology/ManycoreSystems https://gist.githubusercontent.com/joe2k01/718e437790047ca14447af3b8309ef76/raw/a8e362dd5250ccdcb517a82774303dee2b0ab8d9/manycore_schema.xsd"),
            columns: 3,
            rows: 3,
            routing_algo: String::from("RowFirst"),
            cores: Cores {
                list: expected_cores,
            },
            task_graph: expected_graph,
            connections: expected_connections
        };

        let manycore = ManycoreSystem::parse_file("tests/VisualiserOutput1.xml")
            .expect("Could not read input test file \"tests/VisualiserOutput1.xml\"");

        assert_eq!(manycore, expected_manycore)
    }
}
