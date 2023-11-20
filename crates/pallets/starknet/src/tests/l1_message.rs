use mp_felt::Felt252Wrapper;
use mp_transactions::HandleL1MessageTransaction;
use parity_scale_codec::Encode;
use sp_runtime::traits::ValidateUnsigned;
use sp_runtime::transaction_validity::{TransactionSource, TransactionValidityError};
use starknet_api::api_core::Nonce;
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::Fee;

use super::mock::default_mock::*;
use super::mock::*;
use crate::{Call, InvalidTransaction, L1Messages};

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

        L1Messages::<MockRuntime>::insert(Nonce(StarkFelt::from(nonce)), ());

        assert_eq!(
            Starknet::validate_unsigned(tx_source, &call),
            Err(TransactionValidityError::Invalid(InvalidTransaction::Stale))
        );
    });
}
