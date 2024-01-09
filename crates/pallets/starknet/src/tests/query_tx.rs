use frame_support::{assert_err, assert_ok};
use mp_felt::Felt252Wrapper;
use mp_transactions::compute_hash::ComputeTransactionHash;
use mp_transactions::UserTransaction;

use super::mock::default_mock::*;
use super::mock::new_test_ext;
use crate::tests::utils::sign_message_hash;
use crate::tests::{get_invoke_argent_dummy, get_invoke_dummy, get_storage_read_write_dummy};
use crate::{Config, Error};

#[test]
fn estimates_tx_fee_successfully_no_validate() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let tx_1: mp_transactions::InvokeTransactionV1 = get_storage_read_write_dummy();
        let tx_1 = UserTransaction::Invoke(tx_1.into());

        let tx_2 = get_invoke_dummy(Felt252Wrapper::ONE);
        let tx_2 = UserTransaction::Invoke(tx_2.into());

        let txs = vec![tx_1, tx_2];

        let fees = Starknet::estimate_fee(txs).expect("estimate should not fail");

        let (actual, l1_gas_usage) = fees[0];
        assert!(actual > 0, "actual fee is missing");
        assert!(l1_gas_usage > 0, "this should be charged l1_gas as it store a value to storage");

        let (actual, l1_gas_usage) = fees[1];
        assert!(actual > 0, "actual fee is missing");
        assert!(l1_gas_usage == 0, "this should not be charged any l1_gas as it does not store nor send messages");
    });
}

#[test]
fn estimates_tx_fee_with_query_version() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let tx = get_invoke_dummy(Felt252Wrapper::ZERO);
        let pre_storage = Starknet::pending().len();
        let tx = UserTransaction::Invoke(tx.into());

        let tx_vec = vec![tx];

        assert_ok!(Starknet::estimate_fee(tx_vec));

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

        let tx_vec = vec![UserTransaction::Invoke(tx.clone().into())];

        // it should be valid for estimate calls
        assert_ok!(Starknet::estimate_fee(tx_vec));

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
        tx.offset_version = true;
        let tx_hash = tx.compute_hash::<<MockRuntime as Config>::SystemHash>(chain_id, true);
        tx.signature = sign_message_hash(tx_hash);

        let tx_vec = vec![UserTransaction::Invoke(tx.clone().into())];

        // it should be valid for estimate calls
        assert_ok!(Starknet::estimate_fee(tx_vec));

        // it should not be executable
        assert_err!(
            Starknet::invoke(RuntimeOrigin::none(), tx.clone().into()),
            Error::<MockRuntime>::TransactionExecutionFailed
        );
    });
}
