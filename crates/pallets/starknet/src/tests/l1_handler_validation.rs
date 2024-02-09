use assert_matches::assert_matches;
use mp_felt::Felt252Wrapper;
use mp_transactions::{HandleL1MessageTransaction, UserOrL1HandlerTransaction};
use sp_runtime::transaction_validity::InvalidTransaction;
use starknet_api::api_core::Nonce;
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::Fee;

use super::mock::default_mock::*;
use super::mock::*;
use crate::transaction_validation::TxPriorityInfo;
use crate::L1Messages;

#[test]
fn should_ensure_l1_message_not_executed_work_properly() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let nonce = Nonce(StarkFelt::from(1u64));

        assert!(Starknet::ensure_l1_message_not_executed(&nonce).is_ok());

        L1Messages::<MockRuntime>::mutate(|nonces| nonces.insert(nonce));

        assert_eq!(Starknet::ensure_l1_message_not_executed(&nonce), Err(InvalidTransaction::Stale));
    });
}

#[test]
fn should_accept_unused_nonce() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let nonce: u64 = 1;
        let transaction = HandleL1MessageTransaction {
            nonce,
            contract_address: Default::default(),
            entry_point_selector: Default::default(),
            calldata: Default::default(),
        };

        let tx = UserOrL1HandlerTransaction::L1Handler(transaction, Fee(100));

        assert_eq!(
            Starknet::validate_unsigned_tx_nonce(&tx),
            Ok(TxPriorityInfo::L1Handler { nonce: Felt252Wrapper::ONE })
        );
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

        let tx = UserOrL1HandlerTransaction::L1Handler(transaction, Fee(100));

        L1Messages::<MockRuntime>::mutate(|nonces| nonces.insert(Nonce(nonce.into())));

        assert_matches!(Starknet::validate_unsigned_tx_nonce(&tx), Err(InvalidTransaction::Stale));
    });
}

#[test]
fn should_accept_valid_unsigned_l1_message_tx() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let nonce: u64 = 1;
        let transaction = HandleL1MessageTransaction {
            nonce,
            contract_address: Default::default(),
            entry_point_selector: Default::default(),
            calldata: Default::default(),
        };

        let tx = UserOrL1HandlerTransaction::L1Handler(transaction, Fee(100));

        assert!(Starknet::validate_unsigned_tx(&tx).is_ok());
    });
}
