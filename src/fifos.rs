use std::collections::BTreeMap;

use getset::Getters;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub enum FIFODirection {
    NorthInput,
    NorthOutput,
    SouthInput,
    SouthOutput,
    EastInput,
    EastOutput,
    WestInput,
    WestOutput,
    LocalInput,
    LocalOutput,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum FIFOStatus {
    Normal,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Getters)]
pub struct FIFO {
    #[serde(rename = "@direction")]
    direction: FIFODirection,
    #[serde(rename = "@age")]
    age: u8,
    #[serde(rename = "@packets_transmitted")]
    #[getset(get = "pub")]
    packets_transmitted: u16,
    #[serde(skip_serializing_if = "Option::is_none", rename = "@packet_index")]
    packet_index: Option<u8>,
    #[serde(rename = "@status")]
    status: FIFOStatus,
    #[serde(rename = "@bandwidth")]
    #[getset(get = "pub")]
    bandwidth: u16,
}

impl FIFO {
    pub fn new(
        direction: FIFODirection,
        age: u8,
        packets_transmitted: u16,
        packet_index: Option<u8>,
        status: FIFOStatus,
        bandwidth: u16,
    ) -> Self {
        Self {
            direction,
            age,
            packets_transmitted,
            packet_index,
            status,
            bandwidth,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Getters)]
pub struct FIFOs {
    #[serde(
        rename = "FIFO",
        deserialize_with = "FIFOs::deserialize_fifos",
        serialize_with = "FIFOs::serialize_fifos"
    )]
    #[getset(get = "pub")]
    fifo: BTreeMap<FIFODirection, FIFO>,
}

impl FIFOs {
    pub fn new(fifo: BTreeMap<FIFODirection, FIFO>) -> Self {
        Self { fifo }
    }

    fn deserialize_fifos<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<BTreeMap<FIFODirection, FIFO>, D::Error> {
        let fifo_vec: Vec<FIFO> = Deserialize::deserialize(deserializer)?;

        let mut ret = BTreeMap::new();

        for fifo in fifo_vec {
            ret.insert(fifo.direction, fifo);
        }

        Ok(ret)
    }

    fn serialize_fifos<S: Serializer>(
        fifo: &BTreeMap<FIFODirection, FIFO>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(fifo.values())
    }
}
