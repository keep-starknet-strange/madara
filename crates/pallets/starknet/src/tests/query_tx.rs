use blockifier::transaction::account_transaction::AccountTransaction;
use frame_support::{assert_err, assert_ok};
use mp_simulations::SimulationFlags;
use mp_transactions::compute_hash::ComputeTransactionHash;
use starknet_api::core::Nonce;
use starknet_api::hash::StarkFelt;

use super::mock::default_mock::*;
use super::mock::new_test_ext;
use crate::tests::utils::sign_message_hash;
use crate::tests::{get_invoke_argent_dummy, get_invoke_dummy, get_storage_read_write_dummy};
use crate::Error;

#[test]
fn estimates_tx_fee_successfully_no_validate() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let chain_id = Starknet::chain_id();
        let tx_1 = get_storage_read_write_dummy(chain_id);
        let tx_1 = AccountTransaction::Invoke(tx_1);

        let tx_2 = get_invoke_dummy(chain_id, Nonce(StarkFelt::ONE));
        let tx_2 = AccountTransaction::Invoke(tx_2);

        let txs = vec![tx_1, tx_2];

        let fees = Starknet::estimate_fee(txs, &Default::default()).expect("estimate should not fail").unwrap();

        let (actual, l1_gas_usage) = fees[0];
        assert!(actual > 0, "actual fee is missing");
        assert!(l1_gas_usage == 0, "using blobstream we shouldn't pay l1 fee");

        let (actual, l1_gas_usage) = fees[1];
        assert!(actual > 0, "actual fee is missing");
        assert!(l1_gas_usage == 0, "using blobstream we shouldn't pay l1 fee");
    });
}

#[test]
fn estimates_tx_fee_with_query_version() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let transaction = get_invoke_dummy(Starknet::chain_id(), Nonce(StarkFelt::ZERO));
        let pre_storage = Starknet::pending().len();
        let tx = AccountTransaction::Invoke(transaction);

        let tx_vec = vec![tx];

        assert_ok!(Starknet::estimate_fee(tx_vec, &Default::default()));

        assert!(pre_storage == Starknet::pending().len(), "estimate should not add a tx to pending");
    });
}

#[test]
fn executable_tx_should_be_estimable_and_executable() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let mut transaction = get_invoke_argent_dummy(Starknet::chain_id());
        if let starknet_api::transaction::InvokeTransaction::V1(tx) = &mut transaction.tx {
            tx.signature = sign_message_hash(transaction.tx_hash);
        };

        let tx_vec = vec![AccountTransaction::Invoke(transaction.clone())];

        // it should be valid for estimate calls
        assert_ok!(Starknet::estimate_fee(tx_vec, &Default::default()));

        // it should be executable
        assert_ok!(Starknet::invoke(RuntimeOrigin::none(), transaction.clone()));
    });
}

#[test]
fn query_tx_should_not_be_executable() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let mut transaction = get_invoke_argent_dummy(Starknet::chain_id());
        transaction.only_query = true;
        transaction.tx_hash = transaction.tx.compute_hash(Starknet::chain_id(), true);
        if let starknet_api::transaction::InvokeTransaction::V1(tx) = &mut transaction.tx {
            tx.signature = sign_message_hash(transaction.tx_hash);
        };

        let tx_vec = vec![AccountTransaction::Invoke(transaction.clone())];

        // it should be valid for estimate calls
        assert_ok!(Starknet::estimate_fee(tx_vec, &Default::default()));

        // it should not be executable
        assert_err!(
            Starknet::invoke(RuntimeOrigin::none(), transaction.clone()),
            Error::<MockRuntime>::QueryTransactionCannotBeExecuted
        );
    });
}
