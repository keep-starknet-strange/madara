use std::sync::Arc;

use blockifier::execution::contract_class::{
    ClassInfo, ContractClass, ContractClassV0, ContractClassV0Inner, ContractClassV1,
};
use blockifier::execution::errors::ContractClassError;
use blockifier::transaction::account_transaction::AccountTransaction;
use blockifier::transaction::transactions::{DeclareTransaction, DeployAccountTransaction, InvokeTransaction};
use cairo_lang_starknet_classes::casm_contract_class::{
    CasmContractClass, CasmContractEntryPoint, CasmContractEntryPoints, StarknetSierraCompilationError,
};
use cairo_lang_starknet_classes::contract_class::{
    ContractClass as SierraContractClass, ContractEntryPoint, ContractEntryPoints,
};
use cairo_lang_utils::bigint::BigUintAsHex;
use cairo_vm::types::program::Program;
use flate2::read::GzDecoder;
use indexmap::IndexMap;
use mp_felt::Felt252Wrapper;
use num_bigint::{BigInt, BigUint, Sign};
use starknet_api::core::{calculate_contract_address, EntryPointSelector};
use starknet_api::data_availability::DataAvailabilityMode;
use starknet_api::deprecated_contract_class::{EntryPoint, EntryPointOffset, EntryPointType};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::{
    AccountDeploymentData, Calldata, Fee, PaymasterData, Resource, ResourceBounds, ResourceBoundsMapping, Tip,
    TransactionHash, TransactionSignature,
};
use starknet_api::StarknetApiError;
use starknet_core::types::contract::legacy::{
    LegacyContractClass, LegacyEntrypointOffset, RawLegacyEntryPoint, RawLegacyEntryPoints,
};
use starknet_core::types::contract::{CompiledClass, CompiledClassEntrypoint, CompiledClassEntrypointList};
use starknet_core::types::{
    BroadcastedDeclareTransaction, BroadcastedDeclareTransactionV1, BroadcastedDeclareTransactionV2,
    BroadcastedDeclareTransactionV3, BroadcastedDeployAccountTransaction, BroadcastedInvokeTransaction,
    BroadcastedTransaction, CompressedLegacyContractClass, EntryPointsByType, FlattenedSierraClass,
    LegacyContractEntryPoint, LegacyEntryPointsByType, SierraEntryPoint,
};
use starknet_crypto::FieldElement;
use thiserror::Error;

use crate::compute_hash::ComputeTransactionHash;

