use std::sync::Arc;

use mp_starknet::transaction::types::{BroadcastedTransactionConversionErrorWrapper, DeclareTransaction, MaxArraySize};
use sp_core::TypedGet;
use starknet_core::types::contract::legacy::LegacyContractClass;
use starknet_core::types::contract::SierraClass;
use starknet_core::types::{
    BlockTag, BroadcastedDeclareTransactionV1, BroadcastedDeclareTransactionV2, CompressedLegacyContractClass,
    FlattenedSierraClass,
};

use super::*;
use crate::constants::CAIRO_1_NO_VALIDATE_ACCOUNT_COMPILED_CLASS_HASH;
use crate::utils::to_declare_transaction;

#[test]
fn block_id_serialization() {
    assert_eq!(serde_json::to_value(BlockId::Number(42)).unwrap(), serde_json::json!({"block_number": 42}));
    assert_eq!(
        serde_json::to_value(BlockId::Hash(FieldElement::from_hex_be("0x42").unwrap())).unwrap(),
        serde_json::json!({"block_hash": "0x42"})
    );
    assert_eq!(serde_json::to_value(BlockId::Tag(BlockTag::Latest)).unwrap(), "latest");
    assert_eq!(serde_json::to_value(BlockId::Tag(BlockTag::Pending)).unwrap(), "pending");
}

#[test]
fn block_id_deserialization() {
    #[derive(serde::Serialize, serde::Deserialize)]
    struct Payload {
        #[serde(rename = "block_id")]
        block_id: BlockId,
    }

    assert_eq!(
        serde_json::from_str::<Payload>("{ \"block_id\": \"latest\" }").unwrap().block_id,
        BlockId::Tag(BlockTag::Latest)
    );
    assert_eq!(
        serde_json::from_str::<Payload>("{ \"block_id\": \"pending\" }").unwrap().block_id,
        BlockId::Tag(BlockTag::Pending)
    );
    assert_eq!(
        serde_json::from_str::<Payload>("{ \"block_id\": { \"block_hash\": \"0x42\"} }").unwrap().block_id,
        BlockId::Hash(FieldElement::from_hex_be("0x42").unwrap())
    );
    assert_eq!(
        serde_json::from_str::<Payload>("{ \"block_id\": { \"block_number\": 42} }").unwrap().block_id,
        BlockId::Number(42)
    );
}

#[test]
fn test_try_into_declare_transaction_v1_valid() {
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
    let output_result: Result<DeclareTransaction, _> = to_declare_transaction(input);
    assert!(output_result.is_ok());
}

#[test]
fn test_try_into_declare_transaction_v1_max_signature() {
    let compressed_contract_class = get_compressed_legacy_contract_class();

    let txn = BroadcastedDeclareTransactionV1 {
        max_fee: FieldElement::default(),
        signature: vec![FieldElement::default(); MaxArraySize::get() as usize + 1],
        nonce: FieldElement::default(),
        contract_class: Arc::new(compressed_contract_class),
        sender_address: FieldElement::default(),
        is_query: false,
    };

    let input: BroadcastedDeclareTransaction = BroadcastedDeclareTransaction::V1(txn);
    let output_result: Result<DeclareTransaction, _> = to_declare_transaction(input);
    assert!(matches!(output_result.unwrap_err(), BroadcastedTransactionConversionErrorWrapper::SignatureBoundError));
}

#[test]
fn test_try_into_declare_transaction_v1_bad_gzip() {
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
    let output_result: Result<DeclareTransaction, _> = to_declare_transaction(input);
    assert!(matches!(
        output_result.unwrap_err(),
        BroadcastedTransactionConversionErrorWrapper::ContractClassProgramDecompressionError
    ));
}

#[test]
fn test_try_into_declare_transaction_v2_with_correct_compiled_class_hash() {
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
    let output_result: Result<DeclareTransaction, _> = to_declare_transaction(input);

    assert!(output_result.is_ok());
}

#[test]
fn test_try_into_declare_transaction_v2_with_incorrect_compiled_class_hash() {
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
    let output_result: Result<DeclareTransaction, _> = to_declare_transaction(input);

    assert!(matches!(output_result.unwrap_err(), BroadcastedTransactionConversionErrorWrapper::CompiledClassHashError));
}

fn get_compressed_legacy_contract_class() -> CompressedLegacyContractClass {
    let contract_class_bytes = include_bytes!("../../../../cairo-contracts/build/test.json");

    let contract_class: LegacyContractClass = serde_json::from_slice(contract_class_bytes).unwrap();
    let compressed_contract_class: CompressedLegacyContractClass = contract_class.compress().unwrap();

    compressed_contract_class
}

fn get_flattened_sierra_contract_class() -> FlattenedSierraClass {
    // when HelloStarknet is compiled into Sierra, the output does not have inputs: [] in the events ABI
    // this has been manually added right now because starknet-rs expects it
    let contract_class_bytes = include_bytes!("../../../../cairo-contracts/build/cairo_1/HelloStarknet.sierra.json");

    let contract_class: SierraClass = serde_json::from_slice(contract_class_bytes).unwrap();
    let flattened_contract_class: FlattenedSierraClass = contract_class.flatten().unwrap();

    flattened_contract_class
}
