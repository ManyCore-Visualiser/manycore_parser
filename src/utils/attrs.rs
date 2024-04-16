use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer};

/// Utility function to deserialise `other_attributes` map. It deserialises the
/// map values as a sequence after removing `$value` and `$text` entries. These
/// symbolise an XML element inner text. They should not be there in the first place
/// as per my understanding of [`quick_xml::de`]. However, better safe than sorry.
/// Sanitise regardless.
pub(crate) fn deserialize_attrs<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<BTreeMap<String, String>>, D::Error> {
    let map_option: Option<BTreeMap<String, String>> = Deserialize::deserialize(deserializer)?;

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
