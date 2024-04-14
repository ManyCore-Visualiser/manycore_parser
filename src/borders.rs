use std::collections::{BTreeMap, HashMap};

use getset::{Getters, MutGetters};
use manycore_utils::{deserialize_btree_vector, serialise_btreemap_and_sort};
use serde::{Deserialize, Serialize};

use crate::Directions;

use self::sink::Sink;
use self::source::Source;

pub mod sink;
pub mod source;

#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum SinkSourceDirection {
    North,
    South,
    East,
    West,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum BorderEntry {
    Source(u16),
    Sink(u16),
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
    #[serde(skip)]
    #[getset(get = "pub")]
    core_border_map: HashMap<usize, HashMap<SinkSourceDirection, BorderEntry>>,
}

impl Borders {
    #[cfg(test)]
    pub fn new(
        sinks: BTreeMap<u16, Sink>,
        sources: BTreeMap<u16, Source>,
        core_border_map: HashMap<usize, HashMap<SinkSourceDirection, BorderEntry>>,
    ) -> Self {
        Self {
            sinks,
            sources,
            core_border_map,
        }
    }

    pub fn compute_core_border_map(&mut self) {
        for source in self.sources.values() {
            self.core_border_map
                .entry(*source.core_id())
                .or_insert(HashMap::new())
                .insert(*source.direction(), BorderEntry::Source(*source.task_id()));
        }

        for sink in self.sinks.values() {
            self.core_border_map
                .entry(*sink.core_id())
                .or_insert(HashMap::new())
                .insert(*sink.direction(), BorderEntry::Sink(*sink.task_id()));
        }
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
