use frame_support::bounded_vec;
use mp_starknet::execution::types::{
    CallEntryPointWrapper, ContractAddressWrapper, EntryPointTypeWrapper, Felt252Wrapper,
};
use mp_starknet::transaction::types::Transaction;

use crate::message::Message;
use crate::offchain_worker::OffchainWorkerError;

#[test]
fn test_try_into_transaction_correct_message_should_work() {
    let felt_one = Felt252Wrapper::ONE;
    let sender_address = felt_one;
    let hex = "0x0000000000000000000000000000000000000000000000000000000000000001".to_owned();
    let test_message: Message = Message { topics: vec![hex.clone(), hex.clone(), hex.clone(), hex.clone()], data: hex };
    let expected_tx = Transaction {
        sender_address,
        nonce: Felt252Wrapper::ONE,
        call_entrypoint: CallEntryPointWrapper {
            class_hash: None,
            entrypoint_type: EntryPointTypeWrapper::L1Handler,
            entrypoint_selector: Some(felt_one),
            calldata: bounded_vec![Felt252Wrapper::ONE, Felt252Wrapper::ONE],
            storage_address: felt_one,
            caller_address: ContractAddressWrapper::default(),
            initial_gas: Felt252Wrapper::default(),
        },
        ..Transaction::default()
    };
    pretty_assertions::assert_eq!(test_message.try_into_transaction().unwrap(), expected_tx);
}

#[test]
fn test_try_into_transaction_incorrect_topic_should_fail() {
    let hex = "0x1".to_owned();
    let test_message: Message =
        Message { topics: vec![hex.clone(), hex.clone(), "foo".to_owned(), hex.clone()], data: hex };
    assert_eq!(test_message.try_into_transaction().unwrap_err(), OffchainWorkerError::ToTransactionError);
}

#[test]
fn test_try_into_transaction_incorrect_selector_in_topic_should_fail() {
    let hex = "0x1".to_owned();
    let test_message: Message =
        Message { topics: vec![hex.clone(), hex.clone(), hex.clone(), "foo".to_owned()], data: hex };
    assert_eq!(test_message.try_into_transaction().unwrap_err(), OffchainWorkerError::ToTransactionError);
}

#[test]
fn test_try_into_transaction_empty_data_should_fail() {
    let hex = "0x1".to_owned();
    let test_message: Message =
        Message { topics: vec![hex.clone(), hex.clone(), hex.clone(), hex], data: "".to_owned() };
    assert_eq!(test_message.try_into_transaction().unwrap_err(), OffchainWorkerError::EmptyData);
}
