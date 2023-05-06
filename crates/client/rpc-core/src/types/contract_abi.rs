use serde::{Deserialize, Serialize};

use super::{EventABIEntry, FunctionABIEntry, StructABIEntry};

/// A block hash, number or tag
/// TODO: fix block_tag
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum ContractABIEntry {
    FunctionABIEntry(FunctionABIEntry),
    EventABIEntry(EventABIEntry),
    StructABIEntry(StructABIEntry),
}

pub type ContractABI = Vec<ContractABIEntry>;
