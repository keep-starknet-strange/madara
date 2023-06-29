use alloc::sync::Arc;
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::collections::HashMap;

use cairo_lang_casm_contract_class::CasmContractClass;
use cairo_lang_starknet::contract_class::{ContractClass, ContractEntryPoint, ContractEntryPoints};
use cairo_lang_starknet::contract_class_into_casm_contract_class::StarknetSierraCompilationError;
use cairo_lang_utils::bigint::BigUintAsHex;
use num_bigint::BigUint;
use scale_info::TypeDefPrimitive::U256;
use starknet_core::types::{EntryPointsByType, FlattenedSierraClass, SierraEntryPoint};
#[cfg(feature = "std")]
use starknet_core::types::{LegacyContractEntryPoint, LegacyEntryPointsByType};
use starknet_ff::{FieldElement, ValueOutOfRangeError};

use crate::execution::types::{EntryPointTypeWrapper, EntryPointV0Wrapper, Felt252Wrapper};

#[cfg(feature = "std")]
mod reexport_std_types {
    use std::collections::HashMap;

    use starknet_core::types::{LegacyContractEntryPoint, LegacyEntryPointsByType};

    use super::*;
    /// Returns a [HashMap<EntryPointTypeWrapper, Vec<EntryPointWrapper>>] from
    /// [LegacyEntryPointsByType]
    pub fn to_hash_map_entrypoints(
        entries: LegacyEntryPointsByType,
    ) -> HashMap<EntryPointTypeWrapper, Vec<EntryPointWrapper>> {
        let mut entry_points_by_type = HashMap::default();

        entry_points_by_type.insert(EntryPointTypeWrapper::Constructor, get_entrypoint_value(entries.constructor));
        entry_points_by_type.insert(EntryPointTypeWrapper::External, get_entrypoint_value(entries.external));
        entry_points_by_type.insert(EntryPointTypeWrapper::L1Handler, get_entrypoint_value(entries.l1_handler));
        entry_points_by_type
    }

    /// Returns a [Vec<EntryPointWrapper>] from a [Vec<LegacyContractEntryPoint>]
    fn get_entrypoint_value(entries: Vec<LegacyContractEntryPoint>) -> Vec<EntryPointWrapper> {
        entries.iter().map(|e| EntryPointWrapper::from(e.clone())).collect::<Vec<_>>()
    }
}

#[cfg(feature = "std")]
fn get_entrypoint_value(entries: Vec<LegacyContractEntryPoint>) -> Vec<EntryPointV0Wrapper> {
    entries.iter().map(|e| EntryPointV0Wrapper::from(e.clone())).collect::<Vec<_>>()
}

// Utils to convert Flattened Sierra to Casm Contract Class
fn field_element_to_big_uint(value: &FieldElement) -> BigUint {
    BigUint::from_bytes_le(value.to_bits_le().map(|d| if d { 1_u8 } else { 0_u8 }).as_ref())
}

fn field_element_to_big_uint_as_hex(value: &FieldElement) -> BigUintAsHex {
    BigUintAsHex { value: field_element_to_big_uint(value) }
}

fn entry_points_by_type_to_contract_entry_points(value: EntryPointsByType) -> ContractEntryPoints {
    fn sierra_entry_point_to_contract_entry_point(value: SierraEntryPoint) -> ContractEntryPoint {
        ContractEntryPoint {
            function_idx: value.function_idx.try_into().unwrap(),
            selector: field_element_to_big_uint(&value.selector),
        }
    }
    ContractEntryPoints {
        constructor: value.constructor.iter().map(|x| sierra_entry_point_to_contract_entry_point(x.clone())).collect(),
        external: value.external.iter().map(|x| sierra_entry_point_to_contract_entry_point(x.clone())).collect(),
        l1_handler: value.l1_handler.iter().map(|x| sierra_entry_point_to_contract_entry_point(x.clone())).collect(),
    }
}

/// Converts a [FlattenedSierraClass] to a [CasmContractClass]
pub fn flattened_sierra_to_casm_contract_class(
    flattened_sierra: Arc<FlattenedSierraClass>,
) -> Result<CasmContractClass, StarknetSierraCompilationError> {
    let sierra_contract_class = ContractClass {
        sierra_program: flattened_sierra.sierra_program.iter().map(|x| field_element_to_big_uint_as_hex(x)).collect(),
        sierra_program_debug_info: None,
        contract_class_version: flattened_sierra.contract_class_version.clone(),
        entry_points_by_type: entry_points_by_type_to_contract_entry_points(
            flattened_sierra.entry_points_by_type.clone(),
        ),
        abi: None, // we can convert the ABI but for now, to convert to Casm, the ABI isn't needed
    };
    Ok(sierra_contract_class.into_casm_contract_class(false)?)
}
