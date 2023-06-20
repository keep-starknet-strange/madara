use mp_starknet::sequencer_address::{DEFAULT_SEQUENCER_ADDRESS, SEQ_ADDR_STORAGE_KEY};

use super::mock::*;

pub const GOOD_SEQUENCER_ADDRESS: [u8; 32] =
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 222, 173];

pub const BAD_SEQUENCER_ADDRESS: [u8; 24] =
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 222, 173];

#[test]
fn sequencer_address_is_set_to_default_when_not_provided() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        basic_test_setup(0);
        assert_eq!(Starknet::sequencer_address(), DEFAULT_SEQUENCER_ADDRESS);
    });
}

#[test]
fn sequencer_address_is_set_to_default_when_provided_in_bad_format() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        basic_test_setup(0);
        sp_io::offchain_index::set(SEQ_ADDR_STORAGE_KEY, &BAD_SEQUENCER_ADDRESS);
        assert_eq!(Starknet::sequencer_address(), DEFAULT_SEQUENCER_ADDRESS);
    });
}

#[test]
fn sequencer_address_is_set_correctly() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        basic_test_setup(0);
        sp_io::offchain_index::set(SEQ_ADDR_STORAGE_KEY, &GOOD_SEQUENCER_ADDRESS);
        assert_eq!(Starknet::sequencer_address(), GOOD_SEQUENCER_ADDRESS);
    });
    ext.persist_offchain_overlay();
    let offchain_db = ext.offchain_db();
    assert_eq!(offchain_db.get(SEQ_ADDR_STORAGE_KEY), Some(GOOD_SEQUENCER_ADDRESS.to_vec()));
}

#[test]
fn sequencer_address_is_set_only_once_per_block() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        basic_test_setup(0);
        assert!(!Starknet::seq_addr_update());
        sp_io::offchain_index::set(SEQ_ADDR_STORAGE_KEY, &GOOD_SEQUENCER_ADDRESS);
        assert_eq!(Starknet::sequencer_address(), GOOD_SEQUENCER_ADDRESS);
        sp_io::offchain_index::set(SEQ_ADDR_STORAGE_KEY, &DEFAULT_SEQUENCER_ADDRESS);
        assert_eq!(Starknet::sequencer_address(), GOOD_SEQUENCER_ADDRESS);
    });
    ext.persist_offchain_overlay();
    let offchain_db = ext.offchain_db();
    assert_eq!(offchain_db.get(SEQ_ADDR_STORAGE_KEY), Some(DEFAULT_SEQUENCER_ADDRESS.to_vec()));
}

#[test]
fn sequencer_address_has_not_been_updated() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        basic_test_setup(0);
        sp_io::offchain_index::set(SEQ_ADDR_STORAGE_KEY, &GOOD_SEQUENCER_ADDRESS);
        assert_eq!(Starknet::sequencer_address(), GOOD_SEQUENCER_ADDRESS);
        run_to_block(1);
        assert!(!Starknet::seq_addr_update());
    });
}
