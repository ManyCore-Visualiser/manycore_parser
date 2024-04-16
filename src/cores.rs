use crate::{
    channels::Channels, router::*, routing_error, utils, Directions, ManycoreError,
    SinkSourceDirection, WithID, WithXMLAttributes,
};
use getset::{Getters, MutGetters, Setters};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, hash::Hash};

/// Describes where in the matrix edge the core is located.
/// Used to determine number of edge connections.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum EdgePosition {
    Top,
    TopLeft,
    TopRight,
    Left,
    Right,
    Bottom,
    BottomLeft,
    BottomRight,
}

impl From<&EdgePosition> for Vec<SinkSourceDirection> {
    fn from(position: &EdgePosition) -> Self {
        use SinkSourceDirection::*;
        match position {
            EdgePosition::Bottom => vec![South],
            EdgePosition::BottomLeft => vec![South, West],
            EdgePosition::BottomRight => vec![South, East],
            EdgePosition::Left => vec![West],
            EdgePosition::Right => vec![East],
            EdgePosition::Top => vec![North],
            EdgePosition::TopLeft => vec![North, West],
            EdgePosition::TopRight => vec![North, East],
        }
    }
}

/// Object representation of an XML `<Core>` element.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Getters, Setters, MutGetters)]
#[getset(get = "pub", set = "pub", get_mut = "pub")]
pub struct Core {
    /// The core id.
    #[serde(rename = "@id")]
    #[getset(skip)]
    id: u8,
    /// The router connected to the core.
    #[serde(rename = "Router")]
    router: Router,
    /// The task allocated to the core, if any.
    #[serde(rename = "@allocatedTask", skip_serializing_if = "Option::is_none")]
    allocated_task: Option<u16>,
    /// The communication channels associated with this core.
    #[serde(rename = "Channels")]
    channels: Channels,
    /// Map with core's incoming source loads.
    #[serde(skip)]
    source_loads: Option<BTreeMap<Directions, u16>>,
    #[serde(skip)]
    matrix_edge: Option<EdgePosition>,
    /// Any other core attribute present in the XML.
    #[serde(
        flatten,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "utils::attrs::deserialize_attrs"
    )]
    #[getset(skip)]
    other_attributes: Option<BTreeMap<String, String>>,
}

impl Core {
    #[cfg(test)]
    /// Instantiates a new [`Core`] instance.
    pub fn new(
        id: u8,
        columns: u8,
        rows: u8,
        router: Router,
        allocated_task: Option<u16>,
        channels: Channels,
        other_attributes: Option<BTreeMap<String, String>>,
    ) -> Self {
        Self {
            id,
            router,
            allocated_task,
            channels,
            source_loads: None,
            matrix_edge: Core::calculate_edge(id, columns, rows),
            other_attributes,
        }
    }

    /// Utility to determine if a core is on the edge, and if so where.
    fn calculate_edge(id: u8, columns: u8, rows: u8) -> Option<EdgePosition> {
        let bl_bound = (rows - 1) * columns;
        if id % columns == 0 {
            return match id {
                0 => Some(EdgePosition::TopLeft),
                bl if bl == bl_bound => Some(EdgePosition::BottomLeft),
                _ => Some(EdgePosition::Left),
            };
        } else if (id + 1) % columns == 0 {
            return match id {
                tr if tr == (columns - 1) => Some(EdgePosition::TopRight),
                br if br == ((rows * columns) - 1) => Some(EdgePosition::BottomRight),
                _ => Some(EdgePosition::Right),
            };
        } else if id < columns {
            return Some(EdgePosition::Top);
        } else if id > bl_bound {
            return Some(EdgePosition::Bottom);
        }

        None
    }

    /// Utility function to populate the matrix_edge field.
    pub(crate) fn populate_matrix_edge(&mut self, columns: u8, rows: u8) {
        self.matrix_edge = Core::calculate_edge(self.id, columns, rows);
    }

    /// Utility function to add to a source load.
    pub(crate) fn add_source_load(
        &mut self,
        load: u16,
        direction: &Directions,
    ) -> Result<(), ManycoreError> {
        if let None = self.matrix_edge {
            return Err(
                routing_error(
                    format!("Malformed TaskGraph: Attempted to add load from a Source on Core with ID {}. The Core is not on the matrix edge.", self.id)));
        }

        self.source_loads
            .get_or_insert(BTreeMap::new())
            .entry(*direction)
            .and_modify(|current_load| *current_load = current_load.saturating_add(load))
            .or_insert(load);

        Ok(())
    }

    /// Utility function to clear all source loads.
    pub(crate) fn clear_source_loads(&mut self) {
        self.source_loads.take();
    }
}

impl Hash for Core {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // We can take a shortcut here as IDs are unique for our cores
        self.id.hash(state);
    }
}

impl WithXMLAttributes for Core {
    fn other_attributes(&self) -> &Option<BTreeMap<String, String>> {
        &self.other_attributes
    }

    fn variant(&self) -> &'static str {
        &"c"
    }
}

impl WithID<u8> for Core {
    fn id(&self) -> &u8 {
        &self.id
    }
}

/// Object representation of `<Cores>` attributes in input XML.
#[derive(Serialize, Deserialize, Debug, PartialEq, Getters, Setters, MutGetters, Clone)]
#[getset(get = "pub", set = "pub", get_mut = "pub")]
pub struct Cores {
    #[serde(rename = "Core")]
    list: Vec<Core>,
}

impl Cores {
    #[cfg(test)]
    /// Instantiates a new Cores instance.
    pub fn new(list: Vec<Core>) -> Self {
        Self { list }
    }
}
