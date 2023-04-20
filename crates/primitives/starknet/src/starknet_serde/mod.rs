//! This module contains the serialization and deserialization functions for the StarkNet types.
use alloc::string::String;
use alloc::vec::Vec;
use alloc::{fmt, format};

use blockifier::test_utils::get_contract_class;
use frame_support::BoundedVec;
use hex::{FromHex, FromHexError};
use serde::{Deserialize, Serialize};
use sp_core::{H256, U256};

use crate::execution::{CallEntryPointWrapper, ContractAddressWrapper, ContractClassWrapper, EntryPointTypeWrapper};
use crate::transaction::types::{EventWrapper, MaxArraySize, Transaction};

/// Removes the "0x" prefix from a given hexadecimal string
fn remove_prefix(input: &str) -> &str {
    input.strip_prefix("0x").unwrap_or(input)
}

/// Converts a hexadecimal string to an H256 value
fn string_to_h256(hex_str: &str) -> Result<H256, String> {
    let bytes =
        Vec::from_hex(remove_prefix(hex_str)).map_err(|e| format!("Failed to convert hex string to bytes: {:?}", e))?;
    if bytes.len() == 32 { Ok(H256::from_slice(&bytes)) } else { Err(format!("Invalid input length: {}", bytes.len())) }
}

/// Converts a hexadecimal string to a U256 value
fn string_to_u256(hex_str: &str) -> Result<U256, String> {
    let bytes =
        Vec::from_hex(remove_prefix(hex_str)).map_err(|e| format!("Failed to convert hex string to bytes: {:?}", e))?;
    if bytes.len() == 32 {
        Ok(U256::from_big_endian(&bytes))
    } else {
        Err(format!("Invalid input length: {}", bytes.len()))
    }
}

// Deserialization and Conversion for JSON Transactions, Events, and CallEntryPoints
/// Struct for deserializing CallEntryPoint from JSON
#[derive(Debug, Serialize, Deserialize)]
pub struct DeserializeCallEntrypoint {
    /// The class hash
    pub class_hash: Option<String>,
    /// The entrypoint type
    pub entrypoint_type: String,
    /// The entrypoint selector
    /// An invoke transaction without an entry point selector invokes the 'execute' function.
    pub entrypoint_selector: Option<String>,
    /// The Calldata
    pub calldata: Vec<String>,
    /// The storage address
    pub storage_address: String,
    /// The caller address
    pub caller_address: String,
}

/// Error enum for CallEntrypoint deserialization
#[derive(Debug)]
pub enum DeserializeCallEntrypointError {
    /// InvalidClassHash error
    InvalidClassHash(FromHexError),
    /// InvalidCalldata error
    InvalidCalldata(String),
    /// InvalidEntrypointSelector error
    InvalidEntrypointSelector(String),
    /// InvalidEntryPointType error
    InvalidEntryPointType,
    /// CalldataExceedsMaxSize error
    CalldataExceedsMaxSize,
    /// InvalidStorageAddress error
    InvalidStorageAddress(FromHexError),
    /// InvalidCallerAddress error
    InvalidCallerAddress(FromHexError),
}

impl fmt::Display for DeserializeCallEntrypointError {
    /// Implementation of `fmt::Display` for `DeserializeCallEntrypointError`
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeserializeCallEntrypointError::InvalidClassHash(s) => write!(f, "Invalid class hash format: {:?}", s),
            DeserializeCallEntrypointError::InvalidCalldata(s) => write!(f, "Invalid calldata format: {:?}", s),
            DeserializeCallEntrypointError::InvalidEntrypointSelector(s) => {
                write!(f, "Invalid entrypoint_type selector: ${:?}", s)
            }
            DeserializeCallEntrypointError::InvalidEntryPointType => write!(f, "Invalid entrypoint_type"),
            DeserializeCallEntrypointError::CalldataExceedsMaxSize => write!(f, "Calldata exceed max size"),
            DeserializeCallEntrypointError::InvalidStorageAddress(e) => {
                write!(f, "Invalid storage_address format: {:?}", e)
            }
            DeserializeCallEntrypointError::InvalidCallerAddress(e) => {
                write!(f, "Invalid caller_address format: {:?}", e)
            }
        }
    }
}

