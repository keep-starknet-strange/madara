use alloc::sync::Arc;

use starknet_api::api_core::{calculate_contract_address, ContractAddress, PatriciaKey};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::Calldata;
use starknet_crypto::FieldElement;

use crate::crypto::hash::pedersen::PedersenHasher;
use crate::execution::types::Felt252Wrapper;
use crate::transaction::compute_hash::ComputeTransactionHash;
use crate::transaction::{DeclareTransactionV1, DeclareTransactionV2, DeployAccountTransaction, InvokeTransactionV1};

#[test]
fn compute_contract_address_work_like_starknet_api_impl() {
    let tx = DeployAccountTransaction {
        max_fee: Default::default(),
        signature: Default::default(),
        nonce: Default::default(),
        contract_address_salt: Felt252Wrapper::ZERO,
        constructor_calldata: vec![Felt252Wrapper::ONE, Felt252Wrapper::TWO],
        class_hash: Felt252Wrapper::THREE,
    };

    let address = tx.get_account_address();

    let expected_address = calculate_contract_address(
        tx.contract_address_salt.into(),
        tx.class_hash.into(),
        &Calldata(Arc::new(vec![StarkFelt::from(1u128), StarkFelt::from(2u128)])),
        ContractAddress(PatriciaKey(StarkFelt::from(0u128))),
    )
    .unwrap();

    assert_eq!(Felt252Wrapper(address), expected_address.into());
}

#[test]
fn test_deploy_account_tx_hash() {
    // Computed with `calculateDeployAccountTransactionHash` from the starknet.js
    let expected_tx_hash =
        Felt252Wrapper::from_hex_be("0x04cf7bf97d4f8ef73eb83d2e6fb8e5354c04f2121b9bd38510220eff3a07e9df").unwrap();

    let chain_id = Felt252Wrapper(FieldElement::from_byte_slice_be(b"SN_GOERLI").unwrap());

    let transaction = DeployAccountTransaction {
        max_fee: 1,
        signature: vec![],
        nonce: Felt252Wrapper::ZERO,
        constructor_calldata: vec![Felt252Wrapper::ONE, Felt252Wrapper::TWO, Felt252Wrapper::THREE],
        contract_address_salt: Felt252Wrapper::ZERO,
        class_hash: Felt252Wrapper::THREE,
    };

    let tx_hash = transaction.compute_hash::<PedersenHasher>(chain_id, false);

    assert_eq!(tx_hash, expected_tx_hash);
}

#[test]
fn test_declare_v1_tx_hash() {
    // Computed with `calculate_declare_transaction_hash` from the cairo lang package
    let expected_tx_hash =
        Felt252Wrapper::from_hex_be("0x077f205d4855199564663dc9810c1edfcf97573393033dedc3f12dac740aac13").unwrap();

    let chain_id = Felt252Wrapper(FieldElement::from_byte_slice_be(b"SN_GOERLI").unwrap());

    let transaction = DeclareTransactionV1 {
        max_fee: 1,
        signature: vec![],
        nonce: Felt252Wrapper::ZERO,
        class_hash: Felt252Wrapper::THREE,
        sender_address: Felt252Wrapper::from(19911991_u128),
    };

    let tx_hash = transaction.compute_hash::<PedersenHasher>(chain_id, false);

    assert_eq!(tx_hash, expected_tx_hash);
}

#[test]
fn test_declare_v2_tx_hash() {
    // Computed with `calculate_declare_transaction_hash` from the cairo lang package
    let expected_tx_hash =
        Felt252Wrapper::from_hex_be("0x7ca2d13e00a7249a7f61cf65c20a20f2870276d4db00d816e836eb2ca9029ae").unwrap();

    let chain_id = Felt252Wrapper(FieldElement::from_byte_slice_be(b"SN_GOERLI").unwrap());

    let transaction = DeclareTransactionV2 {
        max_fee: 1,
        signature: vec![],
        nonce: Felt252Wrapper::ZERO,
        class_hash: Felt252Wrapper::THREE,
        sender_address: Felt252Wrapper::from(19911991_u128),
        compiled_class_hash: Felt252Wrapper::THREE,
    };

    let tx_hash = transaction.compute_hash::<PedersenHasher>(chain_id, false);

    assert_eq!(tx_hash, expected_tx_hash);
}

#[test]
fn test_invoke_tx_hash() {
    // Computed with `calculate_transaction_hash_common` from the cairo lang package
    let expected_tx_hash =
        Felt252Wrapper::from_hex_be("0x062633b1f3d64708df3d0d44706b388f841ed4534346be6ad60336c8eb2f4b3e").unwrap();

    let chain_id = Felt252Wrapper(FieldElement::from_byte_slice_be(b"SN_GOERLI").unwrap());

    let transaction = InvokeTransactionV1 {
        max_fee: 1,
        signature: vec![],
        nonce: Felt252Wrapper::ZERO,
        sender_address: Felt252Wrapper::from(19911991_u128),
        calldata: vec![Felt252Wrapper::ONE, Felt252Wrapper::TWO, Felt252Wrapper::THREE],
    };

    let tx_hash = transaction.compute_hash::<PedersenHasher>(chain_id, false);

    assert_eq!(tx_hash, expected_tx_hash);
}

// TODO: Add tests for all the other variants
