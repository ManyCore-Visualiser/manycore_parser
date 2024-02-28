use crate::{fifos::FIFOs, router::*, utils, WithXMLAttributes};
use getset::{Getters, MutGetters, Setters};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, hash::Hash};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Getters, Setters, MutGetters)]
#[getset(get = "pub", set = "pub", get_mut = "pub")]
pub struct Core {
    #[serde(rename = "@id")]
    #[getset(skip)]
    id: u8,
    #[serde(rename = "Router")]
    router: Router,
    #[serde(rename = "@allocated_task", skip_serializing_if = "Option::is_none")]
    allocated_task: Option<u16>,
    #[serde(rename = "FIFOs", skip_serializing_if = "Core::skip_serializing_fifos")]
    fifos: Option<FIFOs>,
    #[serde(
        flatten,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "utils::deserialize_attrs"
    )]
    #[getset(skip)]
    other_attributes: Option<BTreeMap<String, String>>,
}

impl Core {
    pub fn new(
        id: u8,
        router: Router,
        allocated_task: Option<u16>,
        fifos: Option<FIFOs>,
        other_attributes: Option<BTreeMap<String, String>>,
    ) -> Self {
        Self {
            id,
            router,
            allocated_task,
            fifos,
            other_attributes,
        }
    }

    fn skip_serializing_fifos(fifos: &Option<FIFOs>) -> bool {
        !fifos.as_ref().is_some_and(|f| !f.fifo().is_empty())
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Getters, Setters, MutGetters, Clone)]
#[getset(get = "pub", set = "pub", get_mut = "pub")]
pub struct Cores {
    #[serde(rename = "Core")]
    list: Vec<Core>,
}

impl Cores {
    pub fn new(list: Vec<Core>) -> Self {
        Self { list }
    }
}
