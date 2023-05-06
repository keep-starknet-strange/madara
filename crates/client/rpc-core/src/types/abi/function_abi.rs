use serde::{Deserialize, Serialize};

use super::TypedParameter;

pub type FunctionName = String;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Default)]
pub enum FunctionABIType {
    #[serde(rename = "function")]
    #[default]
    Function,
    #[serde(rename = "l1_handler")]
    LOneHandler,
    #[serde(rename = "constructor")]
    Constructor,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct FunctionABIEntry {
    #[serde(rename = "type")]
    pub _type: FunctionABIType,
    pub name: FunctionName,
    pub inputs: TypedParameter,
    pub outputs: TypedParameter,
}