/// Struct for deserializing Event from JSON
#[derive(Debug, Serialize, Deserialize)]
pub struct DeserializeEventWrapper {
    /// The keys (topics) of the event.
    pub keys: Vec<String>,
    /// The data of the event.
    pub data: Vec<String>,
    /// The address that emitted the event
    pub from_address: String,
}

/// Error enum for Event deserialization
#[derive(Debug)]
pub enum DeserializeEventError {
    /// InvalidKeys error
    InvalidKeys(String),
    /// KeysExceedMaxSize error
    KeysExceedMaxSize,
    /// InvalidData error
    InvalidData(String),
    /// DataExceedMaxSize error
    DataExceedMaxSize,
    /// InvalidFromAddress error
    InvalidFromAddress(FromHexError),
}

impl fmt::Display for DeserializeEventError {
    /// Implementation of `fmt::Display` for `DeserializeEventError`
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeserializeEventError::InvalidKeys(s) => write!(f, "Invalid keys format: {:?}", s),
            DeserializeEventError::KeysExceedMaxSize => write!(f, "Keys exceed max size"),
            DeserializeEventError::InvalidData(s) => write!(f, "Invalid data format: ${:?}", s),
            DeserializeEventError::DataExceedMaxSize => write!(f, "Data exceed max size"),
            DeserializeEventError::InvalidFromAddress(e) => write!(f, "Invalid data format: ${:?}", e),
        }
    }
}

/// Struct for deserializing Transaction from JSON
#[derive(Debug, Serialize, Deserialize)]
pub struct DeserializeTransaction {
    /// The version of the transaction
    pub version: u8,
    /// Transaction hash.
    pub hash: String,
    /// Signature
    pub signature: Vec<String>,
    /// Events
    pub events: Vec<DeserializeEventWrapper>,
    /// Sender Address
    pub sender_address: String,
    /// Nonce
    pub nonce: u64,
    /// Call entrypoint
    pub call_entrypoint: DeserializeCallEntrypoint,
}

/// Error enum for Transaction deserialization
#[derive(Debug)]
pub enum DeserializeTransactionError {
    /// FailedToParse error
    FailedToParse(String),
    /// InvalidHash error
    InvalidHash(String),
    /// InvalidSignature error
    InvalidSignature(String),
    /// SignatureExceedsMaxSize error
    SignatureExceedsMaxSize,
    /// InvalidEvents error
    InvalidEvents(DeserializeEventError),
    /// EventsExceedMaxSize error
    EventsExceedMaxSize,
    /// InvalidSenderAddress error
    InvalidSenderAddress(FromHexError),
    /// InvalidCallEntryPoint error
    InvalidCallEntryPoint(DeserializeCallEntrypointError),
}

impl fmt::Display for DeserializeTransactionError {
    /// Implementation of `fmt::Display` for `DeserializeTransactionError`
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeserializeTransactionError::FailedToParse(s) => write!(f, "Failed parse json: {:?}", s),
            DeserializeTransactionError::InvalidHash(s) => write!(f, "Invalid hash format: {:?}", s),
            DeserializeTransactionError::InvalidSignature(s) => write!(f, "Invalid signature format: {:?}", s),
            DeserializeTransactionError::SignatureExceedsMaxSize => write!(f, "Signature exceed max size"),
            DeserializeTransactionError::InvalidEvents(e) => write!(f, "Invalid events format: {:?}", e),
            DeserializeTransactionError::EventsExceedMaxSize => write!(f, "Events exceed max size"),
            DeserializeTransactionError::InvalidSenderAddress(e) => write!(f, "Invalid sender address format: {:?}", e),
            DeserializeTransactionError::InvalidCallEntryPoint(e) => {
                write!(f, "Invalid call_entry_point format: {:?}", e)
            }
        }
    }
}

