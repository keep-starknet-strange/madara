use frame_support::assert_err;
use mp_starknet::execution::types::Felt252Wrapper;
use mp_starknet::transaction::types::{DeclareTransaction, Transaction, TxType};
use sp_runtime::traits::ValidateUnsigned;
use sp_runtime::transaction_validity::TransactionSource;

use super::mock::*;
use super::utils::get_contract_class;
use crate::Error;

#[test]
fn given_contract_l1_message_fails_sender_not_deployed() {
    new_test_ext().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();

        // Wrong address (not deployed)
        let contract_address =
            Felt252Wrapper::from_hex_be("0x03e437FB56Bb213f5708Fcd6966502070e276c093ec271aA33433b89E21fd31f").unwrap();

        let erc20_class = get_contract_class("ERC20.json");

        let transaction = DeclareTransaction {
            sender_address: contract_address,
            contract_class: erc20_class,
            version: Default::default(),
            compiled_class_hash: Default::default(),
            nonce: Default::default(),
            signature: Default::default(),
            max_fee: Default::default(),
        };

        assert_err!(Starknet::declare(none_origin, transaction), Error::<MockRuntime>::AccountNotDeployed);
    })
}

#[test]
fn test_verify_tx_longevity() {
    new_test_ext().execute_with(|| {
        basic_test_setup(2);

        let transaction = Transaction { tx_type: TxType::L1Handler, ..Transaction::default() };

        let validate_result =
            Starknet::validate_unsigned(TransactionSource::InBlock, &crate::Call::consume_l1_message { transaction });

        assert!(validate_result.unwrap().longevity == TransactionLongevity::get());
    });
}