#[derive(Debug, Error)]
pub enum BroadcastedTransactionConversionError {
    #[error("Max fee should not be greater than u128::MAX")]
    MaxFeeTooBig,
    #[error("Failed to decompress the program")]
    ProgramDecompressionFailed,
    #[error("Failed to deserialize the program")]
    ProgramDeserializationFailed,
    #[error("Failed compute the hash of the contract class")]
    ClassHashComputationFailed,
    #[error("Failed to convert to CasmContractClass")]
    CasmContractClassConversionFailed,
    #[error("Compiled class hash does not match the class hash")]
    InvalidCompiledClassHash,
    #[error("Failed to compile to Sierra")]
    SierraCompilationFailed,
    #[error("This transaction version is not supported")]
    UnsuportedTransactionVersion,
    #[error("This transaction version is invalid for this tx")]
    InvalidTransactionVersion,
    #[error(transparent)]
    StarknetApi(#[from] StarknetApiError),
    #[error(transparent)]
    ContractClass(#[from] ContractClassError),
}

pub fn try_account_tx_from_broadcasted_tx(
    tx: BroadcastedTransaction,
    chain_id: Felt252Wrapper,
) -> Result<AccountTransaction, BroadcastedTransactionConversionError> {
    match tx {
        BroadcastedTransaction::Invoke(tx) => {
            try_invoke_tx_from_broadcasted_invoke_tx(tx, chain_id).map(AccountTransaction::Invoke)
        }
        BroadcastedTransaction::Declare(tx) => {
            try_declare_tx_from_broadcasted_declare_tx(tx, chain_id).map(AccountTransaction::Declare)
        }
        BroadcastedTransaction::DeployAccount(tx) => {
            try_deploy_tx_from_broadcasted_deploy_tx(tx, chain_id).map(AccountTransaction::DeployAccount)
        }
    }
}

pub fn try_declare_tx_from_broadcasted_declare_tx(
    value: BroadcastedDeclareTransaction,
    chain_id: Felt252Wrapper,
) -> Result<DeclareTransaction, BroadcastedTransactionConversionError> {
    fn try_new_declare_transaction(
        tx: starknet_api::transaction::DeclareTransaction,
        tx_hash: TransactionHash,
        class_info: ClassInfo,
        is_query: bool,
    ) -> Result<DeclareTransaction, BroadcastedTransactionConversionError> {
        if is_query {
            DeclareTransaction::new_for_query(tx, tx_hash, class_info)
        } else {
            DeclareTransaction::new(tx, tx_hash, class_info)
        }
        .map_err(|_| BroadcastedTransactionConversionError::InvalidTransactionVersion)
    }

    let user_tx = match value {
        BroadcastedDeclareTransaction::V1(BroadcastedDeclareTransactionV1 {
            max_fee,
            signature,
            nonce,
            contract_class: compresed_contract_class,
            sender_address,
            is_query,
        }) => {
            // Create a GzipDecoder to decompress the bytes
            let mut gz = GzDecoder::new(&compresed_contract_class.program[..]);

            // Read the decompressed bytes into a Vec<u8>
            let mut decompressed_bytes = Vec::new();
            std::io::Read::read_to_end(&mut gz, &mut decompressed_bytes)
                .map_err(|_| BroadcastedTransactionConversionError::ProgramDecompressionFailed)?;

            let class_hash = {
                let legacy_contract_class = LegacyContractClass {
                    program: serde_json::from_slice(decompressed_bytes.as_slice())
                        .map_err(|_| BroadcastedTransactionConversionError::ProgramDeserializationFailed)?,
                    abi: match compresed_contract_class.abi.as_ref() {
                        Some(abi) => abi.iter().cloned().map(|entry| entry.into()).collect::<Vec<_>>(),
                        None => vec![],
                    },
                    entry_points_by_type: to_raw_legacy_entry_points(
                        compresed_contract_class.entry_points_by_type.clone(),
                    ),
                };

                legacy_contract_class
                    .class_hash()
                    .map_err(|_| BroadcastedTransactionConversionError::ClassHashComputationFailed)?
            };
            let abi_length = compresed_contract_class.abi.as_ref().map(|abi| abi.len()).unwrap_or_default();
            let tx =
                starknet_api::transaction::DeclareTransaction::V1(starknet_api::transaction::DeclareTransactionV0V1 {
                    max_fee: Fee(max_fee
                        .try_into()
                        .map_err(|_| BroadcastedTransactionConversionError::MaxFeeTooBig)?),
                    signature: TransactionSignature(
                        signature.into_iter().map(|v| Felt252Wrapper::from(v).into()).collect(),
                    ),
                    nonce: Felt252Wrapper::from(nonce).into(),
                    class_hash: Felt252Wrapper::from(class_hash).into(),
                    sender_address: Felt252Wrapper::from(sender_address).into(),
                });

            let contract_class = instantiate_blockifier_contract_class(compresed_contract_class, decompressed_bytes)?;
            let tx_hash = tx.compute_hash(chain_id, is_query);

            try_new_declare_transaction(tx, tx_hash, ClassInfo::new(&contract_class, 0, abi_length)?, is_query)?
        }
        BroadcastedDeclareTransaction::V2(BroadcastedDeclareTransactionV2 {
            max_fee,
            signature,
            nonce,
            contract_class: flattened_contract_class,
            sender_address,
            compiled_class_hash,
            is_query,
        }) => {
            let sierra_contract_class = Felt252Wrapper::from(flattened_contract_class.class_hash()).into();
            let sierra_program_length = flattened_contract_class.sierra_program.len();
            let abi_length = flattened_contract_class.abi.len();

            let casm_contract_class = flattened_sierra_to_casm_contract_class(flattened_contract_class)
                .map_err(|_| BroadcastedTransactionConversionError::SierraCompilationFailed)?;
            // ensure that the user has sign the correct class hash
            if get_casm_contract_class_hash(&casm_contract_class) != compiled_class_hash {
                return Err(BroadcastedTransactionConversionError::InvalidCompiledClassHash);
            }
            let tx =
                starknet_api::transaction::DeclareTransaction::V2(starknet_api::transaction::DeclareTransactionV2 {
                    max_fee: Fee(max_fee
                        .try_into()
                        .map_err(|_| BroadcastedTransactionConversionError::MaxFeeTooBig)?),
                    signature: TransactionSignature(
                        signature.into_iter().map(|v| Felt252Wrapper::from(v).into()).collect(),
                    ),
                    nonce: Felt252Wrapper::from(nonce).into(),
                    class_hash: sierra_contract_class,
                    sender_address: Felt252Wrapper::from(sender_address).into(),
                    compiled_class_hash: Felt252Wrapper::from(compiled_class_hash).into(),
                });

            let tx_hash = tx.compute_hash(chain_id, is_query);
            let contract_class = ContractClass::V1(
                ContractClassV1::try_from(casm_contract_class)
                    .map_err(|_| BroadcastedTransactionConversionError::CasmContractClassConversionFailed)?,
            );

            try_new_declare_transaction(
                tx,
                tx_hash,
                ClassInfo::new(&contract_class, sierra_program_length, abi_length)?,
                is_query,
            )?
        }
        BroadcastedDeclareTransaction::V3(BroadcastedDeclareTransactionV3 {
            sender_address,
            compiled_class_hash,
            signature,
            nonce,
            contract_class: flattened_contract_class,
            resource_bounds,
            tip,
            paymaster_data,
            account_deployment_data,
            nonce_data_availability_mode,
            fee_data_availability_mode,
            is_query,
        }) => {
            let sierra_contract_class = Felt252Wrapper::from(flattened_contract_class.class_hash()).into();
            let sierra_program_length = flattened_contract_class.sierra_program.len();
            let abi_length = flattened_contract_class.abi.len();

            let casm_contract_class = flattened_sierra_to_casm_contract_class(flattened_contract_class)
                .map_err(|_| BroadcastedTransactionConversionError::SierraCompilationFailed)?;

            let tx =
                starknet_api::transaction::DeclareTransaction::V3(starknet_api::transaction::DeclareTransactionV3 {
                    resource_bounds: resource_bounds_mapping_conversion(resource_bounds),
                    tip: Tip(tip),
                    signature: TransactionSignature(
                        signature.into_iter().map(|v| Felt252Wrapper::from(v).into()).collect(),
                    ),
                    nonce: Felt252Wrapper::from(nonce).into(),
                    class_hash: sierra_contract_class,
                    compiled_class_hash: Felt252Wrapper::from(compiled_class_hash).into(),
                    sender_address: Felt252Wrapper::from(sender_address).into(),
                    nonce_data_availability_mode: data_availability_mode_conversion(nonce_data_availability_mode),
                    fee_data_availability_mode: data_availability_mode_conversion(fee_data_availability_mode),
                    paymaster_data: PaymasterData(
                        paymaster_data.into_iter().map(|v| Felt252Wrapper::from(v).into()).collect(),
                    ),
                    account_deployment_data: AccountDeploymentData(
                        account_deployment_data.into_iter().map(|v| Felt252Wrapper::from(v).into()).collect(),
                    ),
                });

            let tx_hash = tx.compute_hash(chain_id, is_query);
            let contract_class = ContractClass::V1(
                ContractClassV1::try_from(casm_contract_class)
                    .map_err(|_| BroadcastedTransactionConversionError::CasmContractClassConversionFailed)?,
            );

            try_new_declare_transaction(
                tx,
                tx_hash,
                ClassInfo::new(&contract_class, sierra_program_length, abi_length)?,
                is_query,
            )?
        }
    };

    Ok(user_tx)
}

fn instantiate_blockifier_contract_class(
    contract_class: Arc<CompressedLegacyContractClass>,
    program_decompressed_bytes: Vec<u8>,
) -> Result<ContractClass, BroadcastedTransactionConversionError> {
    // Deserialize it then
    let program: Program = Program::from_bytes(&program_decompressed_bytes, None)
        .map_err(|_| BroadcastedTransactionConversionError::ProgramDeserializationFailed)?;

    let mut entry_points_by_type = <IndexMap<EntryPointType, Vec<EntryPoint>>>::new();
    entry_points_by_type.insert(
        EntryPointType::Constructor,
        contract_class
            .entry_points_by_type
            .constructor
            .iter()
            .map(|entry_point| -> EntryPoint {
                EntryPoint {
                    selector: EntryPointSelector(StarkFelt(entry_point.selector.to_bytes_be())),
                    offset: EntryPointOffset(entry_point.offset),
                }
            })
            .collect::<Vec<EntryPoint>>(),
    );
    entry_points_by_type.insert(
        EntryPointType::External,
        contract_class
            .entry_points_by_type
            .external
            .iter()
            .map(|entry_point| -> EntryPoint {
                EntryPoint {
                    selector: EntryPointSelector(StarkFelt(entry_point.selector.to_bytes_be())),
                    offset: EntryPointOffset(entry_point.offset),
                }
            })
            .collect::<Vec<EntryPoint>>(),
    );
    entry_points_by_type.insert(
        EntryPointType::L1Handler,
        contract_class
            .entry_points_by_type
            .l1_handler
            .iter()
            .map(|entry_point| -> EntryPoint {
                EntryPoint {
                    selector: EntryPointSelector(StarkFelt(entry_point.selector.to_bytes_be())),
                    offset: EntryPointOffset(entry_point.offset),
                }
            })
            .collect::<Vec<EntryPoint>>(),
    );

    let contract_class =
        ContractClass::V0(ContractClassV0(Arc::new(ContractClassV0Inner { program, entry_points_by_type })));

    Ok(contract_class)
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

/// Converts a [FlattenedSierraClass] to a [CasmContractClass]
fn flattened_sierra_to_casm_contract_class(
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
pub fn get_casm_contract_class_hash(casm_contract_class: &CasmContractClass) -> FieldElement {
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
        // TODO: convert those too
        // Actually, maybe those are not needed for us and we can leave them as it is.
        // I don't know
        // Maybe it's not needed for execution, but should be stored somewhere in order for the RPC to be able to return
        // those
        hints: vec![],
        pythonic_hints: None,
        bytecode_segment_lengths: vec![],
    }
}

/// Converts a [BigUint] to a [FieldElement]
fn biguint_to_field_element(value: &BigUint) -> FieldElement {
    let bytes = value.to_bytes_be();
    FieldElement::from_byte_slice_be(bytes.as_slice()).unwrap()
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

pub fn try_invoke_tx_from_broadcasted_invoke_tx(
    broadcasted_tx: BroadcastedInvokeTransaction,
    chain_id: Felt252Wrapper,
) -> Result<InvokeTransaction, BroadcastedTransactionConversionError> {
    Ok(match broadcasted_tx {
        BroadcastedInvokeTransaction::V1(bc_tx) => {
            let tx = starknet_api::transaction::InvokeTransaction::V1(starknet_api::transaction::InvokeTransactionV1 {
                max_fee: Fee(bc_tx
                    .max_fee
                    .try_into()
                    .map_err(|_| BroadcastedTransactionConversionError::MaxFeeTooBig)?),
                signature: TransactionSignature(
                    bc_tx.signature.into_iter().map(|v| Felt252Wrapper::from(v).into()).collect(),
                ),
                nonce: Felt252Wrapper::from(bc_tx.nonce).into(),
                sender_address: Felt252Wrapper::from(bc_tx.sender_address).into(),
                calldata: Calldata(Arc::new(
                    bc_tx.calldata.into_iter().map(|v| Felt252Wrapper::from(v).into()).collect(),
                )),
            });
            let tx_hash = tx.compute_hash(chain_id, bc_tx.is_query);

            InvokeTransaction { tx, tx_hash, only_query: bc_tx.is_query }
        }
        BroadcastedInvokeTransaction::V3(bc_tx) => {
            let tx = starknet_api::transaction::InvokeTransaction::V3(starknet_api::transaction::InvokeTransactionV3 {
                signature: TransactionSignature(
                    bc_tx.signature.into_iter().map(|v| Felt252Wrapper::from(v).into()).collect(),
                ),
                nonce: Felt252Wrapper::from(bc_tx.nonce).into(),
                sender_address: Felt252Wrapper::from(bc_tx.sender_address).into(),
                calldata: Calldata(Arc::new(
                    bc_tx.calldata.into_iter().map(|v| Felt252Wrapper::from(v).into()).collect(),
                )),
                resource_bounds: resource_bounds_mapping_conversion(bc_tx.resource_bounds),
                tip: Tip(bc_tx.tip),
                nonce_data_availability_mode: data_availability_mode_conversion(bc_tx.nonce_data_availability_mode),
                fee_data_availability_mode: data_availability_mode_conversion(bc_tx.fee_data_availability_mode),
                paymaster_data: PaymasterData(
                    bc_tx.paymaster_data.into_iter().map(|v| Felt252Wrapper::from(v).into()).collect(),
                ),
                account_deployment_data: AccountDeploymentData(
                    bc_tx.account_deployment_data.into_iter().map(|v| Felt252Wrapper::from(v).into()).collect(),
                ),
            });
            let tx_hash = tx.compute_hash(chain_id, bc_tx.is_query);

            InvokeTransaction { tx, tx_hash, only_query: bc_tx.is_query }
        }
    })
}

pub fn try_deploy_tx_from_broadcasted_deploy_tx(
    broadcasted_tx: BroadcastedDeployAccountTransaction,
    chain_id: Felt252Wrapper,
) -> Result<DeployAccountTransaction, BroadcastedTransactionConversionError> {
    Ok(match broadcasted_tx {
        BroadcastedDeployAccountTransaction::V1(bc_tx) => {
            let tx = starknet_api::transaction::DeployAccountTransaction::V1(
                starknet_api::transaction::DeployAccountTransactionV1 {
                    max_fee: Fee(bc_tx
                        .max_fee
                        .try_into()
                        .map_err(|_| BroadcastedTransactionConversionError::MaxFeeTooBig)?),
                    signature: TransactionSignature(
                        bc_tx.signature.into_iter().map(|v| Felt252Wrapper::from(v).into()).collect(),
                    ),
                    nonce: Felt252Wrapper::from(bc_tx.nonce).into(),
                    contract_address_salt: Felt252Wrapper::from(bc_tx.contract_address_salt).into(),
                    constructor_calldata: Calldata(Arc::new(
                        bc_tx.constructor_calldata.into_iter().map(|v| Felt252Wrapper::from(v).into()).collect(),
                    )),
                    class_hash: Felt252Wrapper::from(bc_tx.class_hash).into(),
                },
            );
            let tx_hash = tx.compute_hash(chain_id, bc_tx.is_query);
            let contract_address = calculate_contract_address(
                tx.contract_address_salt(),
                tx.class_hash(),
                &tx.constructor_calldata(),
                Default::default(),
            )?;
            DeployAccountTransaction { tx, tx_hash, contract_address, only_query: bc_tx.is_query }
        }
        BroadcastedDeployAccountTransaction::V3(bc_tx) => {
            let tx = starknet_api::transaction::DeployAccountTransaction::V3(
                starknet_api::transaction::DeployAccountTransactionV3 {
                    signature: TransactionSignature(
                        bc_tx.signature.into_iter().map(|v| Felt252Wrapper::from(v).into()).collect(),
                    ),
                    nonce: Felt252Wrapper::from(bc_tx.nonce).into(),
                    contract_address_salt: Felt252Wrapper::from(bc_tx.contract_address_salt).into(),
                    constructor_calldata: Calldata(Arc::new(
                        bc_tx.constructor_calldata.into_iter().map(|v| Felt252Wrapper::from(v).into()).collect(),
                    )),
                    class_hash: Felt252Wrapper::from(bc_tx.class_hash).into(),
                    resource_bounds: resource_bounds_mapping_conversion(bc_tx.resource_bounds),
                    tip: Tip(bc_tx.tip),
                    nonce_data_availability_mode: data_availability_mode_conversion(bc_tx.nonce_data_availability_mode),
                    fee_data_availability_mode: data_availability_mode_conversion(bc_tx.fee_data_availability_mode),
                    paymaster_data: PaymasterData(
                        bc_tx.paymaster_data.into_iter().map(|v| Felt252Wrapper::from(v).into()).collect(),
                    ),
                },
            );
            let tx_hash = tx.compute_hash(chain_id, bc_tx.is_query);
            let contract_address = calculate_contract_address(
                tx.contract_address_salt(),
                tx.class_hash(),
                &tx.constructor_calldata(),
                Default::default(),
            )?;
            DeployAccountTransaction { tx, tx_hash, contract_address, only_query: bc_tx.is_query }
        }
    })
}

fn data_availability_mode_conversion(
    da_mode: starknet_core::types::DataAvailabilityMode,
) -> starknet_api::data_availability::DataAvailabilityMode {
    match da_mode {
        starknet_core::types::DataAvailabilityMode::L1 => DataAvailabilityMode::L1,
        starknet_core::types::DataAvailabilityMode::L2 => DataAvailabilityMode::L2,
    }
}

fn resource_bounds_mapping_conversion(
    resource_bounds: starknet_core::types::ResourceBoundsMapping,
) -> starknet_api::transaction::ResourceBoundsMapping {
    ResourceBoundsMapping::try_from(vec![
        (
            Resource::L1Gas,
            ResourceBounds {
                max_amount: resource_bounds.l1_gas.max_amount,
                max_price_per_unit: resource_bounds.l1_gas.max_price_per_unit,
            },
        ),
        (
            Resource::L2Gas,
            ResourceBounds {
                max_amount: resource_bounds.l2_gas.max_amount,
                max_price_per_unit: resource_bounds.l2_gas.max_price_per_unit,
            },
        ),
    ])
    .unwrap()
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use starknet_core::types::contract::SierraClass;
    use starknet_core::types::FlattenedSierraClass;

    use super::*;

    const CAIRO_1_NO_VALIDATE_ACCOUNT_COMPILED_CLASS_HASH: &str =
        "0xdf4d3042eec107abe704619f13d92bbe01a58029311b7a1886b23dcbb4ea87";
    fn get_compressed_legacy_contract_class() -> CompressedLegacyContractClass {
        let contract_class_bytes = include_bytes!("../../../../cairo-contracts/build/test.json");

        let contract_class: LegacyContractClass = serde_json::from_slice(contract_class_bytes).unwrap();
        let compressed_contract_class: CompressedLegacyContractClass = contract_class.compress().unwrap();

        compressed_contract_class
    }

    fn get_flattened_sierra_contract_class() -> FlattenedSierraClass {
        // when HelloStarknet is compiled into Sierra, the output does not have inputs: [] in the events ABI
        // this has been manually added right now because starknet-rs expects it
        let contract_class_bytes =
            include_bytes!("../../../../cairo-contracts/build/cairo_1/HelloStarknet.sierra.json");

        let contract_class: SierraClass = serde_json::from_slice(contract_class_bytes).unwrap();
        let flattened_contract_class: FlattenedSierraClass = contract_class.flatten().unwrap();

        flattened_contract_class
    }

    #[test]
    fn try_into_declare_transaction_v1_valid() {
        let compressed_contract_class = get_compressed_legacy_contract_class();

        let txn = BroadcastedDeclareTransactionV1 {
            max_fee: FieldElement::default(),
            signature: vec![FieldElement::default()],
            nonce: FieldElement::default(),
            contract_class: Arc::new(compressed_contract_class),
            sender_address: FieldElement::default(),
            is_query: false,
        };

        let input: BroadcastedDeclareTransaction = BroadcastedDeclareTransaction::V1(txn);
        assert!(try_declare_tx_from_broadcasted_declare_tx(input, Default::default()).is_ok());
    }

    #[test]
    fn try_into_declare_transaction_v1_bad_gzip() {
        let mut compressed_contract_class = get_compressed_legacy_contract_class();

        // Manually change some bytes so its no longer a valid gzip
        if let Some(value) = compressed_contract_class.program.get_mut(0) {
            *value = 1;
        }
        if let Some(value) = compressed_contract_class.program.get_mut(1) {
            *value = 1;
        }

        let txn = BroadcastedDeclareTransactionV1 {
            max_fee: FieldElement::default(),
            signature: vec![FieldElement::default()],
            nonce: FieldElement::default(),
            contract_class: Arc::new(compressed_contract_class),
            sender_address: FieldElement::default(),
            is_query: false,
        };

        let input: BroadcastedDeclareTransaction = BroadcastedDeclareTransaction::V1(txn);
        assert_matches!(
            try_declare_tx_from_broadcasted_declare_tx(input, Default::default()),
            Err(BroadcastedTransactionConversionError::ProgramDecompressionFailed)
        );
    }

    #[test]
    fn try_into_declare_transaction_v2_with_correct_compiled_class_hash() {
        let flattened_contract_class: FlattenedSierraClass = get_flattened_sierra_contract_class();

        let txn = BroadcastedDeclareTransactionV2 {
            max_fee: FieldElement::default(),
            signature: vec![FieldElement::default()],
            nonce: FieldElement::default(),
            contract_class: Arc::new(flattened_contract_class),
            sender_address: FieldElement::default(),
            compiled_class_hash: FieldElement::from_hex_be(CAIRO_1_NO_VALIDATE_ACCOUNT_COMPILED_CLASS_HASH).unwrap(),
            is_query: false,
        };

        let input: BroadcastedDeclareTransaction = BroadcastedDeclareTransaction::V2(txn);
        assert!(try_declare_tx_from_broadcasted_declare_tx(input, Default::default()).is_ok());
    }

    #[test]
    fn try_into_declare_transaction_v2_with_incorrect_compiled_class_hash() {
        let flattened_contract_class: FlattenedSierraClass = get_flattened_sierra_contract_class();

        let txn = BroadcastedDeclareTransactionV2 {
            max_fee: FieldElement::default(),
            signature: vec![FieldElement::default()],
            nonce: FieldElement::default(),
            contract_class: Arc::new(flattened_contract_class),
            sender_address: FieldElement::default(),
            compiled_class_hash: FieldElement::from_hex_be("0x1").unwrap(), // incorrect compiled class hash
            is_query: false,
        };

        let input: BroadcastedDeclareTransaction = BroadcastedDeclareTransaction::V2(txn);

        assert_matches!(
            try_declare_tx_from_broadcasted_declare_tx(input, Default::default()),
            Err(BroadcastedTransactionConversionError::InvalidCompiledClassHash)
        );
    }
}
