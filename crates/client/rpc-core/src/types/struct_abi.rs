use serde::{Deserialize, Serialize};

use super::{Members, Size, StructName};

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
