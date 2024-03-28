use getset::Getters;
use serde::{Deserialize, Serialize};

use crate::utils::btree::{BTreeVector, BTreeVectorKeys};

use super::SinkSourceDirection;

#[derive(Serialize, Deserialize, Getters, Debug, PartialEq, Clone, Eq)]
#[getset(get = "pub")]
pub struct Source {
    #[serde(rename = "@coreID")]
    core_id: usize,
    #[serde(rename = "@direction")]
    direction: SinkSourceDirection,
    #[serde(rename = "@taskid")]
    task_id: u16,
}

impl BTreeVector for Source {
    fn key(self) -> (BTreeVectorKeys, Self) {
        let key = self.task_id.clone();

        (BTreeVectorKeys::u16(key), self)
    }
}

impl Ord for Source {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.core_id.cmp(&other.core_id)
    }
}

impl PartialOrd for Source {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.core_id.partial_cmp(&other.core_id)
    }
}

#[cfg(test)]
impl Source {
    pub fn new(core_id: usize, direction: SinkSourceDirection, task_id: u16) -> Self {
        Self {
            core_id,
            direction,
            task_id,
        }
    }
}
