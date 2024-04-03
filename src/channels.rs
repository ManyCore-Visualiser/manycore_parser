use std::{collections::BTreeMap, fmt::Display};

use getset::Getters;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::ManycoreError;
use crate::utils::attrs::deserialize_attrs;
use crate::{ManycoreErrorKind, WithXMLAttributes};

static NORTH: &str = "North";
static SOUTH: &str = "South";
static WEST: &str = "West";
static EAST: &str = "East";

/// An enum containing all allowed channel directions.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub enum Directions {
    North,
    South,
    West,
    East,
}

impl Display for Directions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self))
    }
}

impl From<&Directions> for String {
    fn from(direction: &Directions) -> Self {
        match direction {
            Directions::North => NORTH.into(),
            Directions::South => SOUTH.into(),
            Directions::West => WEST.into(),
            Directions::East => EAST.into(),
        }
    }
}

impl TryFrom<&str> for Directions {
    type Error = ManycoreError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            n if n == NORTH => Ok(Directions::North),
            s if s == SOUTH => Ok(Directions::South),
            w if w == WEST => Ok(Directions::West),
            e if e == EAST => Ok(Directions::East),
            _ => Err(ManycoreError::new(ManycoreErrorKind::GenerationError(
                format!("'{value}' is not a valid direction."),
            ))),
        }
    }
}

/// A channel.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Getters)]
pub struct Channel {
    /// The channel's direction.
    #[serde(rename = "@direction")]
    direction: Directions,
    /// The channel's bandwidth.
    #[serde(rename = "@bandwidth")]
    #[getset(get = "pub")]
    bandwidth: u16,
    #[serde(rename = "@actualComCost")]
    #[getset(get = "pub")]
    actual_com_cost: u16,
    /// The load on the channel
    #[serde(skip)]
    #[getset(get = "pub")]
    current_load: u16,
    /// Any other channel attribute present in the XML.
    #[serde(
        flatten,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_attrs"
    )]
    #[getset(skip)]
    other_attributes: Option<BTreeMap<String, String>>,
}

impl Channel {
    /// Instantiates a new channel.
    pub fn new(
        direction: Directions,
        actual_com_cost: u16,
        bandwidth: u16,
        other_attributes: Option<BTreeMap<String, String>>,
    ) -> Self {
        Self {
            direction,
            actual_com_cost,
            bandwidth,
            other_attributes,
            current_load: 0,
        }
    }

    pub fn add_to_cost(&mut self, cost: u16) {
        self.current_load += cost;
    }
}

impl WithXMLAttributes for Channel {
    fn other_attributes(&self) -> &Option<BTreeMap<String, String>> {
        &self.other_attributes
    }

    fn variant(&self) -> &'static str {
        "l"
    }
}

/// A router's channels map.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Getters)]
pub struct Channels {
    /// A map of channels that uses direction as key and the channel itself as value.
    #[serde(
        rename = "Channel",
        deserialize_with = "Channels::deserialize_channels",
        serialize_with = "Channels::serialize_channels"
    )]
    #[getset(get = "pub")]
    channel: BTreeMap<Directions, Channel>,
}

impl Channels {
    /// Instantiates a new Channels instance.
    pub fn new(channel: BTreeMap<Directions, Channel>) -> Self {
        Self { channel }
    }

    /// Helper function that deserialises channels vector as a BTreeMap.
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

    /// Helper function to serialise channels BTreeMap as Vector.
    fn serialize_channels<S: Serializer>(
        channel: &BTreeMap<Directions, Channel>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(channel.values())
    }

    pub fn channel_mut(&mut self) -> &mut BTreeMap<Directions, Channel> {
        &mut self.channel
    }

    pub fn clear_loads(&mut self) {
        self.channel
            .iter_mut()
            .for_each(|(_, c)| c.current_load = 0);
    }

    /// Adds to the channel's load.
    pub fn add_to_cost(&mut self, cost: u16, direction: Directions) -> Result<(), ManycoreError> {
        self.channel
            .get_mut(&direction)
            .ok_or(ManycoreError::new(
                crate::error::ManycoreErrorKind::RoutingError(format!(
                    "Missing {} channels.",
                    direction
                )),
            ))?
            .current_load += u16::from(cost);

        Ok(())
    }
}
