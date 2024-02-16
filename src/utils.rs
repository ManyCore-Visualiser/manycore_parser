use std::collections::HashMap;

use serde::{Deserialize, Deserializer};

pub fn deserialize_attrs<'de, D: Deserializer<'de>>(
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
