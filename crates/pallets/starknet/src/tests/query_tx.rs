use frame_support::{assert_err, assert_ok};
use mp_felt::Felt252Wrapper;
use mp_transactions::compute_hash::ComputeTransactionHash;
use mp_transactions::UserTransaction;

use super::mock::default_mock::*;
use super::mock::*;
use crate::tests::utils::sign_message_hash;
use crate::tests::{get_invoke_argent_dummy, get_invoke_dummy, get_storage_read_write_dummy};
use crate::{Config, Error};

#[test]
fn estimates_tx_fee_successfully_no_validate() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let tx = get_invoke_dummy(Felt252Wrapper::ZERO);
        let tx = UserTransaction::Invoke(tx.into());

        let (actual, l1_gas_usage) = Starknet::estimate_fee(tx, true).unwrap();
        assert!(actual > 0, "actual fee is missing");
        assert!(l1_gas_usage == 0, "this should not be charged any l1_gas as it does not store nor send messages");

        let tx = get_storage_read_write_dummy();
        let tx = UserTransaction::Invoke(tx.into());

        let (actual, l1_gas_usage) = Starknet::estimate_fee(tx, true).unwrap();
        assert!(actual > 0, "actual fee is missing");
        assert!(l1_gas_usage > 0, "this should be charged l1_gas as it store a value to storage");
    });
}

#[test]
fn estimates_tx_fee_with_query_version() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let tx = get_invoke_dummy(Felt252Wrapper::ZERO);
        let pre_storage = Starknet::pending().len();
        let tx = UserTransaction::Invoke(tx.into());

        assert_ok!(Starknet::estimate_fee(tx, true));

        assert!(pre_storage == Starknet::pending().len(), "estimate should not add a tx to pending");
    });
}

#[test]
fn executable_tx_should_not_be_estimable() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let chain_id = Starknet::chain_id();
        let mut tx = get_invoke_argent_dummy();
        let tx_hash = tx.compute_hash::<<MockRuntime as Config>::SystemHash>(chain_id, false);
        tx.signature = sign_message_hash(tx_hash);

        // it should not be valid for estimate calls
        assert_err!(
            Starknet::estimate_fee(UserTransaction::Invoke(tx.clone().into()), true),
            Error::<MockRuntime>::TransactionExecutionFailed
        );

        // it should be executable
        assert_ok!(Starknet::invoke(RuntimeOrigin::none(), tx.clone().into()));
    });
}

#[test]
fn query_tx_should_not_be_executable() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let chain_id = Starknet::chain_id();
        let mut tx = get_invoke_argent_dummy();
        let tx_hash = tx.compute_hash::<<MockRuntime as Config>::SystemHash>(chain_id, true);
        tx.signature = sign_message_hash(tx_hash);

        // it should be valid for estimate calls
        assert_ok!(Starknet::estimate_fee(UserTransaction::Invoke(tx.clone().into()), true),);

        // it should not be executable
        assert_err!(
            Starknet::invoke(RuntimeOrigin::none(), tx.clone().into()),
            Error::<MockRuntime>::TransactionExecutionFailed
        );
    });
}
