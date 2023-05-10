use serde::{Deserialize, Serialize};

use super::TypedParameter;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Default)]
pub enum EventABIType {
    #[serde(rename = "event")]
    #[default]
    Event,
}

pub type EventName = String;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct EventABIEntry {
    #[serde(rename = "type")]
    pub _type: EventABIType,
    pub name: EventName,
    pub keys: TypedParameter,
    pub data: TypedParameter,
}
