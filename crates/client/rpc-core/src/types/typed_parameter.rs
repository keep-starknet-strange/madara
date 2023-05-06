use serde::{Deserialize, Serialize};

pub type ParameterName = String;
pub type ParameterType = String;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct TypedParameter {
    pub name: ParameterName,
    #[serde(rename = "type")]
    pub _type: ParameterType,
}
