use frame_support::assert_err;
use mp_felt::Felt252Wrapper;
use mp_transactions::{DeclareTransactionV1, HandleL1MessageTransaction};
use sp_runtime::traits::ValidateUnsigned;
use sp_runtime::transaction_validity::TransactionSource;
use starknet_api::transaction::Fee;

use super::mock::default_mock::*;
use super::mock::*;
use super::utils::get_contract_class;
use crate::Error;

#[test]
fn given_contract_l1_message_fails_sender_not_deployed() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();

        // Wrong address (not deployed)
        let contract_address =
            Felt252Wrapper::from_hex_be("0x03e437FB56Bb213f5708Fcd6966502070e276c093ec271aA33433b89E21fd31f").unwrap();

        let erc20_class = get_contract_class("ERC20.json", 0);

        let transaction = DeclareTransactionV1 {
            sender_address: contract_address,
            nonce: Default::default(),
            signature: Default::default(),
            max_fee: Default::default(),
            class_hash: Default::default(),
        };

        assert_err!(
            Starknet::declare(none_origin, transaction.into(), erc20_class),
            Error::<MockRuntime>::AccountNotDeployed
        );
    })
}

#[test]
#[ignore = "l1 handler validation not implemented yet"]
fn verify_tx_longevity() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let transaction = HandleL1MessageTransaction {
            nonce: Default::default(),
            contract_address: Default::default(),
            entry_point_selector: Default::default(),
            calldata: Default::default(),
        };

        let validate_result = Starknet::validate_unsigned(
            TransactionSource::InBlock,
            &crate::Call::consume_l1_message { transaction, paid_fee_on_l1: Fee(100) },
        );

        assert!(validate_result.unwrap().longevity == TransactionLongevity::get());
    });
}
