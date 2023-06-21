use frame_support::assert_ok;
use frame_support::traits::Hooks;
use mp_starknet::sequencer_address::{DEFAULT_SEQUENCER_ADDRESS, SEQ_ADDR_STORAGE_KEY};
use starknet_crypto::FieldElement;

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
        assert_eq!(
            Starknet::sequencer_address(),
            FieldElement::from_byte_slice_be(&GOOD_SEQUENCER_ADDRESS).unwrap().into()
        );
    });
}

#[test]
fn sequencer_address_is_set_to_default_when_provided_in_bad_format() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        basic_test_setup(0);
        sp_io::offchain_index::set(SEQ_ADDR_STORAGE_KEY, &BAD_SEQUENCER_ADDRESS);
        assert_eq!(
            Starknet::sequencer_address(),
            FieldElement::from_byte_slice_be(&DEFAULT_SEQUENCER_ADDRESS).unwrap().into()
        );
    });
}

#[test]
fn sequencer_address_is_set_correctly() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        basic_test_setup(0);
        sp_io::offchain_index::set(SEQ_ADDR_STORAGE_KEY, &GOOD_SEQUENCER_ADDRESS);
        assert_eq!(
            Starknet::sequencer_address(),
            FieldElement::from_byte_slice_be(&GOOD_SEQUENCER_ADDRESS).unwrap().into()
        );
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
        assert_eq!(
            Starknet::sequencer_address(),
            FieldElement::from_byte_slice_be(&GOOD_SEQUENCER_ADDRESS).unwrap().into()
        );
        sp_io::offchain_index::set(SEQ_ADDR_STORAGE_KEY, &DEFAULT_SEQUENCER_ADDRESS);
        assert_eq!(
            Starknet::sequencer_address(),
            FieldElement::from_byte_slice_be(&GOOD_SEQUENCER_ADDRESS).unwrap().into()
        );
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
        assert_eq!(
            Starknet::sequencer_address(),
            FieldElement::from_byte_slice_be(&GOOD_SEQUENCER_ADDRESS).unwrap().into()
        );
        run_to_block(1);
        assert!(!Starknet::seq_addr_update());
    });
}

#[test]
fn on_finalize_hook_takes_storage_update() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        System::set_block_number(1);
        assert!(Starknet::seq_addr_update());
        Starknet::on_finalize(1);
        assert!(!Starknet::seq_addr_update());
    });
}

#[test]
fn inherent_updates_storage() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        let none_origin = RuntimeOrigin::none();

        System::set_block_number(0);
        assert!(Starknet::seq_addr_update());
        Starknet::on_finalize(0);
        assert!(!Starknet::seq_addr_update());

        System::set_block_number(1);
        assert_ok!(Starknet::set_sequencer_address(none_origin, DEFAULT_SEQUENCER_ADDRESS));
        assert!(Starknet::seq_addr_update());
    });
}
