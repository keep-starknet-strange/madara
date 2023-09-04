use frame_support::assert_ok;
use sp_runtime::DispatchError;

use super::mock::default_mock::*;
use super::mock::*;
use crate::tests::get_invoke_dummy;

#[test]
fn estimates_tx_fee_successfully() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let mut tx = get_invoke_dummy();
        tx.is_query = true;

        let (actual, overall) = Starknet::estimate_fee(tx).unwrap();
        assert!(actual > 0, "actual fee is missing");
        assert!(overall > 0, "overall fee is missing");
    });
}

#[test]
fn estimates_tx_fee_with_query_version() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let tx = get_invoke_dummy();

        let estimation_txn = Starknet::estimate_fee(tx);
        assert!(estimation_txn.is_err());
        assert!(matches!(
            estimation_txn.unwrap_err(),
            DispatchError::Other("Cannot estimate_fee with is_query = false")
        ));
    });
}

#[test]
fn estimate_does_not_add_to_pending() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let mut tx = get_invoke_dummy();
        tx.is_query = true;
        let pre_storage = Starknet::pending().len();

        assert_ok!(Starknet::estimate_fee(tx));

        assert!(pre_storage == Starknet::pending().len(), "estimate should not add a tx to pending");
    });
}
