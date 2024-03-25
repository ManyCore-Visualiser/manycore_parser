use std::collections::BTreeMap;

use getset::Setters;
use serde::{Deserialize, Serialize};

use crate::{utils, WithXMLAttributes};

/// A core's router.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Setters)]
pub struct Router {
    /// The associated core id (not part of XML).
    #[serde(skip)]
    #[getset(set = "pub")]
    id: u8,
    /// Any other router attribute present in the XML.
    #[serde(
        flatten,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "utils::attrs::deserialize_attrs"
    )]
    other_attributes: Option<BTreeMap<String, String>>,
}

impl Router {
    /// Instantiates a new Router.
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

    fn variant(&self) -> &'static str {
        &"r"
    }
}
