use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub type StructMember = HashMap<String, serde_json::Value>;
pub type Members = Vec<StructMember>;
pub type Size = i64;
pub type StructName = String;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Default)]
pub enum StructABIType {
    #[serde(rename = "struct")]
    #[default]
    Struct,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct StructABIEntry {
    #[serde(rename = "type")]
    pub _type: StructABIType,
    pub name: StructName,
    pub size: Size,
    pub members: Members,
}
