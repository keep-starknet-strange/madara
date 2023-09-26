use core::convert::TryFrom;

use mp_felt::Felt252Wrapper;
use mp_hashers::pedersen::PedersenHasher;
use sp_core::U256;
use starknet_api::api_core::{ChainId, ContractAddress, PatriciaKey};
use starknet_api::block::{BlockNumber, BlockTimestamp};
use starknet_api::hash::{StarkFelt, StarkHash};

use crate::Header;

fn generate_dummy_header() -> Header {
    Header::new(
        StarkFelt::from(1u128),
        1,
        StarkFelt::from(2u128),
        ContractAddress::default(),
        42,
        0,
        StarkFelt::from(3u128),
        0,
        StarkFelt::from(4u128),
        1,
        Some(U256::from(3)),
    )
}

#[test]
fn test_header_hash() {
    let header = generate_dummy_header();

    let expected_hash =
        Felt252Wrapper::from_hex_be("0x029da584545c7f3ebdb0c6aca74f0fba99156b1e31e9524c70b42776e50efda6").unwrap();

    assert_eq!(header.hash::<PedersenHasher>(), expected_hash);
}

#[test]
fn test_real_header_hash() {
    // Values taken from alpha-mainnet

    let block_number = 86000;
    let block_timestamp = 1687235884;
    let global_state_root =
        StarkHash::try_from("0x006727a7aae8c38618a179aeebccd6302c67ad5f8528894d1dde794e9ae0bbfa").unwrap();
    let parent_block_hash =
        StarkHash::try_from("0x045543088ce763aba7db8f6bfb33e33cc50af5c2ed5a26d38d5071c352a49c1d").unwrap();
    let sequencer_address = ContractAddress(PatriciaKey(
        StarkFelt::try_from("0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8").unwrap(),
    ));
    let transaction_count = 197;
    let transaction_commitment =
        StarkFelt::try_from("0x70369cef825889dc005916dba67332b71f270b7af563d0433cee3342dda527d").unwrap();
    let event_count = 1430;
    let event_commitment =
        StarkFelt::try_from("0x2043ba1ef46882ce1dbb17b501fffa4b71f87f618e8f394e9605959d92efdf6").unwrap();
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

    assert_eq!(header.hash::<PedersenHasher>(), expected_hash);
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
