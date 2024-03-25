use crate::{channels::Channels, router::*, utils, WithXMLAttributes};
use getset::{Getters, MutGetters, Setters};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, hash::Hash};

/// A system's core.
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
    #[serde(
        rename = "Channels",
        skip_serializing_if = "Core::skip_serializing_channels"
    )]
    channels: Option<Channels>,
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
    /// Instantiates a new core.
    pub fn new(
        id: u8,
        router: Router,
        allocated_task: Option<u16>,
        channels: Option<Channels>,
        other_attributes: Option<BTreeMap<String, String>>,
    ) -> Self {
        Self {
            id,
            router,
            allocated_task,
            channels,
            other_attributes,
        }
    }

    /// Helper function to determine whether to serialise channels or not.
    fn skip_serializing_channels(channels: &Option<Channels>) -> bool {
        !channels.as_ref().is_some_and(|c| !c.channel().is_empty())
    }
}

impl Hash for Core {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // We can take a shortcut here as IDs are unique for our cores
        self.id.hash(state);
    }
}

impl WithXMLAttributes for Core {
    fn id(&self) -> &u8 {
        &self.id
    }

    fn other_attributes(&self) -> &Option<BTreeMap<String, String>> {
        &self.other_attributes
    }

    fn variant(&self) -> &'static str {
        &"c"
    }
}

/// List of cores in the system.
#[derive(Serialize, Deserialize, Debug, PartialEq, Getters, Setters, MutGetters, Clone)]
#[getset(get = "pub", set = "pub", get_mut = "pub")]
pub struct Cores {
    #[serde(rename = "Core")]
    list: Vec<Core>,
}

impl Cores {
    /// Instantiates a new Cores instance.
    pub fn new(list: Vec<Core>) -> Self {
        Self { list }
    }
}
