
use core::convert::TryFrom;

use frame_support::BoundedVec;
use crate::block::{Block, BlockTransactions, Header, MaxTransactions};
use crate::execution::types::{CallEntryPointWrapper, ContractAddressWrapper};
use crate::transaction::types::{MaxArraySize, Transaction};
use sp_core::{Encode, H256, U256};

fn generate_dummy_header() -> Header {
    Header::new(
        H256::from_slice(&[1; 32]),
        U256::from(1),
        U256::from(2),
        ContractAddressWrapper::default(),
        42,
        0,
        H256::from_slice(&[3; 32]),
        0,
        H256::from_slice(&[4; 32]),
        Some(1),
        Some(U256::from(3)),
    )
}

fn generate_dummy_transactions() -> BoundedVec<Transaction, MaxTransactions> {
    let vec_signature = vec![H256::from_low_u64_be(1)];
    let dummy_signature = BoundedVec::<H256, MaxArraySize>::try_from(vec_signature).unwrap();

    vec![
        Transaction {
            version: 1,
            hash: H256::from_low_u64_be(1),
            signature: dummy_signature.clone(),
            sender_address: ContractAddressWrapper::default(),
            nonce: U256::from(100),
            call_entrypoint: CallEntryPointWrapper::default(),
            contract_class: None,
            contract_address_salt: None,
        },
        Transaction {
            version: 1,
            hash: H256::from_low_u64_be(2),
            signature: dummy_signature,
            sender_address: ContractAddressWrapper::default(),
            nonce: U256::from(200),
            call_entrypoint: CallEntryPointWrapper::default(),
            contract_class: None,
            contract_address_salt: None,
        },
    ]
    .try_into()
    .unwrap()
}

#[test]
fn test_header_hash() {
    let header = generate_dummy_header();
    let expected_hash =
        H256::from_slice(frame_support::Hashable::blake2_256(&header.block_number.encode()).as_slice());

    assert_eq!(header.hash(), expected_hash);
}

#[test]
fn test_transactions() {
    let header = generate_dummy_header();
    let transactions = BlockTransactions::Full(generate_dummy_transactions());
    let block = Block::new(header, transactions.clone());

    assert_eq!(block.transactions(), &transactions);
}

#[test]
fn test_transactions_hashes() {
    let header = generate_dummy_header();
    let transactions = generate_dummy_transactions();
    let block = Block::new(header, BlockTransactions::Full(transactions.clone()));

    let expected_hashes: Vec<H256> = transactions.iter().map(|tx| tx.hash).collect();

    assert_eq!(block.transactions_hashes(), expected_hashes);
}

#[test]
fn test_transactions_hashes_from_hashes() {
    let header = generate_dummy_header();
    let transactions = generate_dummy_transactions();
    let vec_hashes: Vec<H256> = transactions.iter().map(|tx| tx.hash).collect();
    let hashes = BoundedVec::<H256, MaxTransactions>::try_from(vec_hashes).unwrap();

    let block = Block::new(header, BlockTransactions::Hashes(hashes.clone()));

    let expected_hashes: Vec<H256> = hashes.into_iter().collect();

    assert_eq!(block.transactions_hashes(), expected_hashes);
}
