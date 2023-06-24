use frame_support::assert_ok;
use mp_starknet::starknet_serde::transaction_from_json;

use super::mock::*;

#[test]
fn estimates_tx_fee_successfully() {
    new_test_ext().execute_with(|| {
        basic_test_setup(2);

        let json_content: &str = include_str!("../../../../../resources/transactions/invoke.json");
        let tx = transaction_from_json(json_content, &[]).expect("Failed to create Transaction from JSON");

        let (actual, overall) = Starknet::estimate_fee(tx).unwrap();
        assert!(actual > 0, "actual fee is missing");
        assert!(overall > 0, "overall fee is missing");
    });
}

#[test]
fn estimate_does_not_add_to_pending() {
    new_test_ext().execute_with(|| {
        basic_test_setup(2);

        let json_content: &str = include_str!("../../../../../resources/transactions/invoke.json");
        let tx = transaction_from_json(json_content, &[]).expect("Failed to create Transaction from JSON");

        let pre_storage = Starknet::pending().len();

        assert_ok!(Starknet::estimate_fee(tx));

        assert!(pre_storage == Starknet::pending().len(), "estimate should not add a tx to pending");
    });
}
