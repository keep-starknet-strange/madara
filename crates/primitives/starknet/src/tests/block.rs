use core::convert::TryFrom;

use frame_support::BoundedVec;
use sp_core::U256;
use starknet_api::api_core::{ChainId, ContractAddress};
use starknet_api::block::{BlockNumber, BlockTimestamp};
use starknet_api::hash::StarkFelt;

use crate::block::{Block, BlockTransactionReceipts, Header, MaxTransactions};
use crate::crypto::hash::pedersen::PedersenHasher;
use crate::execution::types::{CallEntryPointWrapper, ContractAddressWrapper, Felt252Wrapper};
use crate::transaction::types::{MaxArraySize, Transaction, TransactionReceiptWrapper, TxType};

fn generate_dummy_header() -> Header {
    Header::new(
        Felt252Wrapper::ONE,
        1,
        Felt252Wrapper::TWO,
        ContractAddressWrapper::default(),
        42,
        0,
        Felt252Wrapper::THREE,
        0,
        Felt252Wrapper::from_dec_str("4").unwrap(),
        1,
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

    let expected_hash =
        Felt252Wrapper::from_hex_be("0x029da584545c7f3ebdb0c6aca74f0fba99156b1e31e9524c70b42776e50efda6").unwrap();

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

#[test]
fn test_real_header_hash() {
    // Values taken from alpha-mainnet
    let hasher = PedersenHasher::default();

    let block_number = 86000;
    let block_timestamp = 1687235884;
    let global_state_root =
        Felt252Wrapper::from_hex_be("0x006727a7aae8c38618a179aeebccd6302c67ad5f8528894d1dde794e9ae0bbfa").unwrap();
    let parent_block_hash =
        Felt252Wrapper::from_hex_be("0x045543088ce763aba7db8f6bfb33e33cc50af5c2ed5a26d38d5071c352a49c1d").unwrap();
    let sequencer_address =
        Felt252Wrapper::from_hex_be("0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8").unwrap();
    let transaction_count = 197;
    let transaction_commitment =
        Felt252Wrapper::from_hex_be("0x70369cef825889dc005916dba67332b71f270b7af563d0433cee3342dda527d").unwrap();
    let event_count = 1430;
    let event_commitment =
        Felt252Wrapper::from_hex_be("0x2043ba1ef46882ce1dbb17b501fffa4b71f87f618e8f394e9605959d92efdf6").unwrap();
    let protocol_version = 0;
    let extra_data = None;

    let header = Header::new(
        parent_block_hash,
        block_number,
        global_state_root,
        sequencer_address,
        block_timestamp,
        transaction_count,
        transaction_commitment,
        event_count,
        event_commitment,
        protocol_version,
        extra_data,
    );

    let expected_hash =
        Felt252Wrapper::from_hex_be("0x001d126ca058c7e546d59cf4e10728e4b023ca0fb368e8abcabf0b5335f4487a").unwrap();

    assert_eq!(header.hash(hasher), expected_hash);
}

#[test]
fn test_to_block_context() {
    let sequencer_address = Felt252Wrapper::from_hex_be("0xFF").unwrap();
    // Create a block header.
    let block_header = Header { block_number: 1, block_timestamp: 1, sequencer_address, ..Default::default() };
    // Create a fee token address.
    let fee_token_address = Felt252Wrapper::from_hex_be("AA").unwrap();
    // Create a chain id.
    let chain_id = ChainId("0x1".to_string());
    // Try to serialize the block header.
    let block_context = block_header.into_block_context(fee_token_address, chain_id);
    let expected_sequencer_address =
        ContractAddress::try_from(StarkFelt::new(sequencer_address.into()).unwrap()).unwrap();
    let expected_fee_token_address =
        ContractAddress::try_from(StarkFelt::new(fee_token_address.into()).unwrap()).unwrap();
    // Check that the block context was serialized correctly.
    assert_eq!(block_context.block_number, BlockNumber(1));
    assert_eq!(block_context.block_timestamp, BlockTimestamp(1));
    assert_eq!(block_context.sequencer_address, expected_sequencer_address);
    assert_eq!(block_context.fee_token_address, expected_fee_token_address);
}
