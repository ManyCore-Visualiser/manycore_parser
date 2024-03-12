use std::collections::BTreeMap;

use getset::Getters;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Absolute fever dream of a trait, not sure the typing on the generic is good practice.
/// Handles a Vector of elements to be serialized as a BTreeMap and deserialized back into a Vec.
/// The BTreeMap key is core_id.
pub trait BTreeMapVector<'a, T>
where
    T: BTreeMapVector<'a, T> + Deserialize<'a> + Serialize,
{
    fn core_id(&self) -> &usize;

    fn deserialize_btreemap_vector<'de, D>(deserializer: D) -> Result<BTreeMap<usize, T>, D::Error>
    where
        D: Deserializer<'de>,
        'a: 'de,
        'de: 'a,
    {
        let list: Vec<T> = Deserialize::deserialize(deserializer)?;
        let mut ret: BTreeMap<usize, T> = BTreeMap::new();
        for element in list {
            ret.insert(*element.core_id(), element);
        }

        Ok(ret)
    }

    fn serialize_btreemap_vector<S>(
        map: &BTreeMap<usize, T>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_seq(map.values())
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, PartialOrd, Ord)]
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

impl BTreeMapVector<'_, Source> for Source {
    fn core_id(&self) -> &usize {
        &self.core_id
    }
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

impl BTreeMapVector<'_, Sink> for Sink {
    fn core_id(&self) -> &usize {
        &self.core_id
    }
}

impl Sink {
    pub fn new(core_id: usize, direction: SinkSourceDirection) -> Self {
        Self { core_id, direction }
    }
}
