use assert_matches::assert_matches;
use sp_runtime::transaction_validity::InvalidTransaction;
use starknet_api::core::Nonce;
use starknet_api::hash::StarkFelt;

use super::mock::default_mock::*;
use super::mock::*;
use crate::tests::create_l1_handler_transaction;
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

        let nonce = Nonce(StarkFelt::ONE);
        let tx = create_l1_handler_transaction(Starknet::chain_id(), nonce, None, None, None);
        assert_eq!(
            Starknet::pre_validate_unsigned_tx(
                &blockifier::transaction::transaction_execution::Transaction::L1HandlerTransaction(tx)
            ),
            Ok(())
        );
    });
}

#[test]
fn should_reject_used_nonce() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let nonce = Nonce(StarkFelt::ONE);
        let tx = create_l1_handler_transaction(Starknet::chain_id(), nonce, None, None, None);

        L1Messages::<MockRuntime>::mutate(|nonces| nonces.insert(nonce));

        assert_matches!(
            Starknet::pre_validate_unsigned_tx(
                &blockifier::transaction::transaction_execution::Transaction::L1HandlerTransaction(tx)
            ),
            Err(InvalidTransaction::Stale)
        );
    });
}
