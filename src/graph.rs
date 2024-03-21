use getset::{Getters, MutGetters};
use serde::{Deserialize, Serialize};

/// A Taskgraph edge.
#[derive(Serialize, Deserialize, Debug, PartialEq, Getters, Clone)]
#[serde(rename_all = "PascalCase")]
#[getset(get = "pub")]
pub struct Edge {
    /// Edge source.
    #[serde(rename = "@from")]
    from: u16,
    /// Edge destination
    #[serde(rename = "@to")]
    to: u16,
    /// Edge cost.
    #[serde(rename = "@communicationCost")]
    communication_cost: u8,
}

impl Edge {
    /// Instantiates a new edge.
    pub fn new(from: u16, to: u16, communication_cost: u8) -> Self {
        Self {
            from,
            to,
            communication_cost,
        }
    }
}

/// A Taskgraph task.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Task {
    #[serde(rename = "@id")]
    id: u16,
    #[serde(rename = "@computationCost")]
    computation_cost: u8,
}

impl Task {
    /// Instantiates a new task.
    pub fn new(id: u16, computation_cost: u8) -> Self {
        Self {
            id,
            computation_cost,
        }
    }
}

/// The system's Taskgraph.
#[derive(Serialize, Deserialize, Debug, PartialEq, Getters, MutGetters, Clone)]
pub struct TaskGraph {
    /// Vector of tasks in the graph (graph nodes).
    #[serde(rename = "Task")]
    tasks: Vec<Task>,
    /// Vector of edges connecting tasks (grpah edges).
    #[serde(rename = "Edge")]
    #[getset(get = "pub", get_mut = "pub")]
    edges: Vec<Edge>,
}

impl TaskGraph {
    /// Instantiates a new Taskgraph.
    pub fn new(tasks: Vec<Task>, edges: Vec<Edge>) -> Self {
        Self { tasks, edges }
    }
}
