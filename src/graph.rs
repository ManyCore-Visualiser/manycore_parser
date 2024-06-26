use std::collections::BTreeMap;
use manycore_utils::{deserialize_btree_vector, serialise_btreemap, BTreeVector};
use getset::{Getters, MutGetters};
use serde::{Deserialize, Serialize};

/// Object representation of an `<Edge>` element in input XML.
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
    communication_cost: u16,
}

impl Edge {
    #[cfg(test)]
    /// Instantiates a new edge.
    pub(crate) fn new(from: u16, to: u16, communication_cost: u16) -> Self {
        Self {
            from,
            to,
            communication_cost,
        }
    }
}

/// Object representation of a `<Task>` element in input XML.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Getters)]
#[getset(get = "pub")]
pub struct Task {
    #[serde(rename = "@id")]
    id: u16,
    #[serde(rename = "@computationCost")]
    computation_cost: u8,
}

impl Task {
    #[cfg(test)]
    /// Instantiates a new task.
    pub(crate) fn new(id: u16, computation_cost: u8) -> Self {
        Self {
            id,
            computation_cost,
        }
    }
}

impl BTreeVector<u16> for Task {
    fn key(&self) -> u16 {
        self.id
    }
}

/// Object representation of `<TaskGrraph>` element in input XML.
#[derive(Serialize, Deserialize, Debug, PartialEq, Getters, MutGetters, Clone)]
pub struct TaskGraph {
    /// Vector of tasks in the graph (graph nodes).
    #[serde(rename = "Task")]
    #[getset(get = "pub", get_mut)]
    #[serde(
        deserialize_with = "deserialize_btree_vector",
        serialize_with = "serialise_btreemap"
    )]
    tasks: BTreeMap<u16, Task>,
    /// Vector of edges connecting tasks (grpah edges).
    #[serde(rename = "Edge")]
    #[getset(get = "pub", get_mut = "pub")]
    edges: Vec<Edge>,
}

impl TaskGraph {
    #[cfg(test)]
    /// Instantiates a new Taskgraph.
    pub(crate) fn new(tasks: BTreeMap<u16, Task>, edges: Vec<Edge>) -> Self {
        Self { tasks, edges }
    }
}
