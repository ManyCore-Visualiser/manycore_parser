use getset::Getters;
use serde::{Deserialize, Serialize};

use crate::utils::btree::{BTreeVector, BTreeVectorKeys};

use super::SinkSourceDirection;

#[derive(Serialize, Deserialize, Getters, Debug, PartialEq, Clone)]
#[getset(get = "pub")]
pub struct Sink {
    #[serde(rename = "@coreID")]
    core_id: usize,
    #[serde(rename = "@direction")]
    direction: SinkSourceDirection,
    #[serde(rename = "@taskid")]
    task_id: u16,
}

impl BTreeVector for Sink {
    fn key(self) -> (BTreeVectorKeys, Self) {
        let key = self.core_id.clone();

        (BTreeVectorKeys::usize(key), self)
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