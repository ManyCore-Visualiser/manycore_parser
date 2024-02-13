use getset::{Getters, MutGetters, Setters};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum RouterStatus {
    Broken,
    Normal,
}

impl Default for RouterStatus {
    fn default() -> Self {
        RouterStatus::Broken
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Getters, Setters, MutGetters)]
#[getset(get = "pub", set = "pub", get_mut = "pub")]
pub struct Router {
    #[serde(rename = "@age")]
    age: u8,
    #[serde(rename = "@temperature")]
    temperature: u8,
    #[serde(rename = "@status")]
    status: RouterStatus,
}

impl Router {
    pub fn new(age: u8, temperature: u8, status: RouterStatus) -> Self {
        Self {
            age,
            temperature,
            status,
        }
    }
}
