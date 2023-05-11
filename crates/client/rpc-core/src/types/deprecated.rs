use hex::{FromHex, ToHex};
use mp_starknet::execution::types::EntryPointWrapper;
use serde::{Deserialize, Serialize};

use super::{ContractABI, Offset, Program, Selector};
use crate::utils::{add_prefix, remove_leading_zeros};

pub type DeprecatedConstructor = Vec<DeprecatedCairoEntryPoint>;
pub type DeprecatedExternal = Vec<DeprecatedCairoEntryPoint>;
pub type DeprecatedL1Handler = Vec<DeprecatedCairoEntryPoint>;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct DeprecatedEntryPointsByType {
    #[serde(rename = "CONSTRUCTOR")]
    pub constructor: DeprecatedConstructor,
    #[serde(rename = "EXTERNAL")]
    pub external: DeprecatedExternal,
    #[serde(rename = "L1_HANDLER")]
    pub l_1_handler: DeprecatedL1Handler,
}

/// Deprecated Cairo contract class (pre Sierra)
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct DeprecatedContractClass {
    pub program: Program,
    pub entry_points_by_type: DeprecatedEntryPointsByType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abi: Option<ContractABI>,
}

/// Deprecated Cairo entry point (pre Sierra)
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct DeprecatedCairoEntryPoint {
    pub offset: Offset,
    pub selector: Selector,
}

impl From<EntryPointWrapper> for DeprecatedCairoEntryPoint {
    fn from(value: EntryPointWrapper) -> Self {
        let selector: String = value.selector.encode_hex();
        let selector = add_prefix(&selector);
        let offset: String = value.offset.to_be_bytes().as_slice().encode_hex();
        let offset = add_prefix(remove_leading_zeros(&offset));
        Self { selector, offset }
    }
}

impl From<DeprecatedCairoEntryPoint> for EntryPointWrapper {
    fn from(value: DeprecatedCairoEntryPoint) -> Self {
        let selector = <[u8; 32]>::from_hex(format_hex(&value.selector)).unwrap();
        let offset = u128::from_str_radix(&format_hex(&value.offset), 16).unwrap();
        Self { selector, offset }
    }
}

/// Removes the "0x" prefix from a given hexadecimal string and pads it with 0s
#[inline(always)]
fn format_hex(input: &str) -> String {
    format!("{:0>64}", input.strip_prefix("0x").unwrap_or(input))
}
