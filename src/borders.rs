use std::collections::BTreeMap;

use getset::Getters;
use serde::{Deserialize, Serialize};

use crate::utils::btree::{BTreeVector, BTreeVectorKeys};
use crate::Directions;

use self::sink::Sink;
use self::source::Source;

pub mod sink;
pub mod source;

#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, PartialOrd, Ord, Clone)]
pub enum SinkSourceDirection {
    North,
    South,
    East,
    West,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct Borders {
    #[serde(
        rename = "Source",
        skip_serializing_if = "BTreeMap::is_empty",
        serialize_with = "Source::serialize_btree_vector",
        deserialize_with = "Source::deserialize_btree_vector"
    )]
    sources: BTreeMap<BTreeVectorKeys, Source>,
    #[serde(
        rename = "Sink",
        skip_serializing_if = "BTreeMap::is_empty",
        serialize_with = "Sink::serialize_btree_vector",
        deserialize_with = "Sink::deserialize_btree_vector"
    )]
    sinks: BTreeMap<BTreeVectorKeys, Sink>,
}

impl Borders {
    pub fn should_skip_serialize(&self) -> bool {
        self.sinks.is_empty() || self.sources.is_empty()
    }

    #[cfg(test)]
    pub fn new(
        sinks: BTreeMap<BTreeVectorKeys, Sink>,
        sources: BTreeMap<BTreeVectorKeys, Source>,
    ) -> Self {
        Self { sinks, sources }
    }
}

impl From<&SinkSourceDirection> for Directions {
    fn from(value: &SinkSourceDirection) -> Self {
        match value {
            SinkSourceDirection::North => Directions::North,
            SinkSourceDirection::South => Directions::South,
            SinkSourceDirection::West => Directions::West,
            SinkSourceDirection::East => Directions::East,
        }
    }
}
