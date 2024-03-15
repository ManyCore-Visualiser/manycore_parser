use std::collections::BTreeMap;

use getset::Getters;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub enum Directions {
    North,
    South,
    West,
    East,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum ChannelStatus {
    Normal,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Getters)]
pub struct Channel {
    #[serde(rename = "@direction")]
    direction: Directions,
    #[serde(rename = "@age")]
    age: u8,
    #[serde(rename = "@packets_transmitted")]
    #[getset(get = "pub")]
    packets_transmitted: u16,
    #[serde(skip_serializing_if = "Option::is_none", rename = "@packet_index")]
    packet_index: Option<u8>,
    #[serde(rename = "@status")]
    status: ChannelStatus,
    #[serde(rename = "@bandwidth")]
    #[getset(get = "pub")]
    bandwidth: u16,
}

impl Channel {
    pub fn new(
        direction: Directions,
        age: u8,
        packets_transmitted: u16,
        packet_index: Option<u8>,
        status: ChannelStatus,
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
pub struct Channels {
    #[serde(
        rename = "Channel",
        deserialize_with = "Channels::deserialize_channels",
        serialize_with = "Channels::serialize_channels"
    )]
    #[getset(get = "pub")]
    channel: BTreeMap<Directions, Channel>,
}

impl Channels {
    pub fn new(channel: BTreeMap<Directions, Channel>) -> Self {
        Self { channel }
    }

    fn deserialize_channels<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<BTreeMap<Directions, Channel>, D::Error> {
        let channel_vec: Vec<Channel> = Deserialize::deserialize(deserializer)?;

        let mut ret = BTreeMap::new();

        for channel in channel_vec {
            ret.insert(channel.direction, channel);
        }

        Ok(ret)
    }

    fn serialize_channels<S: Serializer>(
        channel: &BTreeMap<Directions, Channel>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(channel.values())
    }
}
