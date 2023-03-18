use core::str::FromStr;

use frame_support::assert_ok;
use kp_starknet::{block::wrapper::header::Header, transaction::Transaction};
use sp_core::{H256, U256};

use crate::mock::*;

#[test]
fn given_normal_conditions_when_deploy_sierra_program_then_it_works() {
    new_test_ext().execute_with(|| {
        let deployer_account = 1;
        let deployer_origin = RuntimeOrigin::signed(deployer_account);
        // Go past genesis block so events get deposited
        System::set_block_number(1);
        // Dispatch a signed extrinsic.
        assert_ok!(Starknet::ping(deployer_origin));
    });
}

#[test]
fn given_normal_conditions_when_current_block_then_returns_correct_block() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let current_block = Starknet::current_block();

        let expected_current_block = Header {
            block_timestamp: 12_000,
            block_number: U256::from(2),
            parent_block_hash: H256::from_str("0x1c2b97b7b9ea91c2cde45bfb115058628c2e1c7aa3fecb51a0cdaf256dc8a310")
                .unwrap(),
            transaction_count: 1,
            // This expected value has been computed in the sequencer test (commitment on a tx hash 0 without
            // signature).
            transaction_commitment: H256::from_str(
                "0x039050b107da7374213fffb38becd5f2d76e51ffa0734bf5c7f8f0477a6f2c22",
            )
            .unwrap(),
            event_count: 2,
            event_commitment: H256::from_str("0x03ebee479332edbeecca7dee501cb507c69d51e0df116d28ae84cd2671dfef02")
                .unwrap(),
            ..Header::default()
        };

        assert!(current_block.is_some());
        pretty_assertions::assert_eq!(current_block.unwrap().header, expected_current_block)
    });
}

#[test]
fn given_hardcoded_contract_run_invoke_tx_then_it_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();

		let transaction = Transaction {
			version: U256::from(1),
			..Transaction::default()
		};

		assert_ok!(Starknet::add_invoke_transaction(none_origin, transaction));
	});
}
