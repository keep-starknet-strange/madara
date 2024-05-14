use std::sync::Arc;

use anyhow::Result;
use blockifier::state::cached_state::CommitmentStateDiff;
use cairo_lang_starknet_classes::casm_contract_class::{
    CasmContractClass, CasmContractEntryPoint, CasmContractEntryPoints, StarknetSierraCompilationError,
};
use cairo_lang_starknet_classes::contract_class::{
    ContractClass as SierraContractClass, ContractEntryPoint, ContractEntryPoints,
};
use cairo_lang_utils::bigint::BigUintAsHex;
use indexmap::IndexMap;
use mp_block::Block as StarknetBlock;
use mp_digest_log::find_starknet_block;
use mp_felt::Felt252Wrapper;
use num_bigint::{BigInt, BigUint, Sign};
use sp_api::{BlockT, HeaderT};
use sp_blockchain::HeaderBackend;
use starknet_api::state::ThinStateDiff;
use starknet_core::types::contract::{CompiledClass, CompiledClassEntrypoint, CompiledClassEntrypointList};
use starknet_core::types::{
    ContractStorageDiffItem, DeclaredClassItem, DeployedContractItem, EntryPointsByType, FieldElement,
    FlattenedSierraClass, NonceUpdate, ReplacedClassItem, SierraEntryPoint, StateDiff, StorageEntry,
};

/// Returns a [`StateDiff`] from a [`CommitmentStateDiff`]
pub fn blockifier_to_rpc_state_diff_types(commitment_state_diff: CommitmentStateDiff) -> Result<StateDiff> {
    let storage_diffs: Vec<ContractStorageDiffItem> = commitment_state_diff
        .storage_updates
        .into_iter()
        .map(|(address, storage_map)| {
            let storage_entries = storage_map
                .into_iter()
                .map(|(key, value)| StorageEntry {
                    key: Felt252Wrapper::from(key).into(),
                    value: Felt252Wrapper::from(value).into(),
                })
                .collect();
            ContractStorageDiffItem { address: Felt252Wrapper::from(address).into(), storage_entries }
        })
        .collect();

    let declared_classes = commitment_state_diff
        .class_hash_to_compiled_class_hash
        .into_iter()
        .map(|(class_hash, compiled_class_hash)| DeclaredClassItem {
            class_hash: Felt252Wrapper::from(class_hash).into(),
            compiled_class_hash: Felt252Wrapper::from(compiled_class_hash).into(),
        })
        .collect();

    let deployed_contracts = commitment_state_diff
        .address_to_class_hash
        .into_iter()
        .map(|(address, class_hash)| DeployedContractItem {
            address: Felt252Wrapper::from(address).into(),
            class_hash: Felt252Wrapper::from(class_hash).into(),
        })
        .collect();

    let nonces = commitment_state_diff
        .address_to_nonce
        .into_iter()
        .map(|(address, nonce)| NonceUpdate {
            contract_address: Felt252Wrapper::from(address).into(),
            nonce: Felt252Wrapper::from(nonce).into(),
        })
        .collect();

    Ok(StateDiff {
        storage_diffs,
        deprecated_declared_classes: vec![],
        declared_classes,
        deployed_contracts,
        replaced_classes: vec![],
        nonces,
    })
}

