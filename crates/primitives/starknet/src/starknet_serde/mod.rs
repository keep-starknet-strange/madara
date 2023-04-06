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

fn remove_prefix(input: &str) -> &str {
    input.strip_prefix("0x").unwrap_or(input)
}

fn string_to_h256(hex_str: &str) -> Result<H256, String> {
    let bytes =
        Vec::from_hex(remove_prefix(hex_str)).map_err(|e| format!("Failed to convert hex string to bytes: {:?}", e))?;
    if bytes.len() == 32 { Ok(H256::from_slice(&bytes)) } else { Err(format!("Invalid input length: {}", bytes.len())) }
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
    /// The address that emited the event
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

// Implement TryFrom trait to convert DeserializeTransaction to Transaction
impl TryFrom<DeserializeTransaction> for Transaction {
    type Error = DeserializeTransactionError;

    fn try_from(d: DeserializeTransaction) -> Result<Self, Self::Error> {
        let version = U256::from(d.version);

        let hash = string_to_h256(&d.hash).map_err(|e| DeserializeTransactionError::InvalidHash(e))?;

        let signature = d
            .signature
            .into_iter()
            .map(|s| string_to_h256(&s).map_err(|e| DeserializeTransactionError::InvalidSignature(e)))
            .collect::<Result<Vec<H256>, DeserializeTransactionError>>()?;
        let signature = BoundedVec::<H256, MaxArraySize>::try_from(signature)
            .map_err(|_| DeserializeTransactionError::SignatureExceedsMaxSize)?;

        let events = d
            .events
            .into_iter()
            .map(EventWrapper::try_from)
            .collect::<Result<Vec<EventWrapper>, DeserializeEventError>>()
            .map_err(|e| DeserializeTransactionError::InvalidEvents(e))?;
        let events = BoundedVec::<EventWrapper, MaxArraySize>::try_from(events)
            .map_err(|_| DeserializeTransactionError::EventsExceedMaxSize)?;

        let sender_address = ContractAddressWrapper::from_hex(remove_prefix(&d.sender_address))
            .map_err(|e| DeserializeTransactionError::InvalidSenderAddress(e))?;

        let nonce = U256::from(d.nonce);

        let call_entrypoint = CallEntryPointWrapper::try_from(d.call_entrypoint)
            .map_err(|e| DeserializeTransactionError::InvalidCallEntryPoint(e))?;

        Ok(Self { version, hash, signature, events, sender_address, nonce, call_entrypoint, ..Transaction::default() })
    }
}

/// Implement TryFrom trait to convert DeserializeCallEntrypoint to CallEntryPointWrapper
impl TryFrom<DeserializeCallEntrypoint> for CallEntryPointWrapper {
    type Error = DeserializeCallEntrypointError;

    fn try_from(d: DeserializeCallEntrypoint) -> Result<Self, Self::Error> {
        let class_hash = match d.class_hash {
            Some(hash) => Some(
                <[u8; 32]>::from_hex(remove_prefix(&hash))
                    .map_err(|e| DeserializeCallEntrypointError::InvalidClassHash(e))?,
            ),
            None => None,
        };

        let entrypoint_type = match d.entrypoint_type.as_str() {
            "Constructor" => EntryPointTypeWrapper::Constructor,
            "External" => EntryPointTypeWrapper::External,
            "L1Handler" => EntryPointTypeWrapper::L1Handler,
            _ => return Err(DeserializeCallEntrypointError::InvalidEntryPointType),
        };

        let entrypoint_selector = match d.entrypoint_selector {
            Some(selector) => Some(
                string_to_h256(&selector).map_err(|e| DeserializeCallEntrypointError::InvalidEntrypointSelector(e))?,
            ),
            None => None,
        };

        let calldata: Result<Vec<H256>, DeserializeCallEntrypointError> = d
            .calldata
            .into_iter()
            .map(|hex_str| string_to_h256(&hex_str).map_err(|e| DeserializeCallEntrypointError::InvalidCalldata(e)))
            .collect();
        let calldata = BoundedVec::<H256, MaxArraySize>::try_from(calldata?)
            .map_err(|_| DeserializeCallEntrypointError::CalldataExceedsMaxSize)?;

        let storage_address = <[u8; 32]>::from_hex(remove_prefix(&d.storage_address))
            .map_err(|e| DeserializeCallEntrypointError::InvalidStorageAddress(e))?;

        let caller_address = <[u8; 32]>::from_hex(remove_prefix(&d.caller_address))
            .map_err(|e| DeserializeCallEntrypointError::InvalidCallerAddress(e))?;

        Ok(Self { class_hash, entrypoint_type, entrypoint_selector, calldata, storage_address, caller_address })
    }
}

// Implement TryFrom trait to convert DeserializeEventWrapper to EventWrapper
impl TryFrom<DeserializeEventWrapper> for EventWrapper {
    type Error = DeserializeEventError;

    fn try_from(d: DeserializeEventWrapper) -> Result<Self, Self::Error> {
        let keys: Result<Vec<H256>, DeserializeEventError> = d
            .keys
            .into_iter()
            .map(|hex_str| string_to_h256(&hex_str).map_err(|e| DeserializeEventError::InvalidKeys(e)))
            .collect();
        let keys =
            BoundedVec::<H256, MaxArraySize>::try_from(keys?).map_err(|_| DeserializeEventError::KeysExceedMaxSize)?;

        let data: Result<Vec<H256>, DeserializeEventError> = d
            .data
            .into_iter()
            .map(|hex_str| string_to_h256(&hex_str).map_err(|e| DeserializeEventError::InvalidData(e)))
            .collect();
        let data =
            BoundedVec::<H256, MaxArraySize>::try_from(data?).map_err(|_| DeserializeEventError::DataExceedMaxSize)?;

        let from_address: [u8; 32] = <[u8; 32]>::from_hex(remove_prefix(&d.from_address))
            .map_err(|e| DeserializeEventError::InvalidFromAddress(e))?;

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
pub fn transaction_from_json(
    json_str: &str,
    contract_content: &'static [u8],
) -> Result<Transaction, DeserializeTransactionError> {
    let deserialized_transaction: DeserializeTransaction =
        serde_json::from_str(json_str).map_err(|e| DeserializeTransactionError::FailedToParse(format!("{:?}", e)))?;
    let mut transaction = Transaction::try_from(deserialized_transaction)?;

    if !contract_content.is_empty() {
        transaction.contract_class = Some(ContractClassWrapper::from(get_contract_class(contract_content)));
    } else {
        transaction.contract_class = None;
    }

    Ok(transaction)
}
