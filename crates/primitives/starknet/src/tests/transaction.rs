use core::str::FromStr;
use std::sync::Arc;

use blockifier::abi::abi_utils::selector_from_name;
use frame_support::{bounded_vec, BoundedVec};
use sp_core::{TypedGet, U256};
use starknet_api::api_core::{ContractAddress, PatriciaKey};
use starknet_api::block::{BlockHash, BlockNumber};
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::patricia_key;
use starknet_api::transaction::{
    Event, EventContent, EventData, EventKey, Fee, InvokeTransactionOutput, TransactionHash, TransactionOutput,
    TransactionReceipt,
};
use starknet_core::types::contract::legacy::LegacyContractClass;
use starknet_core::types::contract::SierraClass;
use starknet_core::types::{
    BroadcastedDeclareTransaction, BroadcastedDeclareTransactionV1, BroadcastedDeclareTransactionV2,
    BroadcastedDeployAccountTransaction, BroadcastedInvokeTransaction, BroadcastedInvokeTransactionV0,
    BroadcastedInvokeTransactionV1, CompressedLegacyContractClass, FlattenedSierraClass, StarknetError,
};
use starknet_ff::FieldElement;

use crate::execution::call_entrypoint_wrapper::{CallEntryPointWrapper, MaxCalldataSize};
use crate::execution::types::{ContractAddressWrapper, Felt252Wrapper};
use crate::transaction::constants;
use crate::transaction::types::{
    BroadcastedTransactionConversionErrorWrapper, DeclareTransaction, DeployAccountTransaction, EventError,
    EventWrapper, InvokeTransaction, MaxArraySize, Transaction, TransactionReceiptWrapper, TxType,
};

#[test]
fn test_validate_entry_point_selector_is_declare() {
    // Given
    let tx = Transaction::default();

    // When
    let actual_entrypoint = tx.validate_entry_point_selector(&TxType::Declare).unwrap();

    // Then
    let expected_entrypoint = selector_from_name(constants::VALIDATE_DECLARE_ENTRY_POINT_NAME);
    assert_eq!(expected_entrypoint, actual_entrypoint);
}

#[test]
fn test_validate_entry_point_selector_is_deploy_account() {
    // Given
    let tx = Transaction::default();

    // When
    let actual_entrypoint = tx.validate_entry_point_selector(&TxType::DeployAccount).unwrap();

    // Then
    let expected_entrypoint = selector_from_name(constants::VALIDATE_DEPLOY_ENTRY_POINT_NAME);
    assert_eq!(expected_entrypoint, actual_entrypoint);
}

#[test]
fn test_validate_entry_point_selector_is_invoke() {
    // Given
    let tx = Transaction::default();

    // When
    let actual_entrypoint = tx.validate_entry_point_selector(&TxType::Invoke).unwrap();

    // Then
    let expected_entrypoint = selector_from_name(constants::VALIDATE_ENTRY_POINT_NAME);
    assert_eq!(expected_entrypoint, actual_entrypoint);
}

#[test]
fn test_validate_entry_point_selector_fails_for_l1_handler() {
    // Given
    let tx = Transaction::default();

    // When
    let actual_entrypoint = tx.validate_entry_point_selector(&TxType::L1Handler);

    // Then
    assert!(actual_entrypoint.is_err());
}

fn get_test_class_hash() -> Felt252Wrapper {
    Felt252Wrapper::try_from(&[2; 32]).unwrap()
}

fn get_test_calldata() -> BoundedVec<Felt252Wrapper, MaxCalldataSize> {
    bounded_vec![Felt252Wrapper::from_hex_be("0x1").unwrap(), Felt252Wrapper::from_hex_be("0x2").unwrap()]
}

fn get_test_contract_address_salt() -> U256 {
    U256::from_str("0x000000000000000000000000000000000000000000000000000000000000dead").unwrap()
}

