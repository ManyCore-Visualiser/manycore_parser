use crate::{channels::Channels, utils, AttributeType, Router, WithXMLAttributes};
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
    #[serde(rename = "Channels")]
    channels: Channels,
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
        channels: Channels,
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

    // fn calculate_btree_key(&self) -> u32 {
    //     let mut key = match self.allocated_task() {
    //         Some(task_id) => u32::from(*task_id),
    //         None => 0,
    //     };

    //     let shifted_id = u32::from(self.id) << 16;
    //     key += shifted_id;

    //     key
    // }
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

#[derive(Debug, PartialEq, Clone, Getters, Setters)]
#[getset(get = "pub", set = "pub", get_mut = "pub")]
pub struct CoreMap {
    map: BTreeMap<u8, Core>,
    core_attributes: BTreeMap<String, AttributeType>,
    router_attributes: BTreeMap<String, AttributeType>,
    task_core_map: BTreeMap<u16, u8>,
}

impl CoreMap {
    /// Retrieves all available attributes and their type, and inserts them in the given map.
    fn populate_attribute_map<T: WithXMLAttributes>(
        item: &T,
        map: &mut BTreeMap<String, AttributeType>,
    ) {
        // Are there any attributes we can inspect?
        if let Some(other_attributes) = item.other_attributes() {
            for (key, value) in other_attributes {
                // It's worth inspecting the attribute only if missing in the map.
                if !map.contains_key(key) {
                    // If parsing the attribute value as a number fails, the attribute must
                    // be a string.
                    let attribute_type = match value.parse::<u64>() {
                        Ok(_) => AttributeType::Number,
                        Err(_) => AttributeType::Text,
                    };

                    map.insert(key.clone(), attribute_type);
                }
            }
        }
    }

    /// Gets a core by id. TODO: DO NOT USE UNWRAP
    pub fn core_by_id(&self, id: String) -> &mut Core {
        let id = id.parse::<u8>().unwrap();

        self.map.get_mut(&id).unwrap()
    }

    /// Gets a core by task id
    pub fn core_by_task_id(&self, task_id: &u16) -> Option<&mut Core> {
        if let Some(core_id) = self.task_core_map.get(task_id) {
            return self.map.get_mut(core_id);
        }

        None
    }

    pub fn map_mut(&mut self) -> &mut BTreeMap<u8, Core> {
        // Not sure why this does not get generated
        &mut self.map
    }
}

impl<'de> Deserialize<'de> for CoreMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // This vector is temporary and gets deallocated once deserialisation finishes
        let mut core_vec: Vec<Core> = Vec::deserialize(deserializer)?;

        let mut map = BTreeMap::new();
        let mut core_attributes = BTreeMap::new();
        let mut router_attributes = BTreeMap::new();
        let mut task_core_map = BTreeMap::new();

        for core in core_vec.iter_mut() {
            let core_id = *core.id();

            if let Some(task_id) = core.allocated_task() {
                task_core_map.insert(*task_id, core_id);
            }

            Self::populate_attribute_map(core, &mut core_attributes);
            Self::populate_attribute_map(core.router(), &mut router_attributes);

            core.router_mut().set_id(core_id);

            map.insert(core_id, core.clone());
        }

        // Manually insert core attributes that are not part of the "other_attributes" map.
        core_attributes.insert("@id".to_string(), AttributeType::Text);
        core_attributes.insert("@coordinates".to_string(), AttributeType::Text);

        Ok(Self {
            map,
            core_attributes,
            router_attributes,
            task_core_map,
        })
    }
}

impl Serialize for CoreMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_seq(self.map.values())
    }
}

/// List of cores in the system.
#[derive(Serialize, Deserialize, Debug, PartialEq, Getters, Setters, MutGetters, Clone)]
#[getset(get = "pub", set = "pub", get_mut = "pub")]
pub struct Cores {
    #[serde(rename = "Core")]
    core_map: CoreMap,
}

impl Cores {
    /// Instantiates a new Cores instance.
    pub fn new(core_map: CoreMap) -> Self {
        Self { core_map }
    }
}
