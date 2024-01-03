use core::convert::TryFrom;

use mp_felt::Felt252Wrapper;
use mp_hashers::pedersen::PedersenHasher;
use mp_hashers::HasherT;
use starknet_api::api_core::{ChainId, ContractAddress, PatriciaKey};
use starknet_api::block::{BlockNumber, BlockTimestamp};
use starknet_api::hash::{StarkFelt, StarkHash};

use crate::Header;

fn generate_dummy_header() -> Vec<Felt252Wrapper> {
    vec![
        Felt252Wrapper::ONE,  // block_number
        Felt252Wrapper::ONE,  // global_state_root
        Felt252Wrapper::ONE,  // sequencer_address
        Felt252Wrapper::ONE,  // block_timestamp
        Felt252Wrapper::ONE,  // transaction_count
        Felt252Wrapper::ONE,  // transaction_commitment
        Felt252Wrapper::ONE,  // event_count
        Felt252Wrapper::ONE,  // event_commitment
        Felt252Wrapper::ZERO, // placeholder
        Felt252Wrapper::ZERO, // placeholder
        Felt252Wrapper::ONE,  // parent_block_hash
    ]
}

#[test]
fn test_header_hash() {
    let hash = <PedersenHasher as HasherT>::compute_hash_on_wrappers(&generate_dummy_header());

    let expected_hash =
        Felt252Wrapper::from_hex_be("0x001bef5f78bfd9122370a6bf9e3365b96362bef2bfd2b44b67707d8fbbf27bdc").unwrap();

    assert_eq!(hash, expected_hash);
}

#[test]
fn test_real_header_hash() {
    // Values taken from alpha-mainnet
    let block_number = 86000u32;
    let block_timestamp = 1687235884u32;
    let global_state_root =
        StarkHash::try_from("0x006727a7aae8c38618a179aeebccd6302c67ad5f8528894d1dde794e9ae0bbfa").unwrap();
    let parent_block_hash =
        StarkHash::try_from("0x045543088ce763aba7db8f6bfb33e33cc50af5c2ed5a26d38d5071c352a49c1d").unwrap();
    let sequencer_address = ContractAddress(PatriciaKey(
        StarkFelt::try_from("0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8").unwrap(),
    ));
    let transaction_count = 197u32;
    let transaction_commitment =
        StarkFelt::try_from("0x70369cef825889dc005916dba67332b71f270b7af563d0433cee3342dda527d").unwrap();
    let event_count = 1430u32;
    let event_commitment =
        StarkFelt::try_from("0x2043ba1ef46882ce1dbb17b501fffa4b71f87f618e8f394e9605959d92efdf6").unwrap();
    let protocol_version = 0u32;

    let header: &[Felt252Wrapper] = &[
        block_number.into(),
        global_state_root.into(),
        sequencer_address.into(),
        block_timestamp.into(),
        transaction_count.into(),
        transaction_commitment.into(),
        event_count.into(),
        event_commitment.into(),
        protocol_version.into(),
        Felt252Wrapper::ZERO,
        parent_block_hash.into(),
    ];

    let expected_hash =
        Felt252Wrapper::from_hex_be("0x001d126ca058c7e546d59cf4e10728e4b023ca0fb368e8abcabf0b5335f4487a").unwrap();
    let hash = <PedersenHasher as HasherT>::compute_hash_on_wrappers(header);

    assert_eq!(hash, expected_hash);
}

#[test]
fn test_to_block_context() {
    let sequencer_address = ContractAddress(PatriciaKey(StarkFelt::try_from("0xFF").unwrap()));
    // Create a block header.
    let block_header = Header { block_number: 1, block_timestamp: 1, sequencer_address, ..Default::default() };
    // Create a fee token address.
    let fee_token_address = ContractAddress(PatriciaKey(StarkFelt::try_from("AA").unwrap()));
    // Create a chain id.
    let chain_id = ChainId("0x1".to_string());
    // Try to serialize the block header.
    let block_context = block_header.into_block_context(fee_token_address, chain_id);
    // Check that the block context was serialized correctly.
    assert_eq!(block_context.block_number, BlockNumber(1));
    assert_eq!(block_context.block_timestamp, BlockTimestamp(1));
    assert_eq!(block_context.sequencer_address, sequencer_address);
    assert_eq!(block_context.fee_token_address, fee_token_address);
}
