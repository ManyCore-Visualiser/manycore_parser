use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{utils, WithXMLAttributes};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Router {
    #[serde(
        flatten,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "utils::deserialize_attrs"
    )]
    other_attributes: Option<HashMap<String, String>>,
}

impl Router {
    pub fn new(other_attributes: Option<HashMap<String, String>>) -> Self {
        Self { other_attributes }
    }
}

impl WithXMLAttributes for Router {
    fn id(&self) -> Option<&u8> {
        None
    }

    fn other_attributes(&self) -> &Option<HashMap<String, String>> {
        &self.other_attributes
    }
}
