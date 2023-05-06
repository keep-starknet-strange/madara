use serde::{Deserialize, Serialize};

use super::{ContractClassVersion, Selector, SierraProgram, ABI};

pub type FunctionIndex = i64;

/// Cairo sierra entry point
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct SierraEntryPoint {
    pub selector: Selector,
    pub function_idx: FunctionIndex,
}

pub type Constructor = Vec<SierraEntryPoint>;
pub type External = Vec<SierraEntryPoint>;
pub type L1Handler = Vec<SierraEntryPoint>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct EntryPointsByType {
    #[serde(rename = "CONSTRUCTOR")]
    pub constructor: Constructor,
    #[serde(rename = "EXTERNAL")]
    pub external: External,
    #[serde(rename = "L1_HANDLER")]
    pub l_1_handler: L1Handler,
}

/// Cairo sierra contract class
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct SierraContractClass {
    pub sierra_program: SierraProgram,
    pub contract_class_version: ContractClassVersion,
    pub entry_points_by_type: EntryPointsByType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abi: Option<ABI>,
}
