use crate::starknet_serde::{transaction_from_json, DeserializeCallEntrypointError, DeserializeTransactionError};

#[test]
fn test_missing_not_optional_field() {
    let json_content: &str = r#"{
      "hash": "0x0000000000000000000000000000000000000000000000000000000000000000",
      "signature": [],
      "events": [],
      "sender_address": "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77",
      "nonce": 0,
      "call_entrypoint": {
        "class_hash": "025ec026985a3bf8a0cc1fe17326b245dfdc3ff89b8fde106542a3ea56c5a918",
        "entrypoint_type": "External",
        "calldata": [],
        "storage_address": "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77",
        "caller_address": "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77"
      }
    }"#;

    let transaction = transaction_from_json(json_content, &[]);
    assert!(matches!(transaction, Err(DeserializeTransactionError::FailedToParse(_))));
}

#[test]
fn test_invalid_number_format() {
    let json_content: &str = r#"{
      "version": "invalid",
      "hash": "0x0000000000000000000000000000000000000000000000000000000000000000",
      "signature": [],
      "events": [],
      "sender_address": "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77",
      "nonce": 0,
      "call_entrypoint": {
        "class_hash": "025ec026985a3bf8a0cc1fe17326b245dfdc3ff89b8fde106542a3ea56c5a918",
        "entrypoint_type": "External",
        "calldata": [],
        "storage_address": "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77",
        "caller_address": "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77"
      }
    }"#;

    let transaction = transaction_from_json(json_content, &[]);
    assert!(matches!(transaction, Err(DeserializeTransactionError::FailedToParse(_))));
}

#[test]
fn test_invalid_format_for_h256() {
    // Hash not 32 bytes length
    let json_content: &str = r#"{
      "version": 1,
      "hash": "0x0000000000000000000000000000000000000000000000000000000000000000aa",
      "signature": [],
      "events": [],
      "sender_address": "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77",
      "nonce": 0,
      "call_entrypoint": {
        "class_hash": "025ec026985a3bf8a0cc1fe17326b245dfdc3ff89b8fde106542a3ea56c5a918",
        "entrypoint_type": "External",
        "calldata": [],
        "storage_address": "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77",
        "caller_address": "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77",
        "initial_gas": "0123"
      }
    }"#;
    let transaction = transaction_from_json(json_content, &[]);
    assert!(matches!(transaction, Err(DeserializeTransactionError::InvalidHash(_))));

    // Hash invalid hexa
    let json_content: &str = r#"{
      "version": 1,
      "hash": "Invalid",
      "signature": [],
      "events": [],
      "sender_address": "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77",
      "nonce": 0,
      "call_entrypoint": {
        "class_hash": "025ec026985a3bf8a0cc1fe17326b245dfdc3ff89b8fde106542a3ea56c5a918",
        "entrypoint_type": "External",
        "calldata": [],
        "storage_address": "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77",
        "caller_address": "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77",
        "initial_gas": "0123"
      }
    }"#;
    let transaction = transaction_from_json(json_content, &[]);
    assert!(matches!(transaction, Err(DeserializeTransactionError::InvalidHash(_))));
}

#[test]
fn test_invalid_format_for_address() {
    // Not 32 bytes length, will still work because it's a valid hexa
    let json_content: &str = r#"{
      "version": 1,
      "hash": "0x0000000000000000000000000000000000000000000000000000000000000000",
      "signature": [],
      "events": [],
      "sender_address": "02356b628D108863BAf8644c945d97bAD70190",
      "nonce": 0,
      "call_entrypoint": {
        "class_hash": "025ec026985a3bf8a0cc1fe17326b245dfdc3ff89b8fde106542a3ea56c5a918",
        "entrypoint_type": "External",
        "calldata": [],
        "storage_address": "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77",
        "caller_address": "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77",
        "initial_gas": "0123"
      }
    }"#;
    let transaction = transaction_from_json(json_content, &[]);
    assert!(transaction.is_ok(), "Expected no error because sender_address is a valid hex value.");

    // No valid hexa
    let json_content: &str = r#"{
      "version": 1,
      "hash": "0x0000000000000000000000000000000000000000000000000000000000000000",
      "signature": [],
      "events": [],
      "sender_address": "Invalid",
      "nonce": 0,
      "call_entrypoint": {
        "class_hash": "025ec026985a3bf8a0cc1fe17326b245dfdc3ff89b8fde106542a3ea56c5a918",
        "entrypoint_type": "External",
        "calldata": [],
        "storage_address": "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77",
        "caller_address": "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77",
        "initial_gas": "0123"
      }
    }"#;
    let transaction = transaction_from_json(json_content, &[]);
    assert!(matches!(transaction, Err(DeserializeTransactionError::InvalidSenderAddress(_))));
}

#[test]
fn test_missing_optional_field_no_error() {
    // class_hash in call_entrypoint is optional
    let json_content: &str = r#"{
      "version": 1,
      "hash": "0x0000000000000000000000000000000000000000000000000000000000000000",
      "signature": [],
      "events": [],
      "sender_address": "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77",
      "nonce": 0,
      "call_entrypoint": {
        "entrypoint_type": "External",
        "calldata": [],
        "storage_address": "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77",
        "caller_address": "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77",
        "initial_gas": "0123"
      }
    }"#;

    let transaction = transaction_from_json(json_content, &[]);
    assert!(transaction.is_ok(), "Expected no error because class_hash in call_entrypoint is optional");
}

#[test]
fn test_wrong_entrypoint_type() {
    // class_hash in call_entrypoint is optional
    let json_content: &str = r#"{
      "version": 1,
      "hash": "0x0000000000000000000000000000000000000000000000000000000000000000",
      "signature": [],
      "events": [],
      "sender_address": "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77",
      "nonce": 0,
      "call_entrypoint": {
        "entrypoint_type": "wrong type",
        "calldata": [],
        "storage_address": "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77",
        "caller_address": "02356b628D108863BAf8644c945d97bAD70190AF5957031f4852d00D0F690a77",
        "initial_gas": "0123"
      }
    }"#;

    let transaction = transaction_from_json(json_content, &[]);
    assert!(matches!(
        transaction,
        Err(DeserializeTransactionError::InvalidCallEntryPoint(DeserializeCallEntrypointError::InvalidEntryPointType))
    ));
}