#[test]
fn test_validate_entrypoint_calldata_declare() {
    // Given
    let tx = Transaction {
        call_entrypoint: CallEntryPointWrapper {
            class_hash: Some(get_test_class_hash()),
            ..CallEntryPointWrapper::default()
        },
        ..Transaction::default()
    };

    // When
    let actual_calldata = (*tx.validate_entrypoint_calldata(&TxType::Declare).unwrap().0)
        .iter()
        .map(|x| Felt252Wrapper::from(*x))
        .collect::<Vec<_>>();

    // Then
    let expected_calldata = vec![get_test_class_hash()];
    assert_eq!(expected_calldata, actual_calldata);
}

#[test]
fn test_validate_entrypoint_calldata_deploy_account() {
    // Given
    let tx = Transaction {
        contract_address_salt: Some(get_test_contract_address_salt()),
        call_entrypoint: CallEntryPointWrapper {
            class_hash: Some(get_test_class_hash()),
            calldata: get_test_calldata(),
            ..CallEntryPointWrapper::default()
        },
        ..Transaction::default()
    };

    // When
    let actual_calldata = (*tx.validate_entrypoint_calldata(&TxType::DeployAccount).unwrap().0)
        .iter()
        .map(|x| Felt252Wrapper::from(*x))
        .collect::<Vec<_>>();

    // Then
    let mut salt_bytes = [0; 32];
    get_test_contract_address_salt().to_big_endian(&mut salt_bytes);
    let mut expected_calldata = vec![get_test_class_hash(), Felt252Wrapper::try_from(&salt_bytes).unwrap()];
    expected_calldata.extend(get_test_calldata().to_vec());

    assert_eq!(expected_calldata, actual_calldata);
}

#[test]
fn test_validate_entrypoint_calldata_invoke() {
    // Given
    let tx = Transaction {
        call_entrypoint: CallEntryPointWrapper { calldata: get_test_calldata(), ..CallEntryPointWrapper::default() },
        ..Transaction::default()
    };

    // When
    let actual_calldata = (*tx.validate_entrypoint_calldata(&TxType::Invoke).unwrap().0)
        .iter()
        .map(|x| Felt252Wrapper::from(*x))
        .collect::<Vec<_>>();

    // Then
    let expected_calldata = get_test_calldata().to_vec();

    assert_eq!(expected_calldata, actual_calldata);
}

#[test]
fn test_validate_entrypoint_calldata_fails_for_l1_handler() {
    // Given
    let tx = Transaction::default();

    // When
    let actual_calldata = tx.validate_entrypoint_calldata(&TxType::L1Handler);

    // Then
    assert!(actual_calldata.is_err());
}

#[test]
fn verify_tx_version_passes_for_valid_version() {
    let tx = Transaction {
        version: 1_u8,
        hash: Felt252Wrapper::from(6_u128),
        signature: bounded_vec![
            Felt252Wrapper::from(10_u128),
            Felt252Wrapper::from(20_u128),
            Felt252Wrapper::from(30_u128)
        ],
        sender_address: Felt252Wrapper::ZERO,
        nonce: Felt252Wrapper::ZERO,
        ..Transaction::default()
    };

    assert!(tx.verify_tx_version(&TxType::Invoke).is_ok())
}

