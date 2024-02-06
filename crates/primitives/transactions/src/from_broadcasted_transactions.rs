use alloc::sync::Arc;
use std::collections::HashMap;

use blockifier::execution::contract_class::{ContractClass, ContractClassV0, ContractClassV0Inner, ContractClassV1};
use cairo_lang_casm_contract_class::{CasmContractClass, CasmContractEntryPoint, CasmContractEntryPoints};
use cairo_lang_starknet::contract_class::{
    ContractClass as SierraContractClass, ContractEntryPoint, ContractEntryPoints,
};
use cairo_lang_starknet::contract_class_into_casm_contract_class::StarknetSierraCompilationError;
use cairo_lang_utils::bigint::BigUintAsHex;
use cairo_vm::types::program::Program;
use flate2::read::GzDecoder;
use mp_felt::Felt252Wrapper;
use num_bigint::{BigInt, BigUint, Sign};
use starknet_api::api_core::EntryPointSelector;
use starknet_api::deprecated_contract_class::{EntryPoint, EntryPointOffset, EntryPointType};
use starknet_api::hash::StarkFelt;
use starknet_core::types::contract::legacy::{
    LegacyContractClass, LegacyEntrypointOffset, RawLegacyEntryPoint, RawLegacyEntryPoints,
};
use starknet_core::types::contract::{CompiledClass, CompiledClassEntrypoint, CompiledClassEntrypointList};
use starknet_core::types::{
    BroadcastedDeclareTransaction, BroadcastedDeclareTransactionV1, BroadcastedDeclareTransactionV2,
    BroadcastedDeployAccountTransaction, BroadcastedInvokeTransaction, BroadcastedTransaction,
    CompressedLegacyContractClass, EntryPointsByType, FlattenedSierraClass, LegacyContractEntryPoint,
    LegacyEntryPointsByType, SierraEntryPoint,
};
use starknet_crypto::FieldElement;
use thiserror::Error;

use super::{DeclareTransaction, DeclareTransactionV1, DeclareTransactionV2, UserTransaction};

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
}

impl TryFrom<BroadcastedTransaction> for UserTransaction {
    type Error = BroadcastedTransactionConversionError;

    fn try_from(tx: BroadcastedTransaction) -> Result<Self, Self::Error> {
        match tx {
            BroadcastedTransaction::Invoke(tx) => tx.try_into(),
            BroadcastedTransaction::Declare(tx) => tx.try_into(),
            BroadcastedTransaction::DeployAccount(tx) => tx.try_into(),
        }
    }
}

fn cast_vec_of_field_elements(data: Vec<FieldElement>) -> Vec<Felt252Wrapper> {
    // Non-copy but less dangerous than transmute
    // https://doc.rust-lang.org/std/mem/fn.transmute.html#alternatives

    // Unsafe code but all invariants are checked:

    // 1. ptr must have been allocated using the global allocator -> data is allocated with the Global
    //    allocator.
    // 2. T needs to have the same alignment as what ptr was allocated with -> Felt252Wrapper uses
    //    transparent representation of the inner type.
    // 3. The allocated size in bytes needs to be the same as the pointer -> As FieldElement and
    //    Felt252Wrapper have the same size, and capacity is taken directly from the data Vector, we
    //    will have the same allocated byte size.
    // 4. Length needs to be less than or equal to capacity -> data.len() is always less than or equal
    //    to data.capacity()
    // 5. The first length values must be properly initialized values of type T -> ok since we use data
    //    which was correctly allocated
    // 6. capacity needs to be the capacity that the pointer was allocated with -> data.as_mut_ptr()
    //    returns a pointer to memory having at least capacity initialized memory
    // 7. The allocated size in bytes must be no larger than isize::MAX -> data.capacity() will never be
    //    bigger than isize::MAX (https://doc.rust-lang.org/std/vec/struct.Vec.html#panics-7)
    let mut data = core::mem::ManuallyDrop::new(data);
    unsafe { alloc::vec::Vec::from_raw_parts(data.as_mut_ptr() as *mut Felt252Wrapper, data.len(), data.capacity()) }
}

impl TryFrom<BroadcastedDeclareTransaction> for UserTransaction {
    type Error = BroadcastedTransactionConversionError;

