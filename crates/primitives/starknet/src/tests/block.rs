use core::convert::TryFrom;

use frame_support::BoundedVec;
use sp_core::{Encode, U256};

use crate::block::{Block, BlockTransactionReceipts, Header, MaxTransactions};
use crate::crypto::hash::pedersen::PedersenHasher;
use crate::execution::types::{CallEntryPointWrapper, ContractAddressWrapper, Felt252Wrapper};
use crate::traits::hash::HasherT;
use crate::transaction::types::{MaxArraySize, Transaction, TransactionReceiptWrapper, TxType};

fn generate_dummy_header() -> Header {
    Header::new(
        Felt252Wrapper::ONE,
        U256::from(1),
        Felt252Wrapper::TWO,
        ContractAddressWrapper::default(),
        42,
        0,
        Felt252Wrapper::THREE,
        0,
        Felt252Wrapper::from_dec_str("4").unwrap(),
        Some(1),
        Some(U256::from(3)),
    )
}

fn generate_dummy_transactions() -> BoundedVec<Transaction, MaxTransactions> {
    let vec_signature = vec![Felt252Wrapper::ONE];
    let dummy_signature = BoundedVec::<Felt252Wrapper, MaxArraySize>::try_from(vec_signature).unwrap();

    vec![
        Transaction {
            tx_type: TxType::Invoke,
            version: 1,
            hash: Felt252Wrapper::ONE,
            signature: dummy_signature.clone(),
            sender_address: ContractAddressWrapper::default(),
            nonce: Felt252Wrapper::from_dec_str("100").unwrap(),
            call_entrypoint: CallEntryPointWrapper::default(),
            contract_class: None,
            contract_address_salt: None,
            max_fee: Felt252Wrapper::from_dec_str("1000").unwrap(),
        },
        Transaction {
            tx_type: TxType::Invoke,
            version: 1,
            hash: Felt252Wrapper::TWO,
            signature: dummy_signature,
            sender_address: ContractAddressWrapper::default(),
            nonce: Felt252Wrapper::from_dec_str("200").unwrap(),
            call_entrypoint: CallEntryPointWrapper::default(),
            contract_class: None,
            contract_address_salt: None,
            max_fee: Felt252Wrapper::from_dec_str("1000").unwrap(),
        },
    ]
    .try_into()
    .unwrap()
}

#[test]
fn test_header_hash() {
    let header = generate_dummy_header();
    let hasher = PedersenHasher::default();
    let expected_hash = hasher.hash(&U256::from(1).encode());

    assert_eq!(header.hash(hasher), expected_hash);
}

#[test]
fn test_transactions() {
    let header = generate_dummy_header();
    let transactions = generate_dummy_transactions();
    let transaction_receipts: BlockTransactionReceipts =
        BoundedVec::<TransactionReceiptWrapper, MaxTransactions>::default();
    let block = Block::new(header, transactions.clone(), transaction_receipts);

    assert_eq!(block.transactions(), &transactions);
}

#[test]
fn test_transactions_hashes() {
    let header = generate_dummy_header();
    let transactions = generate_dummy_transactions();
    let transaction_receipts: BlockTransactionReceipts =
        BoundedVec::<TransactionReceiptWrapper, MaxTransactions>::default();
    let block = Block::new(header, transactions.clone(), transaction_receipts);

    let expected_hashes: Vec<Felt252Wrapper> = transactions.iter().map(|tx| tx.hash).collect();

    assert_eq!(block.transactions_hashes(), expected_hashes);
}

#[test]
fn test_transactions_hashes_from_hashes() {
    let header = generate_dummy_header();
    let transactions = generate_dummy_transactions();
    let transaction_receipts: BlockTransactionReceipts =
        BoundedVec::<TransactionReceiptWrapper, MaxTransactions>::default();
    let block = Block::new(header, transactions.clone(), transaction_receipts);

    let vec_hashes: Vec<Felt252Wrapper> = transactions.iter().map(|tx| tx.hash).collect();
    let hashes = BoundedVec::<Felt252Wrapper, MaxTransactions>::try_from(vec_hashes).unwrap();

    let expected_hashes: Vec<Felt252Wrapper> = hashes.into_iter().collect();

    assert_eq!(block.transactions_hashes(), expected_hashes);
}