#[test]
fn verify_tx_version_fails_for_invalid_version() {
    let tx = Transaction {
        version: 0_u8,
        hash: Felt252Wrapper::from(6_u128),
        signature: bounded_vec![
            Felt252Wrapper::from(10_u128),
            Felt252Wrapper::from(20_u128),
            Felt252Wrapper::from(30_u128)
        ],
        sender_address: Felt252Wrapper::ZERO,
        nonce: Felt252Wrapper::ZERO,
        ..Transaction::default()
    };

    assert!(tx.verify_tx_version(&TxType::Invoke).is_err())
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
    assert_eq!(transaction_receipt_wrapper.transaction_hash, Felt252Wrapper::try_from(&[1; 32]).unwrap());
    assert_eq!(transaction_receipt_wrapper.actual_fee, Felt252Wrapper::ZERO);
    assert_eq!(transaction_receipt_wrapper.tx_type, TxType::Invoke);

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

#[test]
fn test_event_wrapper_new() {
    let keys: BoundedVec<Felt252Wrapper, MaxArraySize> =
        bounded_vec![Felt252Wrapper::ZERO, Felt252Wrapper::try_from(&[1; 32]).unwrap()];
    let data: BoundedVec<Felt252Wrapper, MaxArraySize> =
        bounded_vec![Felt252Wrapper::try_from(&[1; 32]).unwrap(), Felt252Wrapper::try_from(&[2; 32]).unwrap()];
    let from_address = Felt252Wrapper::try_from(&[3; 32]).unwrap();
    let transaction_hash = Felt252Wrapper::try_from(&[4; 32]).unwrap();

    let event_wrapper = EventWrapper::new(keys.clone(), data.clone(), from_address, transaction_hash);
    let expected_event = EventWrapper { keys, data, from_address, transaction_hash };

    pretty_assertions::assert_eq!(event_wrapper, expected_event);
}

#[test]
fn test_event_wrapper_empty() {
    let event_wrapper = EventWrapper::empty();

    let expected_event = EventWrapper {
        keys: bounded_vec![],
        data: bounded_vec![],
        from_address: ContractAddressWrapper::default(),
        transaction_hash: Felt252Wrapper::default(),
    };

    pretty_assertions::assert_eq!(event_wrapper, expected_event);
}

#[test]
fn test_event_wrapper_builder() {
    let keys = vec![Felt252Wrapper::ZERO, Felt252Wrapper::try_from(&[1; 32]).unwrap()];
    let data = vec![Felt252Wrapper::try_from(&[1; 32]).unwrap(), Felt252Wrapper::try_from(&[2; 32]).unwrap()];
    let from_address = Felt252Wrapper::try_from(&[3; 32]).unwrap();
    let transaction_hash = Felt252Wrapper::try_from(&[4; 32]).unwrap();

    let event_wrapper = EventWrapper::builder()
        .with_keys(keys.clone())
        .with_data(data.clone())
        .with_from_address(ContractAddress::try_from(StarkFelt::new(from_address.into()).unwrap()).unwrap())
        .with_transaction_hash(TransactionHash(StarkFelt::new(transaction_hash.into()).unwrap()))
        .build()
        .unwrap();

    let expected_event = EventWrapper {
        keys: BoundedVec::<Felt252Wrapper, MaxArraySize>::try_from(keys).unwrap(),
        data: BoundedVec::<Felt252Wrapper, MaxArraySize>::try_from(data).unwrap(),
        from_address,
        transaction_hash,
    };

    pretty_assertions::assert_eq!(event_wrapper, expected_event);
}

#[test]
fn test_event_wrapper_builder_with_event_content() {
    let event_content = EventContent {
        keys: vec![EventKey(StarkFelt::new([0; 32]).unwrap())],
        data: EventData(vec![StarkFelt::new([1; 32]).unwrap(), StarkFelt::new([2; 32]).unwrap()]),
    };

    let event_wrapper = EventWrapper::builder().with_event_content(event_content).build().unwrap();

    let bounded_keys: BoundedVec<Felt252Wrapper, MaxArraySize> = bounded_vec!(Felt252Wrapper::ZERO);
    let bounded_data: BoundedVec<Felt252Wrapper, MaxArraySize> =
        bounded_vec![Felt252Wrapper::try_from(&[1; 32]).unwrap(), Felt252Wrapper::try_from(&[2; 32]).unwrap()];

    let expected_event = EventWrapper {
        keys: bounded_keys,
        data: bounded_data,
        from_address: ContractAddressWrapper::default(),
        transaction_hash: Felt252Wrapper::default(),
    };

    pretty_assertions::assert_eq!(event_wrapper, expected_event);
}

#[test]
fn test_try_into_deploy_account_transaction() {
    let zero_len = get_try_into_and_expected_value(0, 0).expect("failed to bound signature or calldata");
    pretty_assertions::assert_eq!(zero_len.0, zero_len.1);

    let one_len = get_try_into_and_expected_value(1, 1).expect("failed to bound signature or calldata");
    pretty_assertions::assert_eq!(one_len.0, one_len.1);

    let max_array_size: u32 = MaxArraySize::get();
    let max_array_size: usize = max_array_size.try_into().unwrap();

    let max_calldata_size: u32 = MaxCalldataSize::get();
    let max_calldata_size: usize = max_calldata_size.try_into().unwrap();

    let max_len = get_try_into_and_expected_value(max_array_size, max_calldata_size)
        .expect("Expected to work because its within bound limit");

    pretty_assertions::assert_eq!(max_len.0, max_len.1);

    let array_outbound = get_try_into_and_expected_value(max_array_size + 1, max_calldata_size);
    assert!(matches!(array_outbound.unwrap_err(), BroadcastedTransactionConversionErrorWrapper::SignatureBoundError));

    let calldata_outbound = get_try_into_and_expected_value(max_array_size, max_calldata_size + 1);
    assert!(matches!(calldata_outbound.unwrap_err(), BroadcastedTransactionConversionErrorWrapper::CalldataBoundError));
}

#[test]
fn test_try_invoke_txn_from_broadcasted_invoke_txn_v0() {
    let broadcasted_invoke_txn_v0 = BroadcastedInvokeTransactionV0 {
        max_fee: FieldElement::default(),
        signature: vec![FieldElement::default()],
        nonce: FieldElement::default(),
        contract_address: FieldElement::default(),
        entry_point_selector: FieldElement::default(),
        calldata: vec![FieldElement::default()],
    };

    let broadcasted_invoke_txn = BroadcastedInvokeTransaction::V0(broadcasted_invoke_txn_v0);
    let invoke_txn = InvokeTransaction::try_from(broadcasted_invoke_txn);

    assert!(invoke_txn.is_err());
    assert!(matches!(
        invoke_txn.unwrap_err(),
        BroadcastedTransactionConversionErrorWrapper::StarknetError(StarknetError::FailedToReceiveTransaction)
    ))
}

#[test]
fn test_try_invoke_txn_from_broadcasted_invoke_txn_v1() {
    let broadcasted_invoke_txn_v1 = BroadcastedInvokeTransactionV1 {
        max_fee: FieldElement::default(),
        nonce: FieldElement::default(),
        sender_address: FieldElement::default(),
        signature: vec![FieldElement::default()],
        calldata: vec![FieldElement::default()],
    };

    let broadcasted_invoke_txn = BroadcastedInvokeTransaction::V1(broadcasted_invoke_txn_v1);
    let invoke_txn = InvokeTransaction::try_from(broadcasted_invoke_txn).unwrap();

    let expected_sig: BoundedVec<Felt252Wrapper, MaxArraySize> =
        BoundedVec::try_from(vec![Felt252Wrapper::from(FieldElement::default())]).unwrap();
    let expected_calldata: BoundedVec<Felt252Wrapper, MaxArraySize> =
        BoundedVec::try_from(vec![Felt252Wrapper::from(FieldElement::default())]).unwrap();

    pretty_assertions::assert_eq!(invoke_txn.version, 1_u8);
    pretty_assertions::assert_eq!(invoke_txn.sender_address, Felt252Wrapper::from(FieldElement::default()));
    pretty_assertions::assert_eq!(invoke_txn.calldata, expected_calldata);
    pretty_assertions::assert_eq!(invoke_txn.nonce, Felt252Wrapper::from(FieldElement::default()));
    pretty_assertions::assert_eq!(invoke_txn.signature, expected_sig);
    pretty_assertions::assert_eq!(invoke_txn.max_fee, Felt252Wrapper::from(FieldElement::default()));
}

#[test]
fn test_try_invoke_txn_from_broadcasted_invoke_txn_v1_max_sig_size() {
    let signature_size_maxed = vec![FieldElement::default(); MaxArraySize::get() as usize + 1];

    let broadcasted_invoke_txn_v1 = BroadcastedInvokeTransactionV1 {
        max_fee: FieldElement::default(),
        nonce: FieldElement::default(),
        sender_address: FieldElement::default(),
        signature: signature_size_maxed,
        calldata: vec![FieldElement::default()],
    };

    let broadcasted_invoke_txn = BroadcastedInvokeTransaction::V1(broadcasted_invoke_txn_v1);
    let invoke_txn = InvokeTransaction::try_from(broadcasted_invoke_txn);

    assert!(invoke_txn.is_err());
    assert!(matches!(invoke_txn.unwrap_err(), BroadcastedTransactionConversionErrorWrapper::SignatureConversionError));
}

#[test]
fn test_try_invoke_txn_from_broadcasted_invoke_txn_v1_max_calldata_size() {
    let calldata_size_maxed = vec![FieldElement::default(); MaxCalldataSize::get() as usize + 1];

    let broadcasted_invoke_txn_v1 = BroadcastedInvokeTransactionV1 {
        max_fee: FieldElement::default(),
        nonce: FieldElement::default(),
        sender_address: FieldElement::default(),
        signature: vec![FieldElement::default()],
        calldata: calldata_size_maxed,
    };

    let broadcasted_invoke_txn = BroadcastedInvokeTransaction::V1(broadcasted_invoke_txn_v1);
    let invoke_txn = InvokeTransaction::try_from(broadcasted_invoke_txn);

    assert!(invoke_txn.is_err());
    assert!(matches!(invoke_txn.unwrap_err(), BroadcastedTransactionConversionErrorWrapper::CalldataConversionError));
}

#[test]
fn test_try_into_declare_transaction_v1_valid() {
    let compressed_contract_class = get_compressed_legacy_contract_class();

    let txn = BroadcastedDeclareTransactionV1 {
        max_fee: FieldElement::default(),
        signature: vec![FieldElement::default()],
        nonce: FieldElement::default(),
        contract_class: Arc::new(compressed_contract_class),
        sender_address: FieldElement::default(),
    };

    let input: BroadcastedDeclareTransaction = BroadcastedDeclareTransaction::V1(txn);
    let output_result: Result<DeclareTransaction, _> = input.try_into();
    assert!(output_result.is_ok());
}

#[test]
fn test_try_into_declare_transaction_v1_max_signature() {
    let compressed_contract_class = get_compressed_legacy_contract_class();

    let txn = BroadcastedDeclareTransactionV1 {
        max_fee: FieldElement::default(),
        signature: vec![FieldElement::default(); MaxArraySize::get() as usize + 1],
        nonce: FieldElement::default(),
        contract_class: Arc::new(compressed_contract_class),
        sender_address: FieldElement::default(),
    };

    let input: BroadcastedDeclareTransaction = BroadcastedDeclareTransaction::V1(txn);
    let output_result: Result<DeclareTransaction, _> = input.try_into();
    assert!(matches!(output_result.unwrap_err(), BroadcastedTransactionConversionErrorWrapper::SignatureBoundError));
}

#[test]
fn test_try_into_declare_transaction_v1_bad_gzip() {
    let mut compressed_contract_class = get_compressed_legacy_contract_class();

    // Manually change some bytes so its no longer a valid gzip
    if let Some(value) = compressed_contract_class.program.get_mut(0) {
        *value = 1;
    }
    if let Some(value) = compressed_contract_class.program.get_mut(1) {
        *value = 1;
    }

    let txn = BroadcastedDeclareTransactionV1 {
        max_fee: FieldElement::default(),
        signature: vec![FieldElement::default()],
        nonce: FieldElement::default(),
        contract_class: Arc::new(compressed_contract_class),
        sender_address: FieldElement::default(),
    };

    let input: BroadcastedDeclareTransaction = BroadcastedDeclareTransaction::V1(txn);
    let output_result: Result<DeclareTransaction, _> = input.try_into();
    assert!(matches!(
        output_result.unwrap_err(),
        BroadcastedTransactionConversionErrorWrapper::ContractClassProgramDecompressionError
    ));
}

#[test]
// TODO: this should be updated after support for v2 transaction is added
fn test_try_into_declare_transaction_v2() {
    let flattened_contract_class: FlattenedSierraClass = get_flattened_sierra_contract_class();

    let txn = BroadcastedDeclareTransactionV2 {
        max_fee: FieldElement::default(),
        signature: vec![FieldElement::default()],
        nonce: FieldElement::default(),
        contract_class: Arc::new(flattened_contract_class),
        sender_address: FieldElement::default(),
        compiled_class_hash: FieldElement::default(),
    };

    let input: BroadcastedDeclareTransaction = BroadcastedDeclareTransaction::V2(txn);
    let declare_txn = DeclareTransaction::try_from(input);

    assert!(matches!(
        declare_txn.unwrap_err(),
        BroadcastedTransactionConversionErrorWrapper::StarknetError(StarknetError::FailedToReceiveTransaction)
    ));
}

// This helper methods either returns result of `TryInto::try_into()` and expected result or the
// error in case `TryInto::try_into()` fails
fn get_try_into_and_expected_value(
    array_size: usize,
    calldata_size: usize,
) -> Result<(DeployAccountTransaction, DeployAccountTransaction), BroadcastedTransactionConversionErrorWrapper> {
    let signature: Vec<FieldElement> = vec![FieldElement::default(); array_size];
    let constructor_calldata: Vec<FieldElement> = vec![FieldElement::default(); calldata_size];

    let input = BroadcastedDeployAccountTransaction {
        signature,
        constructor_calldata,
        // `FieldElement` can be trivially converted to `Felt252Wrapper` so no need to test them
        max_fee: FieldElement::default(),
        nonce: FieldElement::default(),
        contract_address_salt: FieldElement::default(),
        class_hash: FieldElement::default(),
    };

    let output: DeployAccountTransaction = input.try_into()?;

    let expected_signature = bounded_vec![Felt252Wrapper::default(); array_size];
    let expected_calldata = bounded_vec![Felt252Wrapper::default(); calldata_size];

    let expected_output = DeployAccountTransaction {
        version: 1_u8,
        calldata: expected_calldata,
        signature: expected_signature,
        nonce: FieldElement::default().into(),
        salt: FieldElement::default().into(),
        account_class_hash: FieldElement::default().into(),
        max_fee: FieldElement::default().into(),
    };

    Ok((output, expected_output))
}

fn get_compressed_legacy_contract_class() -> CompressedLegacyContractClass {
    let contract_class_bytes = include_bytes!("../../../../../cairo-contracts/build/test.json");

    let contract_class: LegacyContractClass = serde_json::from_slice(contract_class_bytes).unwrap();
    let compressed_contract_class: CompressedLegacyContractClass = contract_class.compress().unwrap();

    compressed_contract_class
}

fn get_flattened_sierra_contract_class() -> FlattenedSierraClass {
    let contract_class_bytes = include_bytes!("../../../../../cairo-contracts/build/Example.sierra.json");

    let contract_class: SierraClass = serde_json::from_slice(contract_class_bytes).unwrap();
    let flattened_contract_class: FlattenedSierraClass = contract_class.flatten().unwrap();

    flattened_contract_class
}
