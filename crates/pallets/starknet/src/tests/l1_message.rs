use frame_support::{assert_err, assert_ok};
use mp_felt::Felt252Wrapper;
use mp_transactions::HandleL1MessageTransaction;
use sp_runtime::traits::ValidateUnsigned;
use sp_runtime::transaction_validity::{TransactionSource, TransactionValidityError};
use starknet_api::api_core::{ContractAddress, Nonce, PatriciaKey};
use starknet_api::hash::StarkFelt;
use starknet_api::state::StorageKey;
use starknet_api::transaction::Fee;

use super::mock::default_mock::*;
use super::mock::*;
use crate::{Call, Error, InvalidTransaction, L1Messages};

#[test]
fn verify_tx_validity() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let transaction = HandleL1MessageTransaction {
            nonce: Default::default(),
            contract_address: Default::default(),
            entry_point_selector: Default::default(),
            calldata: Default::default(),
        };

        let expected_priority = u64::MAX;
        let expected_longetivity = TransactionLongevity::get();
        let expected_propagate = true;

        let validate_result = Starknet::validate_unsigned(
            TransactionSource::InBlock,
            &Call::consume_l1_message { transaction, paid_fee_on_l1: Fee(100) },
        );

        assert!(validate_result.is_ok());
        let validate_result = validate_result.unwrap();

        assert_eq!(validate_result.priority, expected_priority);
        assert_eq!(validate_result.longevity, expected_longetivity);
        assert_eq!(validate_result.propagate, expected_propagate);
    });
}

#[test]
fn should_reject_used_nonce() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let nonce: u64 = 1;

        let transaction = HandleL1MessageTransaction {
            nonce,
            contract_address: Default::default(),
            entry_point_selector: Default::default(),
            calldata: Default::default(),
        };

        let tx_source = TransactionSource::InBlock;
        let call = Call::consume_l1_message { transaction, paid_fee_on_l1: Fee(100) };

        assert!(Starknet::validate_unsigned(tx_source, &call).is_ok());

        L1Messages::<MockRuntime>::mutate(|nonces| nonces.insert(Nonce(StarkFelt::from(nonce))));

        assert_eq!(
            Starknet::validate_unsigned(tx_source, &call),
            Err(TransactionValidityError::Invalid(InvalidTransaction::Stale))
        );
    });
}

#[test]
fn should_reject_zero_fee() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let nonce: u64 = 1;

        let transaction = HandleL1MessageTransaction {
            nonce,
            contract_address: Default::default(),
            entry_point_selector: Default::default(),
            calldata: Default::default(),
        };

        let tx_source = TransactionSource::InBlock;
        let call = Call::consume_l1_message { transaction, paid_fee_on_l1: Fee(0) };

        assert_eq!(
            Starknet::validate_unsigned(tx_source, &call),
            Err(TransactionValidityError::Invalid(InvalidTransaction::Payment))
        );
    });
}

#[test]
fn work() {
    // Execute `test_l1_handler_store_under_caller_address()`
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let contract_address =
            Felt252Wrapper::from_hex_be("0x024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap();
        let from_address = Felt252Wrapper::from_hex_be("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").unwrap();

        let transaction = HandleL1MessageTransaction {
            nonce: 1,
            contract_address,
            entry_point_selector: Felt252Wrapper::from_hex_be(
                "0x014093c40d95d0a3641c087f7d48d55160e1a58bc7c07b0d2323efeeb3087269", // test_l1_handler_store_under_caller_address
            )
            .unwrap(),
            calldata: vec![
                from_address,
                Felt252Wrapper::from_hex_be("0x1").unwrap(), // value
            ],
        };
        assert_ok!(Starknet::consume_l1_message(RuntimeOrigin::none(), transaction, Fee(1)));

        let storage_key = (
            ContractAddress(PatriciaKey(StarkFelt::from(contract_address))),
            StorageKey(PatriciaKey(StarkFelt::from(from_address))),
        );
        assert_eq!(Starknet::storage(storage_key), StarkFelt::from(1u128));
    });
}

#[test]
fn fail_if_no_fee() {
    // Execute `test_l1_handler_store_under_caller_address()`
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let contract_address =
            Felt252Wrapper::from_hex_be("0x024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap();
        let from_address = Felt252Wrapper::from_hex_be("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").unwrap();

        let transaction = HandleL1MessageTransaction {
            nonce: 1,
            contract_address,
            entry_point_selector: Felt252Wrapper::from_hex_be(
                "0x014093c40d95d0a3641c087f7d48d55160e1a58bc7c07b0d2323efeeb3087269",
            )
            .unwrap(),
            calldata: vec![
                from_address,
                Felt252Wrapper::from_hex_be("0x1").unwrap(), // value
            ],
        };
        assert_err!(
            Starknet::consume_l1_message(RuntimeOrigin::none(), transaction, Fee(0)),
            Error::<MockRuntime>::TransactionExecutionFailed
        );

        let storage_key = (
            ContractAddress(PatriciaKey(StarkFelt::from(contract_address))),
            StorageKey(PatriciaKey(StarkFelt::from(from_address))),
        );
        assert_eq!(Starknet::storage(storage_key), StarkFelt::from(0u128));
    });
}