    fn try_from(value: BroadcastedDeclareTransaction) -> Result<Self, Self::Error> {
        let user_tx = match value {
            BroadcastedDeclareTransaction::V1(BroadcastedDeclareTransactionV1 {
                max_fee,
                signature,
                nonce,
                contract_class,
                sender_address,
                is_query,
                ..
            }) => {
                // Create a GzipDecoder to decompress the bytes
                let mut gz = GzDecoder::new(&contract_class.program[..]);

                // Read the decompressed bytes into a Vec<u8>
                let mut decompressed_bytes = Vec::new();
                std::io::Read::read_to_end(&mut gz, &mut decompressed_bytes)
                    .map_err(|_| BroadcastedTransactionConversionError::ProgramDecompressionFailed)?;

                let class_hash = {
                    let legacy_contract_class = LegacyContractClass {
                        program: serde_json::from_slice(decompressed_bytes.as_slice())
                            .map_err(|_| BroadcastedTransactionConversionError::ProgramDeserializationFailed)?,
                        abi: match contract_class.abi.as_ref() {
                            Some(abi) => abi.iter().cloned().map(|entry| entry.into()).collect::<Vec<_>>(),
                            None => vec![],
                        },
                        entry_points_by_type: to_raw_legacy_entry_points(contract_class.entry_points_by_type.clone()),
                    };

                    legacy_contract_class
                        .class_hash()
                        .map_err(|_| BroadcastedTransactionConversionError::ClassHashComputationFailed)?
                };

                let tx = DeclareTransaction::V1(DeclareTransactionV1 {
                    max_fee: max_fee.try_into().map_err(|_| BroadcastedTransactionConversionError::MaxFeeTooBig)?,
                    signature: cast_vec_of_field_elements(signature),
                    nonce: nonce.into(),
                    class_hash: class_hash.into(),
                    sender_address: sender_address.into(),
                    offset_version: is_query,
                });

                let contract_class = instantiate_blockifier_contract_class(contract_class, decompressed_bytes)?;

                UserTransaction::Declare(tx, contract_class)
            }
            BroadcastedDeclareTransaction::V2(BroadcastedDeclareTransactionV2 {
                max_fee,
                signature,
                nonce,
                contract_class,
                sender_address,
                compiled_class_hash,
                is_query,
                ..
            }) => {
                let tx = DeclareTransaction::V2(DeclareTransactionV2 {
                    max_fee: max_fee.try_into().map_err(|_| BroadcastedTransactionConversionError::MaxFeeTooBig)?,
                    signature: cast_vec_of_field_elements(signature),
                    nonce: nonce.into(),
                    class_hash: contract_class.class_hash().into(),
                    sender_address: sender_address.into(),
                    compiled_class_hash: compiled_class_hash.into(),
                    offset_version: is_query,
                });

                let casm_contract_class = flattened_sierra_to_casm_contract_class(contract_class)
                    .map_err(|_| BroadcastedTransactionConversionError::SierraCompilationFailed)?;

                // ensure that the user has sign the correct class hash
                if get_casm_cotract_class_hash(&casm_contract_class) != compiled_class_hash {
                    return Err(BroadcastedTransactionConversionError::InvalidCompiledClassHash);
                }

                let contract_class = ContractClass::V1(
                    ContractClassV1::try_from(casm_contract_class)
                        .map_err(|_| BroadcastedTransactionConversionError::CasmContractClassConversionFailed)?,
                );

                UserTransaction::Declare(tx, contract_class)
            }
        };

        Ok(user_tx)
    }
}

fn instantiate_blockifier_contract_class(
    contract_class: Arc<CompressedLegacyContractClass>,
    program_decompressed_bytes: Vec<u8>,
) -> Result<ContractClass, BroadcastedTransactionConversionError> {
    // Deserialize it then
    let program: Program = Program::from_bytes(&program_decompressed_bytes, None)
        .map_err(|_| BroadcastedTransactionConversionError::ProgramDeserializationFailed)?;

    let mut entry_points_by_type = <HashMap<EntryPointType, Vec<EntryPoint>>>::new();
    entry_points_by_type.insert(
        EntryPointType::Constructor,
        contract_class
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
        contract_class
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
        contract_class
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

impl TryFrom<BroadcastedInvokeTransaction> for UserTransaction {
    type Error = BroadcastedTransactionConversionError;

    fn try_from(value: BroadcastedInvokeTransaction) -> Result<Self, Self::Error> {
        Ok(UserTransaction::Invoke(super::InvokeTransaction::V1(super::InvokeTransactionV1 {
            max_fee: value.max_fee.try_into().map_err(|_| BroadcastedTransactionConversionError::MaxFeeTooBig)?,
            signature: cast_vec_of_field_elements(value.signature),
            nonce: value.nonce.into(),
            sender_address: value.sender_address.into(),
            calldata: cast_vec_of_field_elements(value.calldata),
            offset_version: value.is_query,
        })))
    }
}

impl TryFrom<BroadcastedDeployAccountTransaction> for UserTransaction {
    type Error = BroadcastedTransactionConversionError;

    fn try_from(tx: BroadcastedDeployAccountTransaction) -> Result<Self, Self::Error> {
        let tx = UserTransaction::DeployAccount(super::DeployAccountTransaction {
            max_fee: tx.max_fee.try_into().map_err(|_| BroadcastedTransactionConversionError::MaxFeeTooBig)?,
            signature: cast_vec_of_field_elements(tx.signature),
            nonce: tx.nonce.into(),
            contract_address_salt: tx.contract_address_salt.into(),
            constructor_calldata: cast_vec_of_field_elements(tx.constructor_calldata),
            class_hash: tx.class_hash.into(),
            offset_version: tx.is_query,
        });

        Ok(tx)
    }
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
        assert!(UserTransaction::try_from(input).is_ok());
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
            UserTransaction::try_from(input),
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
        assert!(UserTransaction::try_from(input).is_ok());
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
            UserTransaction::try_from(input),
            Err(BroadcastedTransactionConversionError::InvalidCompiledClassHash)
        );
    }
}
