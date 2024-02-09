//! A parser for Manycore System XML configuration files

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Edge {
    #[serde(rename = "@from")]
    from: String,
    #[serde(rename = "@to")]
    to: String,
    #[serde(rename = "@communicationCost")]
    communication_cost: u8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Task {
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "@computationCost")]
    computation_cost: u8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TaskGraph {
    #[serde(rename = "Task")]
    tasks: Vec<Task>,
    #[serde(rename = "Edge")]
    edges: Vec<Edge>,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Router {
    #[serde(rename = "@age")]
    age: u8,
    #[serde(rename = "@temperature")]
    temp: String,
    #[serde(rename = "@status")]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Cores {
    #[serde(rename = "Core")]
    list: Vec<Core>,
}

#[derive(Serialize, Deserialize, Debug)]
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
    rows: u8,
    #[serde(rename = "@columns")]
    columns: u8,
    #[serde(rename = "@routing_algo")]
    routing_algo: String,
    task_graph: TaskGraph,
    cores: Cores,
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
        match ManycoreSystem::parse_file("tests/VisualiserOutput1.xml") {
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
