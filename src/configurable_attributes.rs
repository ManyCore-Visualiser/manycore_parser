use std::collections::BTreeMap;

use getset::Getters;
use serde::Serialize;

use crate::RoutingAlgorithms;

pub trait WithXMLAttributes {
    fn other_attributes(&self) -> &Option<BTreeMap<String, String>>;
    fn variant(&self) -> &'static str;
}

pub trait WithID<T> {
    fn id(&self) -> &T;
}

#[derive(Serialize, Debug, PartialEq, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum AttributeType {
    Text,
    Number,
    Coordinates,
    Boolean,
    Routing,
}

#[derive(Serialize, PartialEq, Debug, Clone)]
pub struct ProcessedAttribute {
    #[serde(rename = "type")]
    _type: AttributeType,
    display: String,
}

impl ProcessedAttribute {
    fn format_display(key: &String) -> String {
        if key.len() == 0 {
            // TODO: Throw Error
            return "".to_string();
        }
        // Skip @ symbol
        let clean_key = key.chars().skip(1).collect::<String>();

        // Uppercase chars indices (true = uppercase, false = lowercase)
        let upper_i = clean_key
            .chars()
            .map(|c| c.is_uppercase())
            .collect::<Vec<bool>>();
        // Last iterable item
        let last = upper_i.len() - 1;

        // Previous split index
        let mut prev = 0usize;

        let mut ret = String::new();

        // Last is exclusive here because we always
        // want to be able to grab the current and next char descriptors.
        for i in 0..last {
            // Char at i descriptor
            let first = upper_i[i];
            // Following char
            let second = upper_i[i + 1];

            if first && !second && prev != i {
                // This condition is met for something like Ab.
                // Useful to catch multiple uppercase chars that form a
                // block and are then followed by another word.
                // e.g. helloCAMELCase -> hello camel case
                ret.push_str(&clean_key[prev..=(i - 1)].to_lowercase());
                ret.push(' ');
                prev = i;
            } else if !first && second {
                // This condition is met for something like aB.
                // e.g. camelCase -> camel case
                ret.push_str(&clean_key[prev..=i].to_lowercase());
                ret.push(' ');
                prev = i + 1;
            }
        }
        // Append remaining string, if any
        ret.push_str(&clean_key[prev..].to_lowercase());

        // Trim any excess space
        let mut result = ret.trim_end().to_string();

        // Uppercase first char
        result.replace_range(0..1, &result[0..1].to_uppercase());

        result
    }

    pub(crate) fn new(key: &String, _type: AttributeType) -> Self {
        Self {
            _type,
            display: Self::format_display(key),
        }
    }
}

/// A struct containing information about what customisation
/// parameters to provide the user with.
/// This will be serialised as JSON
#[derive(Serialize, Getters, Default, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConfigurableAttributes {
    core: BTreeMap<String, ProcessedAttribute>,
    router: BTreeMap<String, ProcessedAttribute>,
    algorithms: Vec<RoutingAlgorithms>,
    observed_algorithm: Option<String>,
    channel: BTreeMap<String, ProcessedAttribute>,
}

impl ConfigurableAttributes {
    pub fn new(
        core: BTreeMap<String, ProcessedAttribute>,
        router: BTreeMap<String, ProcessedAttribute>,
        observed_algorithm: Option<String>,
        algorithms: Vec<RoutingAlgorithms>,
        channel: BTreeMap<String, ProcessedAttribute>,
    ) -> Self {
        Self {
            core,
            router,
            algorithms,
            observed_algorithm,
            channel,
        }
    }
}

pub trait AttributesMap {
    fn insert_manual(&mut self, key: &str, _type: AttributeType);
    fn extend_from_element<T: WithXMLAttributes>(&mut self, element: &T);
}

impl AttributesMap for BTreeMap<String, ProcessedAttribute> {
    fn insert_manual(&mut self, key: &str, _type: AttributeType) {
        let key_string = key.to_string();
        self.insert(
            key_string.clone(),
            ProcessedAttribute::new(&key_string, _type),
        );
    }
    fn extend_from_element<T: WithXMLAttributes>(&mut self, element: &T) {
        // Are there any attributes we can inspect?
        if let Some(other_attributes) = element.other_attributes() {
            for (key, value) in other_attributes {
                // It's worth inspecting the attribute only if missing in the map.
                if !self.contains_key(key) {
                    // If parsing the attribute value as a number fails, the attribute must
                    // be a string.
                    let processed_attribute = match value.parse::<u64>() {
                        Ok(_) => ProcessedAttribute::new(key, AttributeType::Number),
                        Err(_) => ProcessedAttribute::new(key, AttributeType::Text),
                    };

                    self.insert(key.clone(), processed_attribute);
                }
            }
        }
    }
}
