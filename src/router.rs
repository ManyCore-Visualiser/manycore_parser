use std::collections::BTreeMap;

use getset::Setters;
use serde::{Deserialize, Serialize};

use crate::{utils, WithXMLAttributes};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Setters)]
pub struct Router {
    #[serde(skip)]
    #[getset(set = "pub")]
    id: u8,
    #[serde(
        flatten,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "utils::deserialize_attrs"
    )]
    other_attributes: Option<BTreeMap<String, String>>,
}

impl Router {
    pub fn new(id: u8, other_attributes: Option<BTreeMap<String, String>>) -> Self {
        Self {
            id,
            other_attributes,
        }
    }
}

impl WithXMLAttributes for Router {
    fn id(&self) -> &u8 {
        &self.id
    }

    fn other_attributes(&self) -> &Option<BTreeMap<String, String>> {
        &self.other_attributes
    }
}
