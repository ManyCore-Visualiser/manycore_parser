use getset::Getters;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, PartialOrd, Ord, Clone)]
pub enum SinkSourceDirection {
    North,
    South,
    East,
    West,
}

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

#[derive(Serialize, Deserialize, Getters, Debug, PartialEq, Clone)]
#[getset(get = "pub")]
pub struct Source {
    #[serde(rename = "@coreID")]
    core_id: usize,
    #[serde(rename = "@direction")]
    direction: SinkSourceDirection,
    #[serde(rename = "@taskid")]
    task_id: u16,
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

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Borders {
    #[serde(rename = "Source", skip_serializing_if = "Vec::is_empty")]
    sources: Vec<Source>,
    #[serde(rename = "Sink", skip_serializing_if = "Vec::is_empty")]
    sinks: Vec<Sink>,
}

impl Borders {
    pub fn should_skip_serialize(&self) -> bool {
        self.sinks.is_empty() || self.sources.is_empty()
    }

    #[cfg(test)]
    pub fn new(sinks: Vec<Sink>, sources: Vec<Source>) -> Self {
        Self { sinks, sources }
    }
}
