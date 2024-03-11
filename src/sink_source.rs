use getset::Getters;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum SinkSourceDirection {
    North,
    South,
    East,
    West,
}

#[derive(Serialize, Deserialize, Getters, Debug, PartialEq)]
#[getset(get = "pub")]
pub struct Source {
    #[serde(rename = "@coreID")]
    core_id: usize,
    #[serde(rename = "@direction")]
    direction: SinkSourceDirection,
}

impl Source {
    pub fn new(core_id: usize, direction: SinkSourceDirection) -> Self {
        Self { core_id, direction }
    }
}

#[derive(Serialize, Deserialize, Getters, Debug, PartialEq)]
#[getset(get = "pub")]
pub struct Sink {
    #[serde(rename = "@coreID")]
    core_id: usize,
    #[serde(rename = "@direction")]
    direction: SinkSourceDirection,
}

impl Sink {
    pub fn new(core_id: usize, direction: SinkSourceDirection) -> Self {
        Self { core_id, direction }
    }
}
