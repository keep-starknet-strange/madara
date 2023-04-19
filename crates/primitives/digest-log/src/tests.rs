use assert_matches::assert_matches;
use sp_runtime::{Digest, DigestItem};

use super::*;

#[test]
fn log_is_found() {
    let mut digest = Digest::default();
    let block = StarknetBlock::default();

    digest.push(DigestItem::Consensus(MADARA_ENGINE_ID, Log::Block(block.clone()).encode()));

    assert!(ensure_log(&digest).is_ok());
    assert_eq!(find_log(&digest).unwrap(), Log::Block(block.clone()));
    assert_eq!(find_starknet_block(&digest).unwrap(), block);
}

#[test]
fn multiple_logs() {
    let mut digest = Digest::default();
    let block = StarknetBlock::default();

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
    let block = StarknetBlock::default();

    digest.push(DigestItem::Consensus([b'o', b't', b'h', b'r'], Log::Block(block).encode()));

    assert_matches!(ensure_log(&digest), Err(FindLogError::NotLog));
    assert_matches!(find_log(&digest), Err(FindLogError::NotLog));
    assert_matches!(find_starknet_block(&digest), Err(FindLogError::NotLog));
}
