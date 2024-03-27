use alloc::sync::Arc;

use mp_felt::Felt252Wrapper;
use mp_hashers::pedersen::PedersenHasher;
use starknet_api::core::{ClassHash, CompiledClassHash, ContractAddress, Nonce, PatriciaKey};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::{
    Calldata, ContractAddressSalt, DeclareTransaction, DeclareTransactionV0V1, DeclareTransactionV2,
    DeployAccountTransactionV1, Fee, InvokeTransactionV1, L1HandlerTransaction, TransactionHash, TransactionSignature,
};
use starknet_crypto::FieldElement;

use crate::compute_hash::ComputeTransactionHash;

#[test]
fn test_deploy_account_tx_hash() {
    // Computed with `calculateDeployAccountTransactionHash` from the starknet.js
    let expected_tx_hash = TransactionHash(
        StarkFelt::try_from("0x04cf7bf97d4f8ef73eb83d2e6fb8e5354c04f2121b9bd38510220eff3a07e9df").unwrap(),
    );

    let chain_id = Felt252Wrapper(FieldElement::from_byte_slice_be(b"SN_GOERLI").unwrap());

    let transaction = DeployAccountTransactionV1 {
        max_fee: Fee(1),
        signature: TransactionSignature(vec![]),
        nonce: Nonce(StarkFelt::ZERO),
        constructor_calldata: Calldata(Arc::new(vec![StarkFelt::ONE, StarkFelt::TWO, StarkFelt::THREE])),
        contract_address_salt: ContractAddressSalt(StarkFelt::ZERO),
        class_hash: ClassHash(StarkFelt::THREE),
    };

    let tx_hash = transaction.compute_hash::<PedersenHasher>(chain_id, false);

    assert_eq!(tx_hash, expected_tx_hash);
}

#[test]
fn test_declare_v0_tx_hash() {
    // Computed with `calculate_declare_transaction_hash` from the cairo lang package
    let expected_tx_hash = TransactionHash(
        StarkFelt::try_from("0x052b849ca86ca1a1ce6ac7e069900a221b5741786bffe023804ef714f7bb46da").unwrap(),
    );

    let chain_id = Felt252Wrapper(FieldElement::from_byte_slice_be(b"SN_GOERLI").unwrap());

    let transaction = DeclareTransaction::V0(DeclareTransactionV0V1 {
        max_fee: Fee(1),
        signature: TransactionSignature(vec![]),
        nonce: Nonce(StarkFelt::ZERO),
        class_hash: ClassHash(StarkFelt::THREE),
        sender_address: ContractAddress(PatriciaKey(StarkFelt::from(19911991_u128))),
    });

    let tx_hash = transaction.compute_hash::<PedersenHasher>(chain_id, false);

    assert_eq!(tx_hash, expected_tx_hash);
}

#[test]
fn test_declare_v1_tx_hash() {
    // Computed with `calculate_declare_transaction_hash` from the cairo lang package
    let expected_tx_hash = TransactionHash(
        StarkFelt::try_from("0x077f205d4855199564663dc9810c1edfcf97573393033dedc3f12dac740aac13").unwrap(),
    );

    let chain_id = Felt252Wrapper(FieldElement::from_byte_slice_be(b"SN_GOERLI").unwrap());

    let transaction = DeclareTransaction::V1(DeclareTransactionV0V1 {
        max_fee: Fee(1),
        signature: TransactionSignature(vec![]),
        nonce: Nonce(StarkFelt::ZERO),
        class_hash: ClassHash(StarkFelt::THREE),
        sender_address: ContractAddress(PatriciaKey(StarkFelt::from(19911991_u128))),
    });

    let tx_hash = transaction.compute_hash::<PedersenHasher>(chain_id, false);
    assert_eq!(tx_hash, expected_tx_hash);
}

#[test]
fn test_declare_v2_tx_hash() {
    // Computed with `calculate_declare_transaction_hash` from the cairo lang package
    let expected_tx_hash = TransactionHash(
        StarkFelt::try_from("0x7ca2d13e00a7249a7f61cf65c20a20f2870276d4db00d816e836eb2ca9029ae").unwrap(),
    );

    let chain_id = Felt252Wrapper(FieldElement::from_byte_slice_be(b"SN_GOERLI").unwrap());

    let transaction = DeclareTransactionV2 {
        max_fee: Fee(1),
        signature: TransactionSignature(vec![]),
        nonce: Nonce(StarkFelt::ZERO),
        class_hash: ClassHash(StarkFelt::THREE),
        sender_address: ContractAddress(PatriciaKey(StarkFelt::from(19911991_u128))),
        compiled_class_hash: CompiledClassHash(StarkFelt::THREE),
    };

    let tx_hash = transaction.compute_hash::<PedersenHasher>(chain_id, false);

    assert_eq!(tx_hash, expected_tx_hash);

    let declare_v2_transaction = DeclareTransaction::V2(transaction);
    let tx_hash = declare_v2_transaction.compute_hash::<PedersenHasher>(chain_id, false);
    assert_eq!(tx_hash, expected_tx_hash);
}

#[test]
fn test_invoke_tx_v1_hash() {
    // Computed with `calculate_transaction_hash_common` from the cairo lang package
    let expected_tx_hash = TransactionHash(
        StarkFelt::try_from("0x062633b1f3d64708df3d0d44706b388f841ed4534346be6ad60336c8eb2f4b3e").unwrap(),
    );

    let chain_id = Felt252Wrapper(FieldElement::from_byte_slice_be(b"SN_GOERLI").unwrap());

    let transaction = InvokeTransactionV1 {
        max_fee: Fee(1),
        signature: TransactionSignature(vec![]),
        nonce: Nonce(StarkFelt::ZERO),
        sender_address: ContractAddress(PatriciaKey(StarkFelt::from(19911991_u128))),
        calldata: Calldata(Arc::new(vec![StarkFelt::ONE, StarkFelt::TWO, StarkFelt::THREE])),
    };

    let tx_hash = transaction.compute_hash::<PedersenHasher>(chain_id, false);

    assert_eq!(tx_hash, expected_tx_hash);
}

#[test]
fn test_handle_l1_message_tx_hash() {
    // Computed with `calculate_transaction_hash_common` from the cairo lang package
    let expected_tx_hash = TransactionHash(
        StarkFelt::try_from("0x023f18bb43e61985fba987824a9b8fdea96276e38e34702c72de4250ba91f518").unwrap(),
    );

    let chain_id = Felt252Wrapper(FieldElement::from_byte_slice_be(b"SN_GOERLI").unwrap());

    let transaction = L1HandlerTransaction {
        nonce: Default::default(),
        contract_address: Default::default(),
        entry_point_selector: Default::default(),
        calldata: Default::default(),
        version: Default::default(),
    };

    let tx_hash = transaction.compute_hash::<PedersenHasher>(chain_id, false);

    assert_eq!(tx_hash, expected_tx_hash);
}
