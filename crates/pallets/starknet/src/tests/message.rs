use crate::message::Message;
use crate::offchain_worker::OffchainWorkerError;

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
