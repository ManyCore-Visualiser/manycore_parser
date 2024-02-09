use std::{
    collections::HashMap,
    str::FromStr,
};

use serde::{de, Deserialize, Deserializer};

/// quick-xml deserialization fails when flattening. Instead of
/// correctly deserializing the data, a map is returned.
/// 
/// This function "intercepts" the map and parses back to the desired type.
/// I'm confident this is disgusting to someone with more rust experience,
/// as I'm just kinda hoping a map can actually be parsed rather than doing
/// something smarter. Unfortunattely, I can't figure out what the smarter
/// thing to do is here (yet, hopefully)
pub fn from_map_entry<'de, D: Deserializer<'de>, T: FromStr>(
    deserializer: D,
) -> Result<T, D::Error> {
    let map: HashMap<String, String> = Deserialize::deserialize(deserializer)?;
    
    // The map contains the attributes of an XML row to my understanding.
    // The $text key is quick-xml way of making the body of a tag serde compliant.
    match map.get("$text") {
        Some(value) => match value.parse() {
            Ok(u) => Ok(u),
            Err(_) => Err(de::Error::custom("Could not parse from String to desired type. Could not deserialize.")),
        },
        None => Err(de::Error::custom("The provided map pair has no $text entry. Could not deserialize.")),
    }
}