/// Returns a [`StateDiff`] from a [`ThinStateDiff`]
pub fn to_rpc_state_diff(thin_state_diff: ThinStateDiff) -> StateDiff {
    let nonces = thin_state_diff
        .nonces
        .into_iter()
        .map(|(contract_address, nonce)| NonceUpdate {
            contract_address: Felt252Wrapper::from(contract_address).into(),
            nonce: Felt252Wrapper::from(nonce).into(),
        })
        .collect();

    let storage_diffs = thin_state_diff
        .storage_diffs
        .into_iter()
        .map(|(contract_address, storage_changes)| ContractStorageDiffItem {
            address: Felt252Wrapper::from(contract_address).into(),
            storage_entries: storage_changes
                .into_iter()
                .map(|(storage_key, value)| StorageEntry {
                    key: Felt252Wrapper::from(storage_key).into(),
                    value: Felt252Wrapper::from(value).into(),
                })
                .collect(),
        })
        .collect();

    let deprecated_declared_classes = thin_state_diff
        .deprecated_declared_classes
        .into_iter()
        .map(|class_hash| Felt252Wrapper::from(class_hash).into())
        .collect();

    let declared_classes = thin_state_diff
        .declared_classes
        .into_iter()
        .map(|(class_hash, compiled_class_hash)| DeclaredClassItem {
            class_hash: Felt252Wrapper::from(class_hash).into(),
            compiled_class_hash: Felt252Wrapper::from(compiled_class_hash).into(),
        })
        .collect();

    let deployed_contracts = thin_state_diff
        .deployed_contracts
        .into_iter()
        .map(|(contract_address, class_hash)| DeployedContractItem {
            address: Felt252Wrapper::from(contract_address).into(),
            class_hash: Felt252Wrapper::from(class_hash).into(),
        })
        .collect();

    let replaced_classes = thin_state_diff
        .replaced_classes
        .into_iter()
        .map(|(contract_address, class_hash)| ReplacedClassItem {
            contract_address: Felt252Wrapper::from(contract_address).into(),
            class_hash: Felt252Wrapper::from(class_hash).into(),
        })
        .collect();

    StateDiff {
        nonces,
        storage_diffs,
        deprecated_declared_classes,
        declared_classes,
        deployed_contracts,
        replaced_classes,
    }
}

/// Returns the current Starknet block from the block header's digest
pub fn get_block_by_block_hash<B, C>(client: &C, block_hash: <B as BlockT>::Hash) -> Result<StarknetBlock>
where
    B: BlockT,
    C: HeaderBackend<B>,
{
    let header =
        client.header(block_hash).ok().flatten().ok_or_else(|| anyhow::Error::msg("Failed to retrieve header"))?;
    let digest = header.digest();
    let block = find_starknet_block(digest)?;
    Ok(block)
}

// Utils to convert Flattened Sierra to Casm Contract Class

/// Converts a [FlattenedSierraClass] to a [CasmContractClass]
pub fn flattened_sierra_to_casm_contract_class(
    flattened_sierra: Arc<FlattenedSierraClass>,
) -> Result<CasmContractClass, StarknetSierraCompilationError> {
    let sierra_contract_class = SierraContractClass {
        sierra_program: flattened_sierra.sierra_program.iter().map(field_element_to_big_uint_as_hex).collect(),
        sierra_program_debug_info: None,
        contract_class_version: flattened_sierra.contract_class_version.clone(),
        entry_points_by_type: entry_points_by_type_to_contract_entry_points(
            flattened_sierra.entry_points_by_type.clone(),
        ),
        abi: None, // we can convert the ABI but for now, to convert to Casm, the ABI isn't needed
    };
    let casm_contract_class = CasmContractClass::from_contract_class(sierra_contract_class, false, usize::MAX)?;
    Ok(casm_contract_class)
}

pub fn flattened_sierra_to_sierra_contract_class(
    flattened_sierra: Arc<FlattenedSierraClass>,
) -> starknet_api::state::ContractClass {
    let mut entry_points_by_type =
        IndexMap::<starknet_api::state::EntryPointType, Vec<starknet_api::state::EntryPoint>>::with_capacity(3);
    for sierra_entrypoint in flattened_sierra.entry_points_by_type.constructor.iter() {
        entry_points_by_type
            .entry(starknet_api::state::EntryPointType::Constructor)
            .or_default()
            .push(rpc_entry_point_to_starknet_api_entry_point(sierra_entrypoint));
    }
    for sierra_entrypoint in flattened_sierra.entry_points_by_type.external.iter() {
        entry_points_by_type
            .entry(starknet_api::state::EntryPointType::External)
            .or_default()
            .push(rpc_entry_point_to_starknet_api_entry_point(sierra_entrypoint));
    }
    for sierra_entrypoint in flattened_sierra.entry_points_by_type.l1_handler.iter() {
        entry_points_by_type
            .entry(starknet_api::state::EntryPointType::L1Handler)
            .or_default()
            .push(rpc_entry_point_to_starknet_api_entry_point(sierra_entrypoint));
    }
    starknet_api::state::ContractClass {
        sierra_program: flattened_sierra.sierra_program.iter().map(|f| Felt252Wrapper(*f).into()).collect(),
        entry_points_by_type,
        abi: flattened_sierra.abi.clone(),
    }
}

