use std::num::NonZeroU128;

use assert_matches::assert_matches;
use blockifier::blockifier::block::GasPrices;
use mp_block::Header;
use sp_runtime::{Digest, DigestItem};

use super::*;

fn create_empty_block() -> StarknetBlock {
    StarknetBlock::try_new(
        Header {
            parent_block_hash: Default::default(),
            block_number: Default::default(),
            sequencer_address: Default::default(),
            block_timestamp: Default::default(),
            transaction_count: Default::default(),
            event_count: Default::default(),
            protocol_version: Default::default(),
            l1_gas_price: unsafe {
                GasPrices {
                    eth_l1_gas_price: NonZeroU128::new_unchecked(10),
                    strk_l1_gas_price: NonZeroU128::new_unchecked(10),
                    eth_l1_data_gas_price: NonZeroU128::new_unchecked(10),
                    strk_l1_data_gas_price: NonZeroU128::new_unchecked(10),
                }
            },
            extra_data: Default::default(),
        },
        Default::default(),
    )
    .unwrap()
}

#[test]
fn log_is_found() {
    let mut digest = Digest::default();
    let block = create_empty_block();

    digest.push(DigestItem::Consensus(MADARA_ENGINE_ID, Log::Block(block.clone()).encode()));

    assert!(ensure_log(&digest).is_ok());
}

#[test]
fn multiple_logs() {
    let mut digest = Digest::default();
    let block = create_empty_block();

    digest.push(DigestItem::Consensus(MADARA_ENGINE_ID, Log::Block(block.clone()).encode()));
    digest.push(DigestItem::Consensus(MADARA_ENGINE_ID, Log::Block(block).encode()));

    assert_matches!(ensure_log(&digest), Err(FindLogError::MultipleLogs));
    assert_matches!(find_log(&digest), Err(FindLogError::MultipleLogs));
    assert_matches!(find_starknet_block(&digest), Err(FindLogError::MultipleLogs));
}

#[test]
fn no_logs() {
    let digest = Digest::default();

    assert_matches!(ensure_log(&digest), Err(FindLogError::NotLog));
    assert_matches!(find_log(&digest), Err(FindLogError::NotLog));
    assert_matches!(find_starknet_block(&digest), Err(FindLogError::NotLog));
}

#[test]
fn other_consensus_engine_id() {
    let mut digest = Digest::default();
    let block = create_empty_block();

    digest.push(DigestItem::Consensus([b'o', b't', b'h', b'r'], Log::Block(block).encode()));

    assert_matches!(ensure_log(&digest), Err(FindLogError::NotLog));
    assert_matches!(find_log(&digest), Err(FindLogError::NotLog));
    assert_matches!(find_starknet_block(&digest), Err(FindLogError::NotLog));
}
