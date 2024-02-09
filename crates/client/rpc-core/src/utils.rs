use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use blockifier::execution::contract_class::ContractClass as BlockifierContractClass;
use cairo_lang_casm_contract_class::{CasmContractClass, CasmContractEntryPoint, CasmContractEntryPoints};
use cairo_lang_starknet::contract_class::{
    ContractClass as SierraContractClass, ContractEntryPoint, ContractEntryPoints,
};
use cairo_lang_starknet::contract_class_into_casm_contract_class::StarknetSierraCompilationError;
use cairo_lang_utils::bigint::BigUintAsHex;
use indexmap::IndexMap;
use mp_block::Block as StarknetBlock;
use mp_digest_log::find_starknet_block;
use mp_felt::Felt252Wrapper;
use num_bigint::{BigInt, BigUint, Sign};
use sp_api::{BlockT, HeaderT};
use sp_blockchain::HeaderBackend;
use starknet_api::deprecated_contract_class::{EntryPoint, EntryPointType};
use starknet_api::state::ThinStateDiff;
use starknet_core::types::contract::{CompiledClass, CompiledClassEntrypoint, CompiledClassEntrypointList};
use starknet_core::types::{
    CompressedLegacyContractClass, ContractClass, ContractStorageDiffItem, DeclaredClassItem, DeployedContractItem,
    EntryPointsByType, FieldElement, FlattenedSierraClass, FromByteArrayError, LegacyContractEntryPoint,
    LegacyEntryPointsByType, NonceUpdate, ReplacedClassItem, SierraEntryPoint, StateDiff, StorageEntry,
};

/// Returns a [`ContractClass`] from a [`BlockifierContractClass`]
pub fn to_rpc_contract_class(contract_class: BlockifierContractClass) -> Result<ContractClass> {
    match contract_class {
        BlockifierContractClass::V0(contract_class) => {
            let entry_points_by_type = to_legacy_entry_points_by_type(&contract_class.entry_points_by_type)?;
            let compressed_program = compress(&contract_class.program.to_bytes())?;
            Ok(ContractClass::Legacy(CompressedLegacyContractClass {
                program: compressed_program,
                entry_points_by_type,
                // FIXME 723
                abi: None,
            }))
        }
        BlockifierContractClass::V1(_contract_class) => Ok(ContractClass::Sierra(FlattenedSierraClass {
            sierra_program: vec![], // FIXME: https://github.com/keep-starknet-strange/madara/issues/775
            contract_class_version: option_env!("COMPILER_VERSION").unwrap_or("0.11.2").into(),
            entry_points_by_type: EntryPointsByType { constructor: vec![], external: vec![], l1_handler: vec![] }, /* TODO: add entry_points_by_type */
            abi: String::from("{}"), // FIXME: https://github.com/keep-starknet-strange/madara/issues/790
        })),
    }
}

