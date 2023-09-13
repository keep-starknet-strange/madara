use assert::assert_err;
use serde_json::{Map, Value as JsonValue};

use super::{KnownStarknetErrorCode, StarknetError, StarknetErrorCode};

fn deserialize_starknet_error(code: &str, message: &str) -> StarknetError {
    serde_json::from_value::<StarknetError>(JsonValue::Object(Map::from_iter([
        ("code".to_string(), JsonValue::String(code.to_string())),
        ("message".to_string(), JsonValue::String(message.to_string())),
    ])))
    .unwrap()
}

#[test]
fn known_error_code_deserialization() {
    const MESSAGE: &str = "message";
    for (code_str, known_code) in [
        ("StarknetErrorCode.UNDECLARED_CLASS", KnownStarknetErrorCode::UndeclaredClass),
        ("StarknetErrorCode.BLOCK_NOT_FOUND", KnownStarknetErrorCode::BlockNotFound),
        ("StarkErrorCode.MALFORMED_REQUEST", KnownStarknetErrorCode::MalformedRequest),
        ("StarknetErrorCode.OUT_OF_RANGE_CLASS_HASH", KnownStarknetErrorCode::OutOfRangeClassHash),
        ("StarknetErrorCode.CLASS_ALREADY_DECLARED", KnownStarknetErrorCode::ClassAlreadyDeclared),
        ("StarknetErrorCode.COMPILATION_FAILED", KnownStarknetErrorCode::CompilationFailed),
        (
            "StarknetErrorCode.CONTRACT_BYTECODE_SIZE_TOO_LARGE",
            KnownStarknetErrorCode::ContractBytecodeSizeTooLarge,
        ),
        (
            "StarknetErrorCode.CONTRACT_CLASS_OBJECT_SIZE_TOO_LARGE",
            KnownStarknetErrorCode::ContractClassObjectSizeTooLarge,
        ),
        ("StarknetErrorCode.DUPLICATED_TRANSACTION", KnownStarknetErrorCode::DuplicatedTransaction),
        (
            "StarknetErrorCode.ENTRY_POINT_NOT_FOUND_IN_CONTRACT",
            KnownStarknetErrorCode::EntryPointNotFoundInContract,
        ),
        (
            "StarknetErrorCode.INSUFFICIENT_ACCOUNT_BALANCE",
            KnownStarknetErrorCode::InsufficientAccountBalance,
        ),
        ("StarknetErrorCode.INSUFFICIENT_MAX_FEE", KnownStarknetErrorCode::InsufficientMaxFee),
        (
            "StarknetErrorCode.INVALID_COMPILED_CLASS_HASH",
            KnownStarknetErrorCode::InvalidCompiledClassHash,
        ),
        (
            "StarknetErrorCode.INVALID_CONTRACT_CLASS_VERSION",
            KnownStarknetErrorCode::InvalidContractClassVersion,
        ),
        (
            "StarknetErrorCode.INVALID_TRANSACTION_NONCE",
            KnownStarknetErrorCode::InvalidTransactionNonce,
        ),
        (
            "StarknetErrorCode.INVALID_TRANSACTION_VERSION",
            KnownStarknetErrorCode::InvalidTransactionVersion,
        ),
        ("StarknetErrorCode.VALIDATE_FAILURE", KnownStarknetErrorCode::ValidateFailure),
        (
            "StarknetErrorCode.TRANSACTION_LIMIT_EXCEEDED",
            KnownStarknetErrorCode::TransactionLimitExceeded,
        ),
    ] {
        let starknet_error = deserialize_starknet_error(code_str, MESSAGE);
        let expected_starknet_error = StarknetError {
            code: StarknetErrorCode::KnownErrorCode(known_code),
            message: MESSAGE.to_string(),
        };
        assert_eq!(expected_starknet_error, starknet_error);
    }
}

#[test]
fn unknown_error_code_deserialization() {
    const MESSAGE: &str = "message";
    const CODE_STR: &str = "StarknetErrorCode.MADE_UP_CODE_FOR_TEST";
    let starknet_error = deserialize_starknet_error(CODE_STR, MESSAGE);
    let expected_starknet_error = StarknetError {
        code: StarknetErrorCode::UnknownErrorCode(CODE_STR.to_string()),
        message: MESSAGE.to_string(),
    };
    assert_eq!(expected_starknet_error, starknet_error);
}

// This test is needed because bugs can happen in the custom deserialization of UnknownErrorCode
#[test]
fn starknet_error_code_invalid_json_format_fails() {
    assert_err!(serde_json::from_str::<StarknetErrorCode>("A string not surrounded with quotes"));
}
