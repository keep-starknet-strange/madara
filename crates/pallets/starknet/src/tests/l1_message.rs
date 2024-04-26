use std::sync::Arc;

use frame_support::{assert_err, assert_ok};
use mp_felt::Felt252Wrapper;
use mp_transactions::compute_hash::ComputeTransactionHash;
use sp_runtime::traits::ValidateUnsigned;
use sp_runtime::transaction_validity::{TransactionSource, TransactionValidityError};
use starknet_api::core::{ContractAddress, EntryPointSelector, Nonce, PatriciaKey};
use starknet_api::hash::StarkFelt;
use starknet_api::state::StorageKey;
use starknet_api::transaction::{Calldata, Fee, TransactionVersion};

use super::mock::default_mock::*;
use super::mock::*;
use crate::tests::mock::setup_mock::fees_disabled_mock::TransactionLongevity;
use crate::{Call, Error, InvalidTransaction, L1Messages};

fn create_handle_l1_message_transaction(
    chain_id: Felt252Wrapper,
    nonce: Nonce,
    paid_fee_on_l1: Fee,
) -> blockifier::transaction::transactions::L1HandlerTransaction {
    let contract_address = ContractAddress(PatriciaKey(
        StarkFelt::try_from("0x024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap(),
    ));
    let from_address =
        ContractAddress(PatriciaKey(StarkFelt::try_from("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").unwrap()));

    let tx = starknet_api::transaction::L1HandlerTransaction {
        nonce,
        contract_address,
        entry_point_selector: EntryPointSelector(
            // test_l1_handler_store_under_caller_address
            StarkFelt::try_from("0x014093c40d95d0a3641c087f7d48d55160e1a58bc7c07b0d2323efeeb3087269").unwrap(),
        ),
        calldata: Calldata(Arc::new(vec![
            from_address.0.0,
            StarkFelt::ONE, // value
        ])),
        version: TransactionVersion(StarkFelt::ZERO),
    };
    let tx_hash = tx.compute_hash(chain_id, false);

    blockifier::transaction::transactions::L1HandlerTransaction { tx, tx_hash, paid_fee_on_l1 }
}

#[test]
fn verify_tx_validity() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let transaction = create_handle_l1_message_transaction(Starknet::chain_id(), Nonce::default(), Fee(1));

        let expected_priority = u64::MAX;
        let expected_longetivity = TransactionLongevity::get();
        let expected_propagate = true;

        let validate_result =
            Starknet::validate_unsigned(TransactionSource::InBlock, &Call::consume_l1_message { transaction });

        assert!(validate_result.is_ok());
        let validate_result = validate_result.unwrap();

        assert_eq!(validate_result.priority, expected_priority);
        assert_eq!(validate_result.longevity, expected_longetivity);
        assert_eq!(validate_result.propagate, expected_propagate);
    });
}

#[test]
fn validate_should_reject_used_nonce() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let nonce = Nonce(StarkFelt::ONE);
        let transaction = create_handle_l1_message_transaction(Starknet::chain_id(), nonce, Fee(1));

        let tx_source = TransactionSource::InBlock;
        let call = Call::consume_l1_message { transaction };

        assert!(Starknet::validate_unsigned(tx_source, &call).is_ok());

        L1Messages::<MockRuntime>::mutate(|nonces| nonces.insert(nonce));

        assert_eq!(
            Starknet::validate_unsigned(tx_source, &call),
            Err(TransactionValidityError::Invalid(InvalidTransaction::Stale))
        );
    });
}

#[test]
fn work() {
    // Execute `test_l1_handler_store_under_caller_address()`
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let transaction = create_handle_l1_message_transaction(Starknet::chain_id(), Nonce(StarkFelt::ONE), Fee(1));

        assert_ok!(Starknet::consume_l1_message(RuntimeOrigin::none(), transaction.clone()));

        let contract_address = transaction.tx.contract_address;
        let from_address = transaction.tx.calldata.0.first().unwrap();
        let storage_key = (contract_address, StorageKey(PatriciaKey(*from_address)));
        assert_eq!(Starknet::storage(storage_key), StarkFelt::from(1u128));
    });
}

#[test]
fn fail_if_no_fee_paid() {
    // Execute `test_l1_handler_store_under_caller_address()`
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let transaction = create_handle_l1_message_transaction(Starknet::chain_id(), Nonce(StarkFelt::ONE), Fee(0));

        // Validate fails
        assert_eq!(
            Starknet::validate_unsigned(
                TransactionSource::InBlock,
                &Call::consume_l1_message { transaction: transaction.clone() }
            ),
            Err(TransactionValidityError::Invalid(InvalidTransaction::Payment))
        );
        // Execution fails
        assert_err!(
            Starknet::consume_l1_message(RuntimeOrigin::none(), transaction.clone()),
            Error::<MockRuntime>::TransactionExecutionFailed
        );

        // Storage unaltered
        let contract_address = transaction.tx.contract_address;
        let from_address = transaction.tx.calldata.0.first().unwrap();
        let storage_key = (contract_address, StorageKey(PatriciaKey(*from_address)));
        assert_eq!(Starknet::storage(storage_key), StarkFelt::from(0u128));
    });
}
