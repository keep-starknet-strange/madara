use core::str::FromStr;

use frame_support::{assert_err, assert_ok, bounded_vec};
use hex::FromHex;
use mp_starknet::transaction::types::EventWrapper;
use sp_core::{H256, U256};
use sp_runtime::DispatchError;

use super::mock::*;
use crate::{Event, SEQUENCER_ADDRESS};

#[test]
#[ignore = "Fees are yet to be reimplemented"]
fn given_balance_on_account_then_transfer_fees_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let from = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 15];
        let _to = Starknet::current_block().header().sequencer_address;
        let amount = 100;
        let token_address = Starknet::fee_token_address();

        // assert_ok!(Starknet::transfer_fees(from, to, amount));
        // Check that balance is deducted from sn account
        assert_eq!(
            Starknet::storage((
                token_address, // Fee token address
                // pedersen(sn_keccak(b"ERC20_balances"), 0x0F) which is the key in the starknet contract for
                // ERC20_balances(0x0F).low
                H256::from_str("0x078e4fa4db2b6f3c7a9ece31571d47ac0e853975f90059f7c9df88df974d9093").unwrap(),
            )),
            U256::from(u128::MAX) - amount
        );
        assert_eq!(
            Starknet::storage((
                token_address, // Fee token address
                // pedersen(sn_keccak(b"ERC20_balances"), 0x0F) + 1 which is the key in the starknet contract for
                // ERC20_balances(0x0F).high
                H256::from_str("0x078e4fa4db2b6f3c7a9ece31571d47ac0e853975f90059f7c9df88df974d9094").unwrap(),
            )),
            U256::from(u128::MAX)
        );
        assert_eq!(
            Starknet::storage((
                token_address, // Fee token address
                // pedersen(sn_keccak(b"ERC20_balances"), 0x02) which is the key in the starknet contract for
                // ERC20_balances(0x0F).low
                H256::from_str("0x01d8bbc4f93f5ab9858f6c0c0de2769599fb97511503d5bf2872ef6846f2146f").unwrap(),
            )),
            U256::from(amount)
        );
        assert_eq!(
            Starknet::storage((
                token_address, // Fee token address
                // pedersen(sn_keccak(b"ERC20_balances"), 0x02) + 1 which is the key in the starknet contract for
                // ERC20_balances(0x0F).high
                H256::from_str("0x01d8bbc4f93f5ab9858f6c0c0de2769599fb97511503d5bf2872ef6846f21470").unwrap(),
            )),
            U256::zero()
        );
        System::assert_last_event(
            Event::StarknetEvent(EventWrapper {
                keys: bounded_vec!(
                    H256::from_str("0x0099cd8bde557814842a3121e8ddfd433a539b8c9f14bf31ebf108d12e6196e9").unwrap()
                ),
                data: bounded_vec!(
                    // From
                    H256::from_slice(&from),
                    // To
                    H256::from_slice(&SEQUENCER_ADDRESS),
                    // Amount low
                    H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000064").unwrap(),
                    // Amount High
                    H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000000").unwrap(),
                ),
                from_address: [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 170,
                ],
            })
            .into(),
        );
    })
}
#[test]
#[ignore = "Fees are yet to be reimplemented"]
fn given_no_balance_on_account_then_transfer_fees_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let _from = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
        let _to = Starknet::current_block().header().sequencer_address;
        let _amount = 100;

        // assert_err!(Starknet::transfer_fees(from, to, amount), Invalid(Payment));
        // Check that balance is not deducted from sn account
        assert_eq!(
            Starknet::storage((
                <[u8; 32]>::from_hex("00000000000000000000000000000000000000000000000000000000000000AA").unwrap(), // Fee token address
                // pedersen(sn_keccak(b"ERC20_balances"), 0x01) which is the key in the starknet contract for
                // ERC20_balances(0x0F).low
                H256::from_str("0x078e4fa4db2b6f3c7a9ece31571d47ac0e853975f90059f7c9df88df974d9093").unwrap(),
            )),
            U256::from(u128::MAX)
        );
        assert_eq!(
            Starknet::storage((
                <[u8; 32]>::from_hex("00000000000000000000000000000000000000000000000000000000000000AA").unwrap(), // Fee token address
                // pedersen(sn_keccak(b"ERC20_balances"), 0x01) + 1 which is the key in the starknet contract for
                // ERC20_balances(0x0F).high
                H256::from_str("0x078e4fa4db2b6f3c7a9ece31571d47ac0e853975f90059f7c9df88df974d9094").unwrap(),
            )),
            U256::from(u128::MAX)
        )
    })
}

#[test]
#[ignore = "Fees are yet to be reimplemented"]
fn given_root_when_set_fee_token_address_then_fee_token_address_is_updated() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let root_origin = RuntimeOrigin::root();
        let current_fee_token_address = Starknet::fee_token_address();
        let new_fee_token_address =
            <[u8; 32]>::from_hex("00000000000000000000000000000000000000000000000000000000000000ff").unwrap();

        assert_ok!(Starknet::set_fee_token_address(root_origin, new_fee_token_address));
        System::assert_last_event(
            Event::FeeTokenAddressChanged { old_fee_token_address: current_fee_token_address, new_fee_token_address }
                .into(),
        );
    })
}

#[test]
#[ignore = "Fees are yet to be reimplemented"]
fn given_non_root_when_set_fee_token_address_then_it_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let non_root_origin = RuntimeOrigin::signed(1);
        let new_fee_token_address =
            <[u8; 32]>::from_hex("00000000000000000000000000000000000000000000000000000000000000ff").unwrap();
        assert_err!(Starknet::set_fee_token_address(non_root_origin, new_fee_token_address), DispatchError::BadOrigin);
    })
}
