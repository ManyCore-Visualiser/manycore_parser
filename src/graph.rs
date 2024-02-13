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

impl Edge {
    pub fn new(from: u16, to: u16, communication_cost: u8) -> Self {
        Self {
            from,
            to,
            communication_cost,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Task {
    #[serde(rename = "@id")]
    id: u16,
    #[serde(rename = "@computationCost")]
    computation_cost: u8,
}

impl Task {
    pub fn new(id: u16, computation_cost: u8) -> Self {
        Self {
            id,
            computation_cost,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct TaskGraph {
    #[serde(rename = "Task")]
    tasks: Vec<Task>,
    #[serde(rename = "Edge")]
    edges: Vec<Edge>,
}

impl TaskGraph {
    pub fn new(tasks: Vec<Task>, edges: Vec<Edge>) -> Self {
        Self { tasks, edges }
    }
}