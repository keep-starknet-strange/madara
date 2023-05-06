use serde::{Deserialize, Serialize};

/// A tag specifying a dynamic reference to a block
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum BlockTag {
    /// The latest accepted block
    #[serde(rename = "latest")]
    Latest,
    /// The current pending block
    #[serde(rename = "pending")]
    Pending,
}
