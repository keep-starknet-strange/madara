use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use blockifier::execution::contract_class::{
    ContractClass as BlockifierContractClass, ContractClassV0, ContractClassV0Inner, ContractClassV1,
};
use cairo_lang_casm_contract_class::{CasmContractClass, CasmContractEntryPoint, CasmContractEntryPoints};
use cairo_lang_starknet::contract_class::{
    ContractClass as SierraContractClass, ContractEntryPoint, ContractEntryPoints,
};
use cairo_lang_starknet::contract_class_into_casm_contract_class::StarknetSierraCompilationError;
use cairo_lang_utils::bigint::BigUintAsHex;
use cairo_vm::types::program::Program;
use flate2::read::GzDecoder;
use mp_digest_log::find_starknet_block;
use mp_starknet::block::Block as StarknetBlock;
use mp_starknet::execution::types::Felt252Wrapper;
use mp_starknet::transaction::types::{
    BroadcastedTransactionConversionErrorWrapper, DeclareTransaction, DeployAccountTransaction, InvokeTransaction,
    Transaction,
};
use num_bigint::{BigInt, BigUint, Sign};
use sp_api::{BlockT, HeaderT};
use sp_blockchain::HeaderBackend;
use starknet_api::api_core::EntryPointSelector;
use starknet_api::deprecated_contract_class::{EntryPoint, EntryPointOffset, EntryPointType};
use starknet_api::hash::StarkFelt;
use starknet_core::types::contract::legacy::{
    LegacyContractClass, LegacyEntrypointOffset, RawLegacyEntryPoint, RawLegacyEntryPoints,
};
use starknet_core::types::contract::{CompiledClass, CompiledClassEntrypoint, CompiledClassEntrypointList};
use starknet_core::types::{
    BroadcastedDeclareTransaction, BroadcastedTransaction, CompressedLegacyContractClass, ContractClass,
    EntryPointsByType, FieldElement, FlattenedSierraClass, FromByteArrayError, LegacyContractEntryPoint,
    LegacyEntryPointsByType, SierraEntryPoint,
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

/// Converts a broadcasted transaction to a transaction
/// Supports `Invoke`, `Declare` and `DeployAccount` transactions
///
/// # Arguments
///
/// * `request` - The broadcasted transaction to convert
///
/// # Returns
///
/// * `Transaction` - The converted transaction
pub fn to_tx(
    request: BroadcastedTransaction,
    chain_id: Felt252Wrapper,
) -> Result<Transaction, BroadcastedTransactionConversionErrorWrapper> {
    match request {
        BroadcastedTransaction::Invoke(invoke_tx) => {
            InvokeTransaction::try_from(invoke_tx).map(|inner| inner.from_invoke(chain_id))
        }
        BroadcastedTransaction::Declare(declare_tx) => {
            to_declare_transaction(declare_tx).map(|inner| inner.from_declare(chain_id))
        }
        BroadcastedTransaction::DeployAccount(deploy_account_tx) => {
            DeployAccountTransaction::try_from(deploy_account_tx).and_then(|inner| {
                inner
                    .from_deploy(chain_id)
                    .map_err(BroadcastedTransactionConversionErrorWrapper::TransactionConversionError)
            })
        }
    }
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
pub fn get_block_by_block_hash<B, C>(client: &C, block_hash: <B as BlockT>::Hash) -> Option<StarknetBlock>
where
    B: BlockT,
    C: HeaderBackend<B>,
{
    let header = client.header(block_hash).ok().flatten()?;
    let digest = header.digest();
    let block = find_starknet_block(digest).ok()?;
    Some(block)
}

// This code was previously inside primitives/starknet/src/transaction/types.rs
// However, for V2 version we need to compile Sierra into Casm and we need to
// import cairo-lang-starknet which currently doesn't support no_std.
// So we moved this code to rpc-core/src/utils.rs
pub fn to_declare_transaction(
    tx: BroadcastedDeclareTransaction,
) -> Result<DeclareTransaction, BroadcastedTransactionConversionErrorWrapper> {
    match tx {
        BroadcastedDeclareTransaction::V1(declare_tx_v1) => {
            let signature = declare_tx_v1
                .signature
                .iter()
                .map(|f| (*f).into())
                .collect::<Vec<Felt252Wrapper>>()
                .try_into()
                .map_err(|_| BroadcastedTransactionConversionErrorWrapper::SignatureBoundError)?;

            // Create a GzipDecoder to decompress the bytes
            let mut gz = GzDecoder::new(&declare_tx_v1.contract_class.program[..]);

            // Read the decompressed bytes into a Vec<u8>
            let mut decompressed_bytes = Vec::new();
            std::io::Read::read_to_end(&mut gz, &mut decompressed_bytes)
                .map_err(|_| BroadcastedTransactionConversionErrorWrapper::ContractClassProgramDecompressionError)?;

            // Deserialize it then
            let program: Program = Program::from_bytes(&decompressed_bytes, None)
                .map_err(|_| BroadcastedTransactionConversionErrorWrapper::ContractClassProgramDeserializationError)?;
            let legacy_contract_class = LegacyContractClass {
                program: serde_json::from_slice(decompressed_bytes.as_slice())
                    .map_err(|_| BroadcastedTransactionConversionErrorWrapper::ProgramConversionError)?,
                abi: match declare_tx_v1.contract_class.abi.as_ref() {
                    Some(abi) => abi.iter().cloned().map(|entry| entry.into()).collect::<Vec<_>>(),
                    None => vec![],
                },
                entry_points_by_type: to_raw_legacy_entry_points(
                    declare_tx_v1.contract_class.entry_points_by_type.clone(),
                ),
            };
            let mut entry_points_by_type = <HashMap<EntryPointType, Vec<EntryPoint>>>::new();
            entry_points_by_type.insert(
                EntryPointType::Constructor,
                declare_tx_v1
                    .contract_class
                    .entry_points_by_type
                    .constructor
                    .iter()
                    .map(|entry_point| -> EntryPoint {
                        EntryPoint {
                            selector: EntryPointSelector(StarkFelt(entry_point.selector.to_bytes_be())),
                            offset: EntryPointOffset(entry_point.offset as usize),
                        }
                    })
                    .collect::<Vec<EntryPoint>>(),
            );
            entry_points_by_type.insert(
                EntryPointType::External,
                declare_tx_v1
                    .contract_class
                    .entry_points_by_type
                    .external
                    .iter()
                    .map(|entry_point| -> EntryPoint {
                        EntryPoint {
                            selector: EntryPointSelector(StarkFelt(entry_point.selector.to_bytes_be())),
                            offset: EntryPointOffset(entry_point.offset as usize),
                        }
                    })
                    .collect::<Vec<EntryPoint>>(),
            );
            entry_points_by_type.insert(
                EntryPointType::L1Handler,
                declare_tx_v1
                    .contract_class
                    .entry_points_by_type
                    .l1_handler
                    .iter()
                    .map(|entry_point| -> EntryPoint {
                        EntryPoint {
                            selector: EntryPointSelector(StarkFelt(entry_point.selector.to_bytes_be())),
                            offset: EntryPointOffset(entry_point.offset as usize),
                        }
                    })
                    .collect::<Vec<EntryPoint>>(),
            );
            Ok(DeclareTransaction {
                version: 1_u8,
                sender_address: declare_tx_v1.sender_address.into(),
                nonce: Felt252Wrapper::from(declare_tx_v1.nonce),
                max_fee: Felt252Wrapper::from(declare_tx_v1.max_fee),
                signature,
                contract_class: BlockifierContractClass::V0(ContractClassV0(Arc::new(ContractClassV0Inner {
                    program,
                    entry_points_by_type,
                }))),
                class_hash: legacy_contract_class.class_hash()?.into(),
                compiled_class_hash: None,
                is_query: declare_tx_v1.is_query,
            })
        }
        BroadcastedDeclareTransaction::V2(declare_tx_v2) => {
            let signature = declare_tx_v2
                .signature
                .iter()
                .map(|f| (*f).into())
                .collect::<Vec<Felt252Wrapper>>()
                .try_into()
                .map_err(|_| BroadcastedTransactionConversionErrorWrapper::SignatureBoundError)?;

            let casm_constract_class = flattened_sierra_to_casm_contract_class(declare_tx_v2.contract_class.clone())
                .map_err(|_| BroadcastedTransactionConversionErrorWrapper::SierraCompilationError)?;
            let contract_class = ContractClassV1::try_from(casm_constract_class.clone())
                .map_err(|_| BroadcastedTransactionConversionErrorWrapper::CasmContractClassConversionError)?;

            // ensuring that the user has signed the correct class hash
            if get_casm_cotract_class_hash(&casm_constract_class) != declare_tx_v2.compiled_class_hash {
                return Err(BroadcastedTransactionConversionErrorWrapper::CompiledClassHashError);
            }

            Ok(DeclareTransaction {
                version: 2_u8,
                sender_address: declare_tx_v2.sender_address.into(),
                nonce: Felt252Wrapper::from(declare_tx_v2.nonce),
                max_fee: Felt252Wrapper::from(declare_tx_v2.max_fee),
                signature,
                contract_class: BlockifierContractClass::V1(contract_class),
                compiled_class_hash: Some(Felt252Wrapper::from(declare_tx_v2.compiled_class_hash)),
                class_hash: declare_tx_v2.contract_class.class_hash().into(),
                is_query: declare_tx_v2.is_query,
            })
        }
    }
}

fn to_raw_legacy_entry_point(entry_point: LegacyContractEntryPoint) -> RawLegacyEntryPoint {
    RawLegacyEntryPoint { offset: LegacyEntrypointOffset::U64AsInt(entry_point.offset), selector: entry_point.selector }
}

fn to_raw_legacy_entry_points(entry_points: LegacyEntryPointsByType) -> RawLegacyEntryPoints {
    RawLegacyEntryPoints {
        constructor: entry_points.constructor.into_iter().map(to_raw_legacy_entry_point).collect(),
        external: entry_points.external.into_iter().map(to_raw_legacy_entry_point).collect(),
        l1_handler: entry_points.l1_handler.into_iter().map(to_raw_legacy_entry_point).collect(),
    }
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

/// Converts a [FieldElement] to a [BigUint]
fn field_element_to_big_uint(value: &FieldElement) -> BigUint {
    BigInt::from_bytes_be(Sign::Plus, &value.to_bytes_be()).to_biguint().unwrap()
}

/// Converts a [FieldElement] to a [BigUintAsHex]
fn field_element_to_big_uint_as_hex(value: &FieldElement) -> BigUintAsHex {
    BigUintAsHex { value: field_element_to_big_uint(value) }
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
