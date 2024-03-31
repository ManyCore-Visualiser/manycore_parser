use std::collections::BTreeMap;

use getset::{Getters, MutGetters};
use manycore_utils::{deserialize_btree_vector, serialise_btreemap_and_sort};
use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Debug, PartialEq, Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct Borders {
    #[serde(
        rename = "Source",
        skip_serializing_if = "BTreeMap::is_empty",
        serialize_with = "serialise_btreemap_and_sort",
        deserialize_with = "deserialize_btree_vector"
    )]
    sources: BTreeMap<u16, Source>,
    #[serde(
        rename = "Sink",
        skip_serializing_if = "BTreeMap::is_empty",
        serialize_with = "serialise_btreemap_and_sort",
        deserialize_with = "deserialize_btree_vector"
    )]
    sinks: BTreeMap<u16, Sink>,
}

impl Borders {
    pub fn should_skip_serialize(&self) -> bool {
        self.sinks.is_empty() || self.sources.is_empty()
    }

    #[cfg(test)]
    pub fn new(sinks: BTreeMap<u16, Sink>, sources: BTreeMap<u16, Source>) -> Self {
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
