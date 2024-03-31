use getset::{Getters, MutGetters};
use manycore_utils::BTreeVector;
use serde::{Deserialize, Serialize};

use super::SinkSourceDirection;

#[derive(Serialize, Deserialize, Getters, Debug, PartialEq, Clone, Eq, MutGetters)]
#[getset(get = "pub")]
pub struct Source {
    #[serde(rename = "@coreID")]
    core_id: usize,
    #[serde(rename = "@direction")]
    direction: SinkSourceDirection,
    #[serde(rename = "@taskid")]
    task_id: u16,
    #[serde(skip)]
    #[getset(get = "pub")]
    current_load: u16,
}

impl Source {
    pub fn add_to_load(&mut self, load: u16) {
        self.current_load += load;
    }

    pub fn clear_load(&mut self) {
        self.current_load = 0;
    }
}

impl BTreeVector<u16> for Source {
    fn key(&self) -> u16 {
        self.task_id
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
            current_load: 0,
        }
    }
}
