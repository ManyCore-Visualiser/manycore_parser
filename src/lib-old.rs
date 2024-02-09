//! A parser for Manycore System XML configuration files
//!
//! # Description

mod flatten_deserialize;

use flatten_deserialize::from_map_entry;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};

#[derive(Debug)]
pub struct Edge {
    to: Task,
    cost: u8,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct TaskConnection {
    #[serde(rename = "@from")]
    from: String,
    #[serde(rename = "@to")]
    to: String,
    communication_cost: u8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Graph {
    #[serde(rename = "Connection")]
    links: Vec<TaskConnection>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Task {
    #[serde(rename = "@id")]
    id: String,
    computation_cost: u8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tasks {
    #[serde(rename = "Task")]
    list: Vec<Task>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RouterStatus {
    Broken,
    Normal,
}

impl Default for RouterStatus {
    fn default() -> Self {
        RouterStatus::Broken
    }
}

impl FromStr for RouterStatus {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Normal" => Ok(RouterStatus::Normal),
            _ => Ok(RouterStatus::Broken),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Router {
    #[serde(rename = "RouterAge", deserialize_with = "from_map_entry")]
    age: u8,
    #[serde(rename = "RouterTemp", deserialize_with = "from_map_entry")]
    temp: String,
    #[serde(rename = "RouterStatus", deserialize_with = "from_map_entry")]
    status: RouterStatus,
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Core {
    #[serde(rename = "@id")]
    id: u8,
    age: u16,
    temperature: String,
    status: CoreStatus,
    #[serde(flatten)]
    router: Router,
    #[serde(rename = "LocatdedTaskID")]
    task: u16,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Cores {
    #[serde(rename = "Core")]
    list: Vec<Core>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ManycoreSystem {
    #[serde(rename = "NumRows")]
    rows: u8,
    #[serde(rename = "NumColumns")]
    columns: u8,
    #[serde(rename = "RoutingAlgo")]
    algo: String,
    cores: Cores,
    tasks: Tasks,
    task_graph: Graph,
    #[serde(skip)]
    adj_list: HashMap<u8, Vec<Edge>>
}

impl ManycoreSystem {
    pub fn parse_file(path: &str) -> Result<ManycoreSystem, Box<dyn std::error::Error>> {
        let file_content = std::fs::read_to_string(path)?;

        let manycore: ManycoreSystem = quick_xml::de::from_str(&file_content)?;

        Ok(manycore)
    }
}

#[cfg(test)]
mod tests {
    use crate::ManycoreSystem;

    #[test]
    fn can_parse() {
        match ManycoreSystem::parse_file("tests/VisualiserOutput0.xml") {
            Ok(manycore) => {
                println!("All Ok");
                println!("{:?}", manycore);

                println!("{}", quick_xml::se::to_string(&manycore).unwrap());
            }
            Err(e) => {
                println!("{}", e)
            }
        }

        assert_eq!(1, 1)
    }
}
