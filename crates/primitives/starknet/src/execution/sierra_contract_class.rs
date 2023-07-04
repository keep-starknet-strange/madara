use scale_codec::{Decode, Encode};
use sp_std::vec::Vec;
use starknet_core::types::FlattenedSierraClass;

use crate::execution::felt252_wrapper::Felt252Wrapper;

// SierraClass is used in the runtime instead of `starknet-core::FlattenedSierraClass`.
// It should  have the exact same memory representation

/// A sierra Starknet contract class.
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq, scale_info::TypeInfo)]
pub struct SierraContractClass {
    /// The list of sierra instructions of which the program consists
    pub sierra_program: Vec<Felt252Wrapper>,
    /// The version of the contract class object. Currently, the Starknet os supports version 0.1.0
    pub contract_class_version: Vec<u8>,
    /// Entry points by type
    pub entry_points_by_type: EntryPointsByType,
    /// The class abi, as supplied by the user declaring the class
    pub abi: Vec<u8>,
}

/// The contract entrypoints, grouped by types
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq, scale_info::TypeInfo)]
pub struct EntryPointsByType {
    /// Constructor
    pub constructor: Vec<SierraEntryPoint>,
    /// External
    pub external: Vec<SierraEntryPoint>,
    /// L1 handler
    pub l1_handler: Vec<SierraEntryPoint>,
}

/// A Sierra entrypoint
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq, scale_info::TypeInfo)]
pub struct SierraEntryPoint {
    /// A unique identifier of the entry point (function) in the program
    pub selector: Felt252Wrapper,
    /// The index of the function in the program
    pub function_idx: u64,
}

// O(0) type conversion conversion

impl From<FlattenedSierraClass> for SierraContractClass {
    fn from(value: FlattenedSierraClass) -> Self {
        unsafe { core::mem::transmute(value) }
    }
}

impl From<SierraContractClass> for FlattenedSierraClass {
    fn from(value: SierraContractClass) -> Self {
        unsafe { core::mem::transmute(value) }
    }
}

#[cfg(test)]
mod tests {
    use starknet_core::types::{
        EntryPointsByType as FlattenedEntryPointsByType, FlattenedSierraClass,
        SierraEntryPoint as FlattenedSierraEntryPoint,
    };
    use starknet_crypto::FieldElement;

    use super::SierraContractClass;

    #[test]
    fn from_into_ok() {
        let original = FlattenedSierraClass {
            sierra_program: vec![FieldElement::ZERO, FieldElement::ONE, FieldElement::MAX],
            contract_class_version: "0.1.0".to_string(),
            entry_points_by_type: FlattenedEntryPointsByType {
                constructor: vec![
                    FlattenedSierraEntryPoint { selector: FieldElement::ZERO, function_idx: 0 },
                    FlattenedSierraEntryPoint { selector: FieldElement::ONE, function_idx: 1 },
                    FlattenedSierraEntryPoint { selector: FieldElement::MAX, function_idx: u64::MAX },
                ],
                external: vec![
                    FlattenedSierraEntryPoint { selector: FieldElement::MAX, function_idx: u64::MAX },
                    FlattenedSierraEntryPoint { selector: FieldElement::ZERO, function_idx: 0 },
                    FlattenedSierraEntryPoint { selector: FieldElement::ONE, function_idx: 1 },
                ],
                l1_handler: vec![
                    FlattenedSierraEntryPoint { selector: FieldElement::ONE, function_idx: 1 },
                    FlattenedSierraEntryPoint { selector: FieldElement::MAX, function_idx: u64::MAX },
                    FlattenedSierraEntryPoint { selector: FieldElement::ZERO, function_idx: 0 },
                ],
            },
            abi: "some_abi".to_string(),
        };

        let primitive_type: SierraContractClass = original.into();

        let cairo_rs_type: FlattenedSierraClass = primitive_type.clone().into();

        assert_eq!(primitive_type, cairo_rs_type.into());
    }
}
