use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
#[allow(non_camel_case_types)]
pub enum BTreeVectorKeys {
    usize(usize),
    u16(u16),
}

pub trait BTreeVector: Clone {
    fn key(self) -> (BTreeVectorKeys, Self);

    fn deserialize_btree_vector<'de, D, T>(
        deserializer: D,
    ) -> Result<BTreeMap<BTreeVectorKeys, T>, D::Error>
    where
        D: Deserializer<'de>,
        T: BTreeVector + Deserialize<'de> + 'de,
    {
        let list: Vec<T> = Deserialize::deserialize(deserializer)?;
        let mut ret = BTreeMap::new();
        for element in list {
            let (k, e) = element.key();
            ret.insert(k, e);
        }

        Ok(ret)
    }

    fn serialize_btree_vector<S, T>(
        map: &BTreeMap<BTreeVectorKeys, T>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize + Ord,
    {
        let mut vec = map.values().collect::<Vec<&T>>();
        vec.sort();

        serializer.collect_seq(vec)
    }
}