/// Implementation of `TryFrom<DeserializeTransaction>` for `Transaction`.
///
/// Converts a `DeserializeTransaction` into a `Transaction`, performing necessary validations
/// and transformations on the input data.
impl TryFrom<DeserializeTransaction> for Transaction {
    type Error = DeserializeTransactionError;

    /// Converts a `DeserializeTransaction` into a `Transaction`.
    ///
    /// Returns a `DeserializeTransactionError` variant if any field fails validation or conversion.
    fn try_from(d: DeserializeTransaction) -> Result<Self, Self::Error> {
        // Convert version to u8
        let version = d.version;

        // Convert hash to H256
        let hash = string_to_h256(&d.hash).map_err(DeserializeTransactionError::InvalidHash)?;

        // Convert signatures to BoundedVec<H256, MaxArraySize> and check if it exceeds max size
        let signature = d
            .signature
            .into_iter()
            .map(|s| string_to_h256(&s).map_err(DeserializeTransactionError::InvalidSignature))
            .collect::<Result<Vec<H256>, DeserializeTransactionError>>()?;
        let signature = BoundedVec::<H256, MaxArraySize>::try_from(signature)
            .map_err(|_| DeserializeTransactionError::SignatureExceedsMaxSize)?;

        // Convert sender_address to ContractAddressWrapper
        let sender_address = ContractAddressWrapper::from_hex(remove_prefix(&d.sender_address))
            .map_err(DeserializeTransactionError::InvalidSenderAddress)?;

        // Convert nonce to U256
        let nonce = U256::from(d.nonce);

        // Convert call_entrypoint to CallEntryPointWrapper
        let call_entrypoint = CallEntryPointWrapper::try_from(d.call_entrypoint)
            .map_err(DeserializeTransactionError::InvalidCallEntryPoint)?;

        // Create Transaction with validated and converted fields
        Ok(Self { version, hash, signature, sender_address, nonce, call_entrypoint, ..Transaction::default() })
    }
}

/// Implementation of `TryFrom<DeserializeCallEntrypoint>` for `CallEntryPointWrapper`.
///
/// Converts a `DeserializeCallEntrypoint` into a `CallEntryPointWrapper`, performing necessary
/// validations and transformations on the input data.
impl TryFrom<DeserializeCallEntrypoint> for CallEntryPointWrapper {
    type Error = DeserializeCallEntrypointError;

    /// Converts a `DeserializeCallEntrypoint` into a `CallEntryPointWrapper`.
    ///
    /// Returns a `DeserializeCallEntrypointError` variant if any field fails validation or
    /// conversion.
    fn try_from(d: DeserializeCallEntrypoint) -> Result<Self, Self::Error> {
        // Convert class_hash to Option<[u8; 32]> if present
        let class_hash = match d.class_hash {
            Some(hash) => Some(
                <[u8; 32]>::from_hex(remove_prefix(&hash)).map_err(DeserializeCallEntrypointError::InvalidClassHash)?,
            ),
            None => None,
        };

        // Convert entrypoint_type to EntryPointTypeWrapper
        let entrypoint_type = match d.entrypoint_type.as_str() {
            "Constructor" => EntryPointTypeWrapper::Constructor,
            "External" => EntryPointTypeWrapper::External,
            "L1Handler" => EntryPointTypeWrapper::L1Handler,
            _ => return Err(DeserializeCallEntrypointError::InvalidEntryPointType),
        };

        // Convert entrypoint_selector to Option<H256> if present
        let entrypoint_selector = match d.entrypoint_selector {
            Some(selector) => {
                Some(string_to_h256(&selector).map_err(DeserializeCallEntrypointError::InvalidEntrypointSelector)?)
            }
            None => None,
        };

        // Convert calldata to BoundedVec<U256, MaxArraySize> and check if it exceeds max size
        let calldata: Result<Vec<U256>, DeserializeCallEntrypointError> = d
            .calldata
            .into_iter()
            .map(|hex_str| string_to_u256(&hex_str).map_err(DeserializeCallEntrypointError::InvalidCalldata))
            .collect();
        let calldata = BoundedVec::<U256, MaxArraySize>::try_from(calldata?)
            .map_err(|_| DeserializeCallEntrypointError::CalldataExceedsMaxSize)?;

        // Convert storage_address to [u8; 32]
        let storage_address = <[u8; 32]>::from_hex(remove_prefix(&d.storage_address))
            .map_err(DeserializeCallEntrypointError::InvalidStorageAddress)?;

        // Convert caller_address to [u8; 32]
        let caller_address = <[u8; 32]>::from_hex(remove_prefix(&d.caller_address))
            .map_err(DeserializeCallEntrypointError::InvalidCallerAddress)?;

        // Create CallEntryPointWrapper with validated and converted fields
        Ok(Self { class_hash, entrypoint_type, entrypoint_selector, calldata, storage_address, caller_address })
    }
}

