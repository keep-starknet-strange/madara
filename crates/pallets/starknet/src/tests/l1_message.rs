use mp_felt::Felt252Wrapper;
use mp_transactions::HandleL1MessageTransaction;
use sp_runtime::traits::ValidateUnsigned;
use sp_runtime::transaction_validity::{TransactionSource, TransactionTag};
use starknet_api::api_core::ContractAddress;
use starknet_api::transaction::Fee;

use super::mock::default_mock::*;
use super::mock::*;

use frame_support::codec::Encode;

const VALID_TX_BUILDER_TAG: &str = "starknet";

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

        let expected_priority = u64::MAX - transaction.nonce;
        let expected_provide: (Felt252Wrapper, Felt252Wrapper) = (ContractAddress::default().into(), transaction.nonce.into());
		let expected_provide: TransactionTag = (VALID_TX_BUILDER_TAG, expected_provide).encode();
        let expected_longetivity = TransactionLongevity::get();
        let expected_propagate = true;

        let validate_result = Starknet::validate_unsigned(
            TransactionSource::InBlock,
            &crate::Call::consume_l1_message { transaction, paid_fee_on_l1: Fee(100) },
        );

        assert!(validate_result.is_ok());
        let validate_result = validate_result.unwrap();

		assert_eq!(validate_result.priority, expected_priority);
		assert_eq!(validate_result.requires, Vec::<TransactionTag>::new());
        assert_eq!(validate_result.provides, Vec::<TransactionTag>::from([expected_provide]));
		assert_eq!(validate_result.longevity, expected_longetivity);
		assert_eq!(validate_result.propagate, expected_propagate);
    });
}
