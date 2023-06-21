//! This module contains the serialization and deserialization functions for the StarkNet types.
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use blockifier::execution::contract_class::ContractClass;
use frame_support::BoundedVec;
use serde::{Deserialize, Serialize};
use sp_core::U256;
use thiserror_no_std::Error;

use crate::execution::types::{
    CallEntryPointWrapper, ContractClassWrapper, EntryPointTypeWrapper, Felt252Wrapper, Felt252WrapperError,
    MaxCalldataSize,
};
use crate::transaction::types::{EventWrapper, MaxArraySize, Transaction};

/// Removes the "0x" prefix from a given hexadecimal string
fn remove_prefix(input: &str) -> &str {
    input.strip_prefix("0x").unwrap_or(input)
}

/// Converts a hexadecimal string to an Felt252Wrapper value
fn string_to_felt(hex_str: &str) -> Result<Felt252Wrapper, String> {
    match Felt252Wrapper::from_hex_be(hex_str) {
        Ok(f) => Ok(f),
        Err(e) => Err(e.to_string()),
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
    /// The initial gas
    pub initial_gas: String,
}

/// Error enum for CallEntrypoint deserialization
#[derive(Debug, Error)]
pub enum DeserializeCallEntrypointError {
    /// InvalidClassHash error
    #[error("Invalid class hash format: {0}")]
    InvalidClassHash(Felt252WrapperError),
    /// InvalidCalldata error
    #[error("Invalid calldata format: {0}")]
    InvalidCalldata(String),
    /// InvalidEntrypointSelector error
    #[error("Invalid entrypoint_type selector: {0}")]
    InvalidEntrypointSelector(String),
    /// InvalidEntryPointType error
    #[error("Invalid entrypoint_type")]
    InvalidEntryPointType,
    /// CalldataExceedsMaxSize error
    #[error("Calldata exceed max size")]
    CalldataExceedsMaxSize,
    /// InvalidStorageAddress error
    #[error("Invalid storage_address format: {0:?}")]
    InvalidStorageAddress(Felt252WrapperError),
    /// InvalidCallerAddress error
    #[error("Invalid caller_address format: {0:?}")]
    InvalidCallerAddress(Felt252WrapperError),
    /// InvalidCallerAddress error
    #[error("Invalid initial_gas format: {0:?}")]
    InvalidInitialGas(Felt252WrapperError),
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
    /// The transaction hash that emitted the event
    pub transaction_hash: String,
}

/// Error enum for Event deserialization
#[derive(Debug, Error)]
pub enum DeserializeEventError {
    /// InvalidKeys error
    #[error("Invalid keys format: {0}")]
    InvalidKeys(String),
    /// KeysExceedMaxSize error
    #[error("Keys exceed max size")]
    KeysExceedMaxSize,
    /// InvalidData error
    #[error("Invalid data format: {0}")]
    InvalidData(String),
    /// DataExceedMaxSize error
    #[error("Data exceed max size")]
    DataExceedMaxSize,
    /// InvalidFelt252 error
    #[error(transparent)]
    InvalidFelt252(#[from] Felt252WrapperError),
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
#[derive(Debug, Error)]
pub enum DeserializeTransactionError {
    /// FailedToParse error
    #[error("Failed to parse json: {0}")]
    FailedToParse(String),
    /// InvalidHash error
    #[error("Invalid hash format: {0}")]
    InvalidHash(String),
    /// InvalidSignature error
    #[error("Invalid signature format: {0}")]
    InvalidSignature(String),
    /// SignatureExceedsMaxSize error
    #[error("Signature exceed max size")]
    SignatureExceedsMaxSize,
    /// InvalidEvents error
    #[error(transparent)]
    InvalidEvents(#[from] DeserializeEventError),
    /// EventsExceedMaxSize error
    #[error("Events exceed max size")]
    EventsExceedMaxSize,
    /// InvalidSenderAddress error
    #[error("Invalid sender address format: {0}")]
    InvalidSenderAddress(String),
    /// InvalidCallEntryPoint error
    #[error(transparent)]
    InvalidCallEntryPoint(#[from] DeserializeCallEntrypointError),
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

        // Convert hash to Felt252Wrapper
        let hash = string_to_felt(&d.hash).map_err(DeserializeTransactionError::InvalidHash)?;

        // Convert signatures to BoundedVec<Felt252Wrapper, MaxArraySize> and check if it exceeds max size
        let signature = d
            .signature
            .into_iter()
            .map(|s| string_to_felt(&s).map_err(DeserializeTransactionError::InvalidSignature))
            .collect::<Result<Vec<Felt252Wrapper>, DeserializeTransactionError>>()?;
        let signature = BoundedVec::<Felt252Wrapper, MaxArraySize>::try_from(signature)
            .map_err(|_| DeserializeTransactionError::SignatureExceedsMaxSize)?;

        // Convert sender_address to ContractAddressWrapper
        let sender_address = string_to_felt(remove_prefix(&d.sender_address))
            .map_err(DeserializeTransactionError::InvalidSenderAddress)?;

        // Convert nonce to U256
        let nonce = Felt252Wrapper::try_from(U256::from(d.nonce)).unwrap();

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
        // Convert class_hash to Option<Felt252Wrapper> if present
        let class_hash = match d.class_hash {
            Some(hash_str) => match Felt252Wrapper::from_hex_be(hash_str.as_str()) {
                Ok(felt) => Some(felt),
                Err(e) => return Err(DeserializeCallEntrypointError::InvalidClassHash(e)),
            },
            None => None,
        };

        // Convert entrypoint_type to EntryPointTypeWrapper
        let entrypoint_type = match d.entrypoint_type.as_str() {
            "Constructor" => EntryPointTypeWrapper::Constructor,
            "External" => EntryPointTypeWrapper::External,
            "L1Handler" => EntryPointTypeWrapper::L1Handler,
            _ => return Err(DeserializeCallEntrypointError::InvalidEntryPointType),
        };

        // Convert entrypoint_selector to Option<Felt252Wrapper> if present
        let entrypoint_selector = match d.entrypoint_selector {
            Some(selector) => {
                Some(string_to_felt(&selector).map_err(DeserializeCallEntrypointError::InvalidEntrypointSelector)?)
            }
            None => None,
        };

        // Convert calldata to BoundedVec<Felt252Wrapper, MaxArraySize> and check if it exceeds max size
        let calldata: Result<Vec<Felt252Wrapper>, DeserializeCallEntrypointError> = d
            .calldata
            .into_iter()
            .map(|hex_str| string_to_felt(&hex_str).map_err(DeserializeCallEntrypointError::InvalidCalldata))
            .collect();
        let calldata = BoundedVec::<Felt252Wrapper, MaxCalldataSize>::try_from(calldata?)
            .map_err(|_| DeserializeCallEntrypointError::CalldataExceedsMaxSize)?;

        // Convert storage_address to Felt252Wrapper
        let storage_address = match Felt252Wrapper::from_hex_be(d.storage_address.as_str()) {
            Ok(felt) => felt,
            Err(e) => return Err(DeserializeCallEntrypointError::InvalidStorageAddress(e)),
        };

        // Convert caller_address to Felt252Wrapper
        let caller_address = match Felt252Wrapper::from_hex_be(d.caller_address.as_str()) {
            Ok(felt) => felt,
            Err(e) => return Err(DeserializeCallEntrypointError::InvalidCallerAddress(e)),
        };

        let initial_gas = match Felt252Wrapper::from_hex_be(d.initial_gas.as_str()) {
            Ok(felt) => felt,
            Err(e) => return Err(DeserializeCallEntrypointError::InvalidInitialGas(e)),
        };

        // Create CallEntryPointWrapper with validated and converted fields
        Ok(Self {
            class_hash,
            entrypoint_type,
            entrypoint_selector,
            calldata,
            storage_address,
            caller_address,
            initial_gas,
        })
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
        // Convert keys to BoundedVec<Felt252Wrapper, MaxArraySize> and check if it exceeds max size
        let keys: Result<Vec<Felt252Wrapper>, DeserializeEventError> = d
            .keys
            .into_iter()
            .map(|hex_str| string_to_felt(&hex_str).map_err(DeserializeEventError::InvalidKeys))
            .collect();
        let keys = BoundedVec::<Felt252Wrapper, MaxArraySize>::try_from(keys?)
            .map_err(|_| DeserializeEventError::KeysExceedMaxSize)?;

        // Convert data to BoundedVec<Felt252Wrapper, MaxArraySize> and check if it exceeds max size
        let data: Result<Vec<Felt252Wrapper>, DeserializeEventError> = d
            .data
            .into_iter()
            .map(|hex_str| string_to_felt(&hex_str).map_err(DeserializeEventError::InvalidData))
            .collect();
        let data = BoundedVec::<Felt252Wrapper, MaxArraySize>::try_from(data?)
            .map_err(|_| DeserializeEventError::DataExceedMaxSize)?;

        // Convert from_address to [u8; 32]
        let from_address = match Felt252Wrapper::from_hex_be(d.from_address.as_str()) {
            Ok(felt) => felt,
            Err(e) => return Err(DeserializeEventError::InvalidFelt252(e)),
        };

        let transaction_hash = match Felt252Wrapper::from_hex_be(d.transaction_hash.as_str()) {
            Ok(felt) => felt,
            Err(e) => return Err(DeserializeEventError::InvalidFelt252(e)),
        };

        // Create EventWrapper with validated and converted fields
        Ok(Self { keys, data, from_address, transaction_hash })
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
        // FIXME 707
        let raw_contract_class: ContractClass =
            ContractClass::V0(serde_json::from_slice(contract_content).map_err(|e| {
                DeserializeTransactionError::FailedToParse(format!("invalid contract content: {:?}", e))
            })?);
        transaction.contract_class =
            Some(ContractClassWrapper::try_from(raw_contract_class).map_err(|e| {
                DeserializeTransactionError::FailedToParse(format!("invalid contract content: {:?}", e))
            })?);
    } else {
        transaction.contract_class = None;
    }

    Ok(transaction)
}
