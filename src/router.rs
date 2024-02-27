use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{utils, WithXMLAttributes};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Router {
    #[serde(
        flatten,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "utils::deserialize_attrs"
    )]
    other_attributes: Option<BTreeMap<String, String>>,
}

impl Router {
    pub fn new(other_attributes: Option<BTreeMap<String, String>>) -> Self {
        Self { other_attributes }
    }
}

impl WithXMLAttributes for Router {
    fn id(&self) -> Option<&u8> {
        None
    }

    fn other_attributes(&self) -> &Option<BTreeMap<String, String>> {
        &self.other_attributes
    }
}
