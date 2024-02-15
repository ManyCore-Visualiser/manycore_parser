use std::{collections::HashMap, hash::Hash};

use crate::router::*;
use getset::{Getters, MutGetters, Setters};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum CoreStatus {
    Broken,
    Low,
    Mid,
    High,
}

impl Default for CoreStatus {
    fn default() -> Self {
        CoreStatus::Broken
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct FIFOs {}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Getters, Setters, MutGetters)]
#[getset(get = "pub", set = "pub", get_mut = "pub")]
pub struct Core {
    #[serde(rename = "@id")]
    id: u8,
    // //#[serde(rename = "@age")]
    // //age: u16,
    // #[serde(rename = "@temperature")]
    // temperature: u8,
    // #[serde(rename = "@status")]
    // status: CoreStatus,
    #[serde(rename = "Router")]
    router: Router,
    #[serde(rename = "@allocated_task", skip_serializing_if = "Option::is_none")]
    allocated_task: Option<u16>,
    #[serde(rename = "FIFOs")]
    fifos: Option<FIFOs>,
    #[serde(
        flatten,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_attrs"
    )]
    other_attributes: Option<HashMap<String, String>>,
}

fn deserialize_attrs<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<HashMap<String, String>>, D::Error> {
    let map_option: Option<HashMap<String, String>> = Deserialize::deserialize(deserializer)?;

    match map_option {
        Some(mut map) => {
            map.remove("$value");
            map.remove("$text");

            if map.is_empty() {
                return Ok(None);
            }

            Ok(Some(map))
        }
        None => Ok(None),
    }
}

impl Core {
    pub fn new(
        id: u8,
        // age: u16,
        // temperature: u8,
        // status: CoreStatus,
        router: Router,
        allocated_task: Option<u16>,
        fifos: Option<FIFOs>,
        other_attributes: Option<HashMap<String, String>>,
    ) -> Self {
        Self {
            id,
            // age,
            // temperature,
            // status,
            router,
            allocated_task,
            fifos,
            other_attributes,
        }
    }
}

impl Hash for Core {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // We can take a shortcut here as IDs are unique for our cores
        self.id.hash(state);
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Getters, Setters, MutGetters)]
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
