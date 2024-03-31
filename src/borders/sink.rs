use getset::Getters;
use manycore_utils::BTreeVector;
use serde::{Deserialize, Serialize};

use super::SinkSourceDirection;

#[derive(Serialize, Deserialize, Getters, Debug, PartialEq, Clone, Eq)]
#[getset(get = "pub")]
pub struct Sink {
    #[serde(rename = "@coreID")]
    core_id: usize,
    #[serde(rename = "@direction")]
    direction: SinkSourceDirection,
    #[serde(rename = "@taskid")]
    task_id: u16,
}

impl BTreeVector<u16> for Sink {
    fn key(&self) -> u16 {
        self.task_id
    }
}

impl Ord for Sink {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.core_id.cmp(&other.core_id)
    }
}

impl PartialOrd for Sink {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.core_id.partial_cmp(&other.core_id)
    }
}

#[cfg(test)]
impl Sink {
    pub fn new(core_id: usize, direction: SinkSourceDirection, task_id: u16) -> Self {
        Self {
            core_id,
            direction,
            task_id,
        }
    }
}
