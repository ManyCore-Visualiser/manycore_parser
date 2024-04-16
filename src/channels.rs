use std::{collections::BTreeMap, fmt::Display};

use getset::{Getters, MutGetters};
use manycore_utils::{deserialize_btree_vector, serialise_btreemap, BTreeVector};
use serde::{Deserialize, Serialize};

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

/// Object representation of a `<Channel>` element as provided in XML input.
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
    #[cfg(test)]
    /// Instantiates a new [`Channel`] instance.
    pub(crate) fn new(
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

    /// Adds to the current load of a [`Channel`].
    pub(crate) fn add_to_load(&mut self, cost: u16) {
        self.current_load += cost;
    }
}

impl BTreeVector<Directions> for Channel {
    fn key(&self) -> Directions {
        self.direction
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

/// Object representation of a `<Channels>` element as provided in XML input.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Getters, MutGetters)]
pub struct Channels {
    /// A map of channels that uses direction as key and the [`Channel`] itself as value.
    #[serde(
        rename = "Channel",
        deserialize_with = "deserialize_btree_vector",
        serialize_with = "serialise_btreemap"
    )]
    #[getset(get = "pub", get_mut = "pub")]
    channel: BTreeMap<Directions, Channel>,
}

impl Channels {
    #[cfg(test)]
    /// Instantiates a new Channels instance.
    pub(crate) fn new(channel: BTreeMap<Directions, Channel>) -> Self {
        Self { channel }
    }

    /// Clears all [`Channel`] loads within the provided [`Channels`] instance.
    pub(crate) fn clear_loads(&mut self) {
        self.channel
            .iter_mut()
            .for_each(|(_, c)| c.current_load = 0);
    }

    /// Adds to the [`Channel`]'s load in the given [`Directions`] within the provided [`Channels`] instance.
    pub(crate) fn add_to_load(
        &mut self,
        cost: u16,
        direction: Directions,
    ) -> Result<(), ManycoreError> {
        self.channel
            .get_mut(&direction)
            .ok_or(ManycoreError::new(
                crate::error::ManycoreErrorKind::RoutingError(format!(
                    "Missing {} channels.",
                    direction
                )),
            ))?
            .add_to_load(u16::from(cost));

        Ok(())
    }
}
