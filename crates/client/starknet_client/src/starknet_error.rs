#[cfg(test)]
#[path = "starknet_error_test.rs"]
mod starknet_error_test;

use std::fmt::{self, Display, Formatter};

#[cfg(any(feature = "testing", test))]
use enum_iterator::Sequence;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};

/// Error codes returned by the starknet gateway.
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(untagged)]
pub enum StarknetErrorCode {
    #[serde(deserialize_with = "deserialize_unknown_error_code")]
    UnknownErrorCode(String),
    KnownErrorCode(KnownStarknetErrorCode),
}

// This struct is needed because #[serde(other)] supports only unit variants and because
// #[serde(field_identifier)] doesn't work with serializable types.
// The issue requesting that #[serde(other)] will deserialize the variant with the unknown tag's
// content is: https://github.com/serde-rs/serde/issues/1701
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[cfg_attr(any(test, feature = "testing"), derive(Sequence))]
pub enum KnownStarknetErrorCode {
    #[serde(rename = "StarknetErrorCode.UNDECLARED_CLASS")]
    UndeclaredClass,
    #[serde(rename = "StarknetErrorCode.BLOCK_NOT_FOUND")]
    BlockNotFound,
    #[serde(rename = "StarkErrorCode.MALFORMED_REQUEST")]
    MalformedRequest,
    #[serde(rename = "StarknetErrorCode.OUT_OF_RANGE_CLASS_HASH")]
    OutOfRangeClassHash,
    #[serde(rename = "StarknetErrorCode.CLASS_ALREADY_DECLARED")]
    ClassAlreadyDeclared,
    #[serde(rename = "StarknetErrorCode.COMPILATION_FAILED")]
    CompilationFailed,
    #[serde(rename = "StarknetErrorCode.CONTRACT_BYTECODE_SIZE_TOO_LARGE")]
    ContractBytecodeSizeTooLarge,
    #[serde(rename = "StarknetErrorCode.CONTRACT_CLASS_OBJECT_SIZE_TOO_LARGE")]
    ContractClassObjectSizeTooLarge,
    #[serde(rename = "StarknetErrorCode.DUPLICATED_TRANSACTION")]
    DuplicatedTransaction,
    #[serde(rename = "StarknetErrorCode.ENTRY_POINT_NOT_FOUND_IN_CONTRACT")]
    EntryPointNotFoundInContract,
    #[serde(rename = "StarknetErrorCode.INSUFFICIENT_ACCOUNT_BALANCE")]
    InsufficientAccountBalance,
    #[serde(rename = "StarknetErrorCode.INSUFFICIENT_MAX_FEE")]
    InsufficientMaxFee,
    #[serde(rename = "StarknetErrorCode.INVALID_COMPILED_CLASS_HASH")]
    InvalidCompiledClassHash,
    #[serde(rename = "StarknetErrorCode.INVALID_CONTRACT_CLASS_VERSION")]
    InvalidContractClassVersion,
    #[serde(rename = "StarknetErrorCode.INVALID_TRANSACTION_NONCE")]
    InvalidTransactionNonce,
    #[serde(rename = "StarknetErrorCode.INVALID_TRANSACTION_VERSION")]
    InvalidTransactionVersion,
    #[serde(rename = "StarknetErrorCode.VALIDATE_FAILURE")]
    ValidateFailure,
    #[serde(rename = "StarknetErrorCode.TRANSACTION_LIMIT_EXCEEDED")]
    TransactionLimitExceeded,
}

/// A client error wrapping error codes returned by the starknet gateway.
#[derive(thiserror::Error, Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct StarknetError {
    pub code: StarknetErrorCode,
    pub message: String,
}

impl Display for StarknetError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

pub fn deserialize_unknown_error_code<'de, D>(de: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let string: String = Deserialize::deserialize(de)?;
    let string_as_json = format!("\"{string}\"");
    match serde_json::from_str::<KnownStarknetErrorCode>(&string_as_json) {
        Ok(_) => Err(D::Error::custom("Trying to serialize a known Starknet error code into UnknownErrorCode")),
        Err(json_err) => {
            if json_err.is_data() {
                return Ok(string);
            }
            Err(D::Error::custom(json_err))
        }
    }
}