/// Returns a [`StateDiff`] from a [`ThinStateDiff`]
pub fn to_rpc_state_diff(thin_state_diff: ThinStateDiff) -> StateDiff {
    let nonces = thin_state_diff
        .nonces
        .iter()
        .map(|x| NonceUpdate { contract_address: x.0.0.0.into(), nonce: x.1.0.into() })
        .collect();

    let storage_diffs = thin_state_diff
        .storage_diffs
        .iter()
        .map(|x| ContractStorageDiffItem {
            address: x.0.0.0.into(),
            storage_entries: x.1.iter().map(|y| StorageEntry { key: y.0.0.0.into(), value: (*y.1).into() }).collect(),
        })
        .collect();

    let deprecated_declared_classes = thin_state_diff.deprecated_declared_classes.iter().map(|x| x.0.into()).collect();

    let declared_classes = thin_state_diff
        .declared_classes
        .iter()
        .map(|x| DeclaredClassItem { class_hash: x.0.0.into(), compiled_class_hash: x.1.0.into() })
        .collect();

    let deployed_contracts = thin_state_diff
        .deployed_contracts
        .iter()
        .map(|x| DeployedContractItem { address: x.0.0.0.into(), class_hash: x.1.0.into() })
        .collect();

    let replaced_classes = thin_state_diff
        .replaced_classes
        .iter()
        .map(|x| ReplacedClassItem { contract_address: x.0.0.0.into(), class_hash: x.1.0.into() })
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

/// Returns a compressed vector of bytes
pub(crate) fn compress(data: &[u8]) -> Result<Vec<u8>> {
    let mut gzip_encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    // 2023-08-22: JSON serialization is already done in Blockifier
    // https://github.com/keep-starknet-strange/blockifier/blob/no_std-support-7578442/crates/blockifier/src/execution/contract_class.rs#L129
    // https://github.com/keep-starknet-strange/blockifier/blob/no_std-support-7578442/crates/blockifier/src/execution/contract_class.rs#L389
    // serde_json::to_writer(&mut gzip_encoder, data)?;
    gzip_encoder.write_all(data)?;
    Ok(gzip_encoder.finish()?)
}

/// Returns a [Result<LegacyEntryPointsByType>] (starknet-rs type) from a [HashMap<EntryPointType,
/// Vec<EntryPoint>>]
fn to_legacy_entry_points_by_type(
    entries: &HashMap<EntryPointType, Vec<EntryPoint>>,
) -> Result<LegacyEntryPointsByType> {
    fn collect_entry_points(
        entries: &HashMap<EntryPointType, Vec<EntryPoint>>,
        entry_point_type: EntryPointType,
    ) -> Result<Vec<LegacyContractEntryPoint>> {
        Ok(entries
            .get(&entry_point_type)
            .ok_or(anyhow!("Missing {:?} entry point", entry_point_type))?
            .iter()
            .map(|e| to_legacy_entry_point(e.clone()))
            .collect::<Result<Vec<LegacyContractEntryPoint>, FromByteArrayError>>()?)
    }

    let constructor = collect_entry_points(entries, EntryPointType::Constructor)?;
    let external = collect_entry_points(entries, EntryPointType::External)?;
    let l1_handler = collect_entry_points(entries, EntryPointType::L1Handler)?;

    Ok(LegacyEntryPointsByType { constructor, external, l1_handler })
}

/// Returns a [LegacyContractEntryPoint] (starknet-rs) from a [EntryPoint] (starknet-api)
fn to_legacy_entry_point(entry_point: EntryPoint) -> Result<LegacyContractEntryPoint, FromByteArrayError> {
    let selector = FieldElement::from_bytes_be(&entry_point.selector.0.0)?;
    let offset = entry_point.offset.0 as u64;
    Ok(LegacyContractEntryPoint { selector, offset })
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
    let casm_contract_class = sierra_contract_class.into_casm_contract_class(false)?;
    Ok(casm_contract_class)
}

pub fn flattened_sierra_to_sierra_contract_class(
    flattened_sierra: Arc<FlattenedSierraClass>,
) -> starknet_api::state::ContractClass {
    let mut entry_point_by_type =
        IndexMap::<starknet_api::state::EntryPointType, Vec<starknet_api::state::EntryPoint>>::with_capacity(3);
    for sierra_entrypoint in flattened_sierra.entry_points_by_type.constructor.iter() {
        entry_point_by_type
            .entry(starknet_api::state::EntryPointType::Constructor)
            .or_default()
            .push(rpc_entry_point_to_starknet_api_entry_point(sierra_entrypoint));
    }
    for sierra_entrypoint in flattened_sierra.entry_points_by_type.external.iter() {
        entry_point_by_type
            .entry(starknet_api::state::EntryPointType::External)
            .or_default()
            .push(rpc_entry_point_to_starknet_api_entry_point(sierra_entrypoint));
    }
    for sierra_entrypoint in flattened_sierra.entry_points_by_type.l1_handler.iter() {
        entry_point_by_type
            .entry(starknet_api::state::EntryPointType::L1Handler)
            .or_default()
            .push(rpc_entry_point_to_starknet_api_entry_point(sierra_entrypoint));
    }
    starknet_api::state::ContractClass {
        sierra_program: flattened_sierra.sierra_program.iter().map(|f| Felt252Wrapper(*f).into()).collect(),
        entry_point_by_type,
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
    let compiled_class = casm_contract_class_to_compiled_class(casm_contract_class);
    compiled_class.class_hash().unwrap()
}

/// Converts a [CasmContractClass] to a [CompiledClass]
pub fn casm_contract_class_to_compiled_class(casm_contract_class: &CasmContractClass) -> CompiledClass {
    CompiledClass {
        prime: casm_contract_class.prime.to_string(),
        compiler_version: casm_contract_class.compiler_version.clone(),
        bytecode: casm_contract_class.bytecode.iter().map(|x| biguint_to_field_element(&x.value)).collect(),
        entry_points_by_type: casm_entry_points_to_compiled_entry_points(&casm_contract_class.entry_points_by_type),
        hints: vec![],        // not needed to get class hash so ignoring this
        pythonic_hints: None, // not needed to get class hash so ignoring this
    }
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