/// Converts a [FieldElement] to a [BigUint]
fn field_element_to_big_uint(value: &FieldElement) -> BigUint {
    BigInt::from_bytes_be(Sign::Plus, &value.to_bytes_be()).to_biguint().unwrap()
}

/// Converts a [FieldElement] to a [BigUintAsHex]
fn field_element_to_big_uint_as_hex(value: &FieldElement) -> BigUintAsHex {
    BigUintAsHex { value: field_element_to_big_uint(value) }
}

fn rpc_entry_point_to_starknet_api_entry_point(value: &SierraEntryPoint) -> starknet_api::state::EntryPoint {
    starknet_api::state::EntryPoint {
        function_idx: starknet_api::state::FunctionIndex(value.function_idx),
        selector: Felt252Wrapper(value.selector).into(),
    }
}

/// Converts a [EntryPointsByType] to a [ContractEntryPoints]
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

// Utils to convert Casm contract class to Compiled class
pub fn get_casm_cotract_class_hash(casm_contract_class: &CasmContractClass) -> FieldElement {
    // Let's not expose it as it don't produce a full fleshed CompiledClass
    // and is therefore only usefull in the context of computing the ClassHash
    fn casm_contract_class_to_partial_compiled_class(casm_contract_class: &CasmContractClass) -> CompiledClass {
        CompiledClass {
            prime: casm_contract_class.prime.to_string(),
            compiler_version: casm_contract_class.compiler_version.clone(),
            bytecode: casm_contract_class.bytecode.iter().map(|x| biguint_to_field_element(&x.value)).collect(),
            entry_points_by_type: casm_entry_points_to_compiled_entry_points(&casm_contract_class.entry_points_by_type),
            // The following fields are not usefull to compute the class hash, so no need to fill those
            hints: vec![],
            pythonic_hints: None,
            bytecode_segment_lengths: vec![],
        }
    }

    let compiled_class = casm_contract_class_to_partial_compiled_class(casm_contract_class);
    compiled_class.class_hash().unwrap()
}

/// Converts a [CasmContractEntryPoints] to a [CompiledClassEntrypointList]
fn casm_entry_points_to_compiled_entry_points(value: &CasmContractEntryPoints) -> CompiledClassEntrypointList {
    CompiledClassEntrypointList {
        external: value.external.iter().map(casm_entry_point_to_compiled_entry_point).collect(),
        l1_handler: value.l1_handler.iter().map(casm_entry_point_to_compiled_entry_point).collect(),
        constructor: value.constructor.iter().map(casm_entry_point_to_compiled_entry_point).collect(),
    }
}

/// Converts a [CasmContractEntryPoint] to a [CompiledClassEntrypoint]
fn casm_entry_point_to_compiled_entry_point(value: &CasmContractEntryPoint) -> CompiledClassEntrypoint {
    CompiledClassEntrypoint {
        selector: biguint_to_field_element(&value.selector),
        offset: value.offset.try_into().unwrap(),
        builtins: value.builtins.clone(),
    }
}

/// Converts a [BigUint] to a [FieldElement]
fn biguint_to_field_element(value: &BigUint) -> FieldElement {
    let bytes = value.to_bytes_be();
    FieldElement::from_byte_slice_be(bytes.as_slice()).unwrap()
}
