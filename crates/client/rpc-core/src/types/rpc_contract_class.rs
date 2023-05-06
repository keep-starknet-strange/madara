use serde::{Deserialize, Serialize};

use super::{DeprecatedContractClass, SierraContractClass};

/// Starknet contract class
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum RPCContractClass {
    DeprecatedContractClass(DeprecatedContractClass),
    ContractClass(SierraContractClass),
}
