use assert_matches::assert_matches;
use blockifier::transaction::transaction_execution::Transaction;
use blockifier::transaction::transactions::L1HandlerTransaction as BlockifierL1HandlerTransaction;
use mp_felt::Felt252Wrapper;
use mp_transactions::compute_hash::ComputeTransactionHash;
use sp_runtime::transaction_validity::InvalidTransaction;
use starknet_api::core::{ContractAddress, Nonce, PatriciaKey};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::{Fee, L1HandlerTransaction as StarknetApiL1HandlerTransaction, TransactionVersion};

use super::mock::default_mock::*;
use super::mock::*;
use crate::transaction_validation::TxPriorityInfo;
use crate::L1Messages;

fn create_l1_handler_transaction(chain_id: Felt252Wrapper, nonce: Nonce) -> Transaction {
    let tx = StarknetApiL1HandlerTransaction {
        nonce: Nonce(StarkFelt::ONE),
        contract_address: ContractAddress(PatriciaKey(Default::default())),
        entry_point_selector: Default::default(),
        calldata: Default::default(),
        version: TransactionVersion(StarkFelt::ZERO),
    };

    let tx_hash = tx.compute_hash(chain_id, false);

    Transaction::L1HandlerTransaction(BlockifierL1HandlerTransaction { tx, tx_hash, paid_fee_on_l1: Fee(100) })
}

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
        let tx = create_l1_handler_transaction(Starknet::chain_id(), nonce);
        assert_eq!(Starknet::validate_unsigned_tx_nonce(&tx), Ok(TxPriorityInfo::L1Handler { nonce }));
    });
}

#[test]
fn should_reject_used_nonce() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let nonce = Nonce(StarkFelt::ONE);
        let tx = create_l1_handler_transaction(Starknet::chain_id(), nonce);

        L1Messages::<MockRuntime>::mutate(|nonces| nonces.insert(nonce.into()));

        assert_matches!(Starknet::validate_unsigned_tx_nonce(&tx), Err(InvalidTransaction::Stale));
    });
}
