use std::num::NonZeroU128;

use frame_support::traits::Hooks;
use frame_support::{assert_err, assert_ok};
use mp_starknet_inherent::{L1GasPrices, StarknetInherentData, DEFAULT_SEQUENCER_ADDRESS, SEQ_ADDR_STORAGE_KEY};
use starknet_api::core::{ContractAddress, PatriciaKey};
use starknet_api::hash::StarkFelt;

use super::mock::default_mock::*;
use super::mock::*;

pub const GOOD_SEQUENCER_ADDRESS: [u8; 32] =
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 222, 175];

pub const BAD_SEQUENCER_ADDRESS: [u8; 24] =
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 222, 173];

fn get_dummy_l1_gas_price() -> L1GasPrices {
    unsafe {
        L1GasPrices {
            eth_l1_gas_price: NonZeroU128::new_unchecked(123),
            eth_l1_data_gas_price: NonZeroU128::new_unchecked(123),
            strk_l1_gas_price: NonZeroU128::new_unchecked(123),
            strk_l1_data_gas_price: NonZeroU128::new_unchecked(123),
            last_update_timestamp: 2,
        }
    }
}

#[test]
fn inherent_updates_storage() {
    let mut ext = new_test_ext::<MockRuntime>();
    ext.execute_with(|| {
        let none_origin = RuntimeOrigin::none();

        System::set_block_number(0);
        assert!(Starknet::inherent_update());
        Starknet::on_finalize(0);
        assert!(!Starknet::inherent_update());

        System::set_block_number(1);
        let l1_gas_price = get_dummy_l1_gas_price();
        assert_ok!(Starknet::set_starknet_inherent_data(
            none_origin,
            StarknetInherentData { sequencer_address: GOOD_SEQUENCER_ADDRESS, l1_gas_price: l1_gas_price.clone() }
        ));
        assert!(Starknet::inherent_update());
        assert_eq!(
            Starknet::sequencer_address(),
            ContractAddress(PatriciaKey(StarkFelt::new(GOOD_SEQUENCER_ADDRESS).unwrap()))
        );
        assert_eq!(Starknet::current_l1_gas_prices(), l1_gas_price);
    });
}

#[test]
fn on_finalize_hook_takes_storage_update() {
    let mut ext = new_test_ext::<MockRuntime>();
    ext.execute_with(|| {
        System::set_block_number(1);
        assert!(Starknet::inherent_update());
        Starknet::on_finalize(1);
        assert!(!Starknet::inherent_update());
    });
}

#[test]
#[should_panic]
fn inherent_updates_only_once_per_block() {
    let mut ext = new_test_ext::<MockRuntime>();
    ext.execute_with(|| {
        let none_origin = RuntimeOrigin::none();

        System::set_block_number(1);
        assert!(Starknet::inherent_update());
        Starknet::on_finalize(1);
        assert!(!Starknet::inherent_update());

        // setting block number to 2 as we ignore "already updated" check
        // for genesis blocks
        System::set_block_number(2);
        let l1_gas_price = get_dummy_l1_gas_price();

        // setting it first time works
        assert_ok!(Starknet::set_starknet_inherent_data(
            none_origin.clone(),
            StarknetInherentData { sequencer_address: DEFAULT_SEQUENCER_ADDRESS, l1_gas_price: l1_gas_price.clone() }
        ));
        assert_eq!(Starknet::current_l1_gas_prices(), l1_gas_price);

        let l1_gas_price_new = L1GasPrices { last_update_timestamp: 999, ..l1_gas_price };
        // setting it second time causes it to panic
        let _ = Starknet::set_starknet_inherent_data(
            none_origin,
            StarknetInherentData {
                sequencer_address: DEFAULT_SEQUENCER_ADDRESS,
                l1_gas_price: l1_gas_price_new.clone(),
            },
        );
    });
}