/// Implementation of `TryFrom<DeserializeEventWrapper>` for `EventWrapper`.
///
/// Converts a `DeserializeEventWrapper` into an `EventWrapper`, performing necessary validations
/// and transformations on the input data.
impl TryFrom<DeserializeEventWrapper> for EventWrapper {
    type Error = DeserializeEventError;

    /// Converts a `DeserializeEventWrapper` into an `EventWrapper`.
    ///
    /// Returns a `DeserializeEventError` variant if any field fails validation or conversion.
    fn try_from(d: DeserializeEventWrapper) -> Result<Self, Self::Error> {
        // Convert keys to BoundedVec<H256, MaxArraySize> and check if it exceeds max size
        let keys: Result<Vec<H256>, DeserializeEventError> = d
            .keys
            .into_iter()
            .map(|hex_str| string_to_h256(&hex_str).map_err(DeserializeEventError::InvalidKeys))
            .collect();
        let keys =
            BoundedVec::<H256, MaxArraySize>::try_from(keys?).map_err(|_| DeserializeEventError::KeysExceedMaxSize)?;

        // Convert data to BoundedVec<H256, MaxArraySize> and check if it exceeds max size
        let data: Result<Vec<H256>, DeserializeEventError> = d
            .data
            .into_iter()
            .map(|hex_str| string_to_h256(&hex_str).map_err(DeserializeEventError::InvalidData))
            .collect();
        let data =
            BoundedVec::<H256, MaxArraySize>::try_from(data?).map_err(|_| DeserializeEventError::DataExceedMaxSize)?;

        // Convert from_address to [u8; 32]
        let from_address: [u8; 32] =
            <[u8; 32]>::from_hex(remove_prefix(&d.from_address)).map_err(DeserializeEventError::InvalidFromAddress)?;

        // Create EventWrapper with validated and converted fields
        Ok(Self { keys, data, from_address })
    }
}

/// Create a `Transaction` from a JSON string and an optional contract content.
///
/// This function takes a JSON string (`json_str`) and a byte slice (`contract_content`) containing
/// the contract content, if available.
///
/// If `contract_content` is not empty, the function will use it to set the `contract_class`
/// field of the resulting `Transaction` object. Otherwise, the `contract_class` field will be set
/// to `None`.
///
/// Returns a `DeserializeTransactionError` if JSON deserialization fails, or if the deserialized
/// object fails to convert into a `Transaction`.
pub fn transaction_from_json(
    json_str: &str,
    contract_content: &'static [u8],
) -> Result<Transaction, DeserializeTransactionError> {
    // Deserialize the JSON string into a DeserializeTransaction and convert it into a Transaction
    let deserialized_transaction: DeserializeTransaction =
        serde_json::from_str(json_str).map_err(|e| DeserializeTransactionError::FailedToParse(format!("{:?}", e)))?;
    let mut transaction = Transaction::try_from(deserialized_transaction)?;

    // Set the contract_class field based on contract_content
    if !contract_content.is_empty() {
        transaction.contract_class = Some(ContractClassWrapper::from(get_contract_class(contract_content)));
    } else {
        transaction.contract_class = None;
    }

    Ok(transaction)
}
