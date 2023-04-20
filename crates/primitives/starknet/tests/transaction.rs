use frame_support::bounded_vec;
use mp_starknet::transaction::types::{EventError, EventWrapper, Transaction, TransactionReceiptWrapper, TxType};
use sp_core::{H256, U256};
use starknet_api::api_core::{ContractAddress, PatriciaKey};
use starknet_api::block::{BlockHash, BlockNumber};
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::patricia_key;
use starknet_api::transaction::{
    Event, EventContent, EventData, EventKey, Fee, InvokeTransactionOutput, TransactionHash, TransactionOutput,
    TransactionReceipt,
};

#[test]
fn verify_tx_version_passes_for_valid_version() {
    let tx = Transaction {
        version: 1_u8,
        hash: H256::from_low_u64_be(6),
        signature: bounded_vec![H256::from_low_u64_be(10), H256::from_low_u64_be(20), H256::from_low_u64_be(30)],
        sender_address: [0; 32],
        nonce: U256::zero(),
        ..Transaction::default()
    };

    assert!(tx.verify_tx_version(&TxType::InvokeTx).is_ok())
}

#[test]
fn verify_tx_version_fails_for_invalid_version() {
    let tx = Transaction {
        version: 0_u8,
        hash: H256::from_low_u64_be(6),
        signature: bounded_vec![H256::from_low_u64_be(10), H256::from_low_u64_be(20), H256::from_low_u64_be(30)],
        sender_address: [0; 32],
        nonce: U256::zero(),
        ..Transaction::default()
    };

    assert!(tx.verify_tx_version(&TxType::InvokeTx).is_err())
}

#[test]
fn test_try_into_transaction_receipt_wrapper() {
    let event1 = Event {
        from_address: ContractAddress(patricia_key!("0x0")),
        content: EventContent {
            keys: vec![EventKey(StarkFelt::new([0; 32]).unwrap())],
            data: EventData(vec![StarkFelt::new([1; 32]).unwrap(), StarkFelt::new([2; 32]).unwrap()]),
        },
    };

    let event2 = Event {
        from_address: ContractAddress(patricia_key!("0x1")),
        content: EventContent {
            keys: vec![EventKey(StarkFelt::new([1; 32]).unwrap())],
            data: EventData(vec![StarkFelt::new([3; 32]).unwrap(), StarkFelt::new([4; 32]).unwrap()]),
        },
    };

    // Create a sample TransactionReceipt
    let transaction_receipt = &TransactionReceipt {
        transaction_hash: TransactionHash(StarkFelt::new([1; 32]).unwrap()),
        output: TransactionOutput::Invoke(InvokeTransactionOutput {
            actual_fee: Fee(0),
            messages_sent: vec![],
            events: vec![event1.clone(), event2.clone()],
        }),
        block_hash: BlockHash(StarkFelt::new([0; 32]).unwrap()),
        block_number: BlockNumber(0),
    };

    // Convert TransactionReceipt to TransactionReceiptWrapper
    let result: Result<TransactionReceiptWrapper, EventError> = transaction_receipt.try_into();

    // Check if the conversion is successful
    assert!(result.is_ok());

    let transaction_receipt_wrapper = result.unwrap();
    let events = transaction_receipt_wrapper.events;

    // Check if the transaction hash, actual fee, and tx type are correctly converted
    assert_eq!(transaction_receipt_wrapper.transaction_hash, H256::from_slice(&[1; 32]));
    assert_eq!(transaction_receipt_wrapper.actual_fee, U256::from(0));
    assert_eq!(transaction_receipt_wrapper.tx_type, TxType::InvokeTx);

    // Check if the events are correctly converted
    let event_wrapper1 = EventWrapper::builder()
        .with_event_content(event1.content)
        .with_from_address(ContractAddress(patricia_key!("0x0")))
        .build()
        .unwrap();
    let event_wrapper2 = EventWrapper::builder()
        .with_event_content(event2.content)
        .with_from_address(ContractAddress(patricia_key!("0x1")))
        .build()
        .unwrap();

    assert_eq!(events.len(), 2);

    assert_eq!(events.get(0).unwrap().data, event_wrapper1.data);
    assert_eq!(events.get(0).unwrap().from_address, event_wrapper1.from_address);

    assert_eq!(events.get(1).unwrap().data, event_wrapper2.data);
    assert_eq!(events.get(1).unwrap().from_address, event_wrapper2.from_address);
}

#[test]
fn test_try_into_transaction_receipt_wrapper_with_too_many_events() {
    let events: Vec<Event> = (0..=10001)
        .map(|_| Event {
            from_address: ContractAddress(patricia_key!("0x0")),
            content: EventContent {
                keys: vec![EventKey(StarkFelt::new([0; 32]).unwrap())],
                data: EventData(vec![StarkFelt::new([1; 32]).unwrap()]),
            },
        })
        .collect();

    // Create a sample TransactionReceipt with too many events
    let transaction_receipt = &TransactionReceipt {
        transaction_hash: TransactionHash(StarkFelt::new([1; 32]).unwrap()),
        output: TransactionOutput::Invoke(InvokeTransactionOutput {
            actual_fee: Fee(0),
            messages_sent: vec![],
            events,
        }),
        block_hash: BlockHash(StarkFelt::new([0; 32]).unwrap()),
        block_number: BlockNumber(0),
    };

    // Convert TransactionReceipt to TransactionReceiptWrapper
    let result: Result<TransactionReceiptWrapper, EventError> = transaction_receipt.try_into();

    // Check if the conversion fails with the expected error
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), EventError::TooManyEvents);
}
