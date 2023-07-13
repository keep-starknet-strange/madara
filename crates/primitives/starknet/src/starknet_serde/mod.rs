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
use crate::transaction::types::{
    DeclareTransaction, DeclareTransactionV1, DeclareTransactionV2, DeployAccountTransaction, EventWrapper,
    InvokeTransaction, InvokeTransactionV0, InvokeTransactionV1, MaxArraySize, Transaction,
};

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
    /// InvalidCasmClassHash error
    #[error("Invalid casm class hash format: {0}")]
    InvalidCasmClassHash(Felt252WrapperError),
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
pub enum DeserializeTransaction {
    /// Invoke deserialize transaction
    #[serde(rename = "INVOKE")]
    Invoke(DeserializeInvokeTransaction),
    /// Declare deserialize transaction
    #[serde(rename = "DECLARE")]
    Declare(DeserializeDeclareTransaction),
    /// DeployAccount deserialize transaction
    #[serde(rename = "DEPLOY_ACCOUNT")]
    DeployAccount(DeserializeDeployAccountTransaction),
}

/// Struct for deserializing InvokeTransaction from JSON
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "version")]
pub enum DeserializeInvokeTransaction {
    #[serde(rename = "0x0")]
    V0(DeserializeInvokeTransactionV0),
    #[serde(rename = "0x1")]
    V1(DeserializeInvokeTransactionV1),
}

/// Struct for deserializing InvokeTransactionV0 from JSON
#[derive(Debug, Serialize, Deserialize)]
pub struct DeserializeInvokeTransactionV0 {
    /// The hash identifying the transaction
    pub transaction_hash: String,
    /// The maximal fee that can be charged for including the transaction
    pub max_fee: u64,
    /// Signature
    pub signature: Vec<String>,
    /// Nonce
    pub nonce: u64,
    /// Contract address
    pub contract_address: String,
    /// Entry point selector
    pub entry_point_selector: String,
    /// The parameters passed to the function
    pub calldata: Vec<String>,
}

/// Struct for deserializing InvokeTransactionV1 from JSON
#[derive(Debug, Serialize, Deserialize)]
pub struct DeserializeInvokeTransactionV1 {
    /// The hash identifying the transaction
    pub transaction_hash: String,
    /// The maximal fee that can be charged for including the transaction
    pub max_fee: u64,
    /// Signature
    pub signature: Vec<String>,
    /// Nonce
    pub nonce: u64,
    /// Sender address
    pub sender_address: String,
    /// The data expected by the account’s `execute` function (in most usecases, this includes the
    /// called contract address and a function selector)
    pub calldata: Vec<String>,
}

/// Implement getters for DeserializeInvokeTransaction
macro_rules! implement_invoke_tx_getters {
    ($(($field:ident, $field_type:ty)),*) => {
        $(pub fn $field(&self) -> $field_type {
            match self {
                Self::V0(tx) => tx.$field.clone(),
                Self::V1(tx) => tx.$field.clone(),
            }
        })*
    };
}

impl DeserializeInvokeTransaction {
    implement_invoke_tx_getters!(
        (transaction_hash, String),
        (nonce, u64),
        (max_fee, u64),
        (signature, Vec<String>),
        (calldata, Vec<String>)
    );

    pub fn sender_address(&self) -> String {
        match self {
            DeserializeInvokeTransaction::V0(tx) => tx.contract_address.clone(),
            DeserializeInvokeTransaction::V1(tx) => tx.sender_address.clone(),
        }
    }
}

/// Struct for deserializing DeclareTransaction from JSON
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "version")]
pub enum DeserializeDeclareTransaction {
    #[serde(rename = "0x1")]
    V1(DeserializeDeclareTransactionV1),
    #[serde(rename = "0x2")]
    V2(DeserializeDeclareTransactionV2),
}

/// Struct for deserializing DeclareTransactionV1 from JSON
#[derive(Debug, Serialize, Deserialize)]
pub struct DeserializeDeclareTransactionV1 {
    /// The hash identifying the transaction
    pub transaction_hash: String,
    /// The maximal fee that can be charged for including the transaction
    pub max_fee: u64,
    /// Signaturecargo
    pub signature: Vec<String>,
    // Transaction nonce
    pub nonce: u64,
    /// Contract to declare.
    pub contract_class: String,
    /// The hash of the declared class
    pub class_hash: String,
    /// The address of the account contract sending the declaration transaction
    pub sender_address: String,
}

/// Struct for deserializing DeclareTransactionV1 from JSON
#[derive(Debug, Serialize, Deserialize)]
pub struct DeserializeDeclareTransactionV2 {
    /// The hash identifying the transaction
    pub transaction_hash: String,
    /// The maximal fee that can be charged for including the transaction
    pub max_fee: u64,
    /// Signaturecargo
    pub signature: Vec<String>,
    pub nonce: u64,
    /// The contract class
    pub contract_class: String,
    /// The hash of the declared sierra class
    pub class_hash: String,
    /// The hash of the compiled
    pub compiled_class_hash: String,
    /// The address of the account contract sending the declaration transaction
    pub sender_address: String,
}

// From https://github.com/tdelabro/starknet-api/blob/main/src/transaction.rs
macro_rules! implement_declare_tx_getters {
    ($(($field:ident, $field_type:ty)),*) => {
        $(pub fn $field(&self) -> $field_type {
            match self {
                Self::V1(tx) => tx.$field.clone(),
                Self::V2(tx) => tx.$field.clone(),
            }
        })*
    };
}

impl DeserializeDeclareTransaction {
    implement_declare_tx_getters!(
        (transaction_hash, String),
        (class_hash, String),
        (nonce, u64),
        (sender_address, String),
        (max_fee, u64),
        (signature, Vec<String>),
        (contract_class, String)
    );
}

/// Struct for deserializing InvokeTransaction from JSON
#[derive(Debug, Serialize, Deserialize)]
pub struct DeserializeDeployAccountTransaction {
    /// The hash identifying the transaction
    pub transaction_hash: String,
    /// The maximal fee that can be charged for including the transaction
    pub max_fee: u64,
    /// Version of the transaction scheme
    pub version: u64,
    /// Signature
    pub signature: Vec<String>,
    /// Nonce
    pub nonce: u64,
    /// The salt for the address of the deployed contract
    pub contract_address_salt: String,
    /// The parameters passed to the constructor
    pub constructor_calldata: Vec<String>,
    /// The hash of the deployed contract’s class
    pub class_hash: String,
    /// The address of the account contract being deployed
    pub sender_address: String,
}

/// Error enum for Transaction deserialization
#[derive(Debug, Error)]
pub enum DeserializeTransactionError {
    /// InvalidClassHash error
    #[error("Invalid class hash format: {0}")]
    InvalidClassHash(Felt252WrapperError),
    /// InvalidCasmClassHash error
    #[error("Invalid casm class hash format: {0}")]
    InvalidCasmClassHash(Felt252WrapperError),
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

/// Implementation of `TryFrom<DeserializeInvokeTransaction>` for `InvokeTransaction`.
///
/// Converts a `DeserializeInvokeTransaction` into a `InvokeTransaction`, performing necessary
/// validations and transformations on the input data.
impl TryFrom<DeserializeInvokeTransaction> for InvokeTransaction {
    type Error = DeserializeTransactionError;

    /// Converts a `DeserializeTransaction` into a `Transaction`.
    ///
    /// Returns a `DeserializeTransactionError` variant if any field fails validation or conversion.
    fn try_from(d: DeserializeInvokeTransaction) -> Result<Self, Self::Error> {
        // Convert hash to Felt252Wrapper
        let transaction_hash =
            string_to_felt(&d.transaction_hash()).map_err(DeserializeTransactionError::InvalidHash)?;

        // Convert signatures to BoundedVec<Felt252Wrapper, MaxArraySize> and check if it exceeds max size
        let signature = d
            .signature()
            .into_iter()
            .map(|s| string_to_felt(&s).map_err(DeserializeTransactionError::InvalidSignature))
            .collect::<Result<Vec<Felt252Wrapper>, DeserializeTransactionError>>()?;
        let signature = BoundedVec::<Felt252Wrapper, MaxArraySize>::try_from(signature)
            .map_err(|_| DeserializeTransactionError::SignatureExceedsMaxSize)?;

        // Convert sender_address to ContractAddressWrapper
        let sender_address = string_to_felt(remove_prefix(&d.sender_address()))
            .map_err(DeserializeTransactionError::InvalidSenderAddress)?;

        // Convert nonce to U256
        let nonce = Felt252Wrapper::try_from(U256::from(d.nonce())).unwrap();

        // Convert max_fee to U256
        let max_fee = Felt252Wrapper::try_from(U256::from(d.max_fee())).unwrap();

        // Convert calldata to BoundedVec<Felt252Wrapper, MaxArraySize> and check if it exceeds max size
        let calldata: Result<Vec<Felt252Wrapper>, DeserializeCallEntrypointError> = d
            .calldata()
            .into_iter()
            .map(|hex_str| string_to_felt(&hex_str).map_err(DeserializeCallEntrypointError::InvalidCalldata))
            .collect();
        let calldata = BoundedVec::<Felt252Wrapper, MaxCalldataSize>::try_from(calldata?)
            .map_err(|_| DeserializeCallEntrypointError::CalldataExceedsMaxSize)?;

        match d {
            DeserializeInvokeTransaction::V0(d_invoke_tx_v0) => Ok(InvokeTransaction::V0(InvokeTransactionV0 {
                transaction_hash,
                signature,
                nonce,
                max_fee,
                contract_address: sender_address,
                entry_point_selector: string_to_felt(&d_invoke_tx_v0.entry_point_selector)
                    .map_err(DeserializeCallEntrypointError::InvalidEntrypointSelector)?,
                calldata,
            })),
            DeserializeInvokeTransaction::V1(_) => Ok(InvokeTransaction::V1(InvokeTransactionV1 {
                transaction_hash,
                signature,
                nonce,
                max_fee,
                sender_address,
                calldata,
            })),
        }
    }
}

/// Implementation of `TryFrom<DeserializeDeclareTransaction>` for `DeclareTransaction`.
///
/// Converts a `DeserializeDeclareTransaction` into a `DeclareTransaction`, performing necessary
/// validations and transformations on the input data.
impl TryFrom<DeserializeDeclareTransaction> for DeclareTransaction {
    type Error = DeserializeTransactionError;

    /// Converts a `DeserializeTransaction` into a `Transaction`.
    ///
    /// Returns a `DeserializeTransactionError` variant if any field fails validation or conversion.
    fn try_from(d: DeserializeDeclareTransaction) -> Result<Self, Self::Error> {
        // Convert hash to Felt252Wrapper
        let transaction_hash =
            string_to_felt(&d.transaction_hash()).map_err(DeserializeTransactionError::InvalidHash)?;

        // Convert signatures to BoundedVec<Felt252Wrapper, MaxArraySize> and check if it exceeds max size
        let signature = d
            .signature()
            .into_iter()
            .map(|s| string_to_felt(&s).map_err(DeserializeTransactionError::InvalidSignature))
            .collect::<Result<Vec<Felt252Wrapper>, DeserializeTransactionError>>()?;
        let signature = BoundedVec::<Felt252Wrapper, MaxArraySize>::try_from(signature)
            .map_err(|_| DeserializeTransactionError::SignatureExceedsMaxSize)?;

        // Convert sender_address to ContractAddressWrapper
        let sender_address = string_to_felt(remove_prefix(&d.sender_address()))
            .map_err(DeserializeTransactionError::InvalidSenderAddress)?;

        // Convert nonce to U256
        let nonce = Felt252Wrapper::try_from(U256::from(d.nonce())).unwrap();

        // Convert max_fee to U256
        let max_fee = Felt252Wrapper::try_from(U256::from(d.max_fee())).unwrap();

        let contract_class = serde_json::from_str(&d.contract_class())
            .map_err(|e| DeserializeTransactionError::FailedToParse(format!("{:?}", e)))?;
        let class_hash = match Felt252Wrapper::from_hex_be(&d.class_hash().as_str()) {
            Ok(felt) => felt,
            Err(e) => return Err(DeserializeTransactionError::InvalidClassHash(e)),
        };

        match d {
            DeserializeDeclareTransaction::V1(_) => Ok(DeclareTransaction::V1(DeclareTransactionV1 {
                transaction_hash,
                signature,
                nonce,
                max_fee,
                contract_class,
                class_hash,
                sender_address,
            })),
            DeserializeDeclareTransaction::V2(d_declare_tx_v2) => {
                let casm_class_hash = match Felt252Wrapper::from_hex_be(d_declare_tx_v2.compiled_class_hash.as_str()) {
                    Ok(felt) => felt,
                    Err(e) => return Err(DeserializeTransactionError::InvalidCasmClassHash(e)),
                };

                Ok(DeclareTransaction::V2(DeclareTransactionV2 {
                    transaction_hash,
                    signature,
                    nonce,
                    max_fee,
                    contract_class,
                    class_hash,
                    compiled_class_hash: casm_class_hash,
                    sender_address,
                }))
            }
        }
    }
}

/// Implementation of `TryFrom<DeserializeDeployAccountTransaction>` for `DeployAccountTransaction`.
///
/// Converts a `DeserializeDeployAccountTransaction` into a `DeployAccountTransaction`, performing
/// necessary validations and transformations on the input data.
impl TryFrom<DeserializeDeployAccountTransaction> for DeployAccountTransaction {
    type Error = DeserializeTransactionError;

    /// Converts a `DeserializeTransaction` into a `Transaction`.
    ///
    /// Returns a `DeserializeTransactionError` variant if any field fails validation or conversion.
    fn try_from(d: DeserializeDeployAccountTransaction) -> Result<Self, Self::Error> {
        // Convert hash to Felt252Wrapper
        let transaction_hash = string_to_felt(&d.transaction_hash).map_err(DeserializeTransactionError::InvalidHash)?;

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

        // Convert max_fee to U256
        let max_fee = Felt252Wrapper::try_from(U256::from(d.max_fee)).unwrap();

        let class_hash = match Felt252Wrapper::from_hex_be(&d.class_hash.as_str()) {
            Ok(felt) => felt,
            Err(e) => return Err(DeserializeTransactionError::InvalidClassHash(e)),
        };

        let constructor_calldata: Result<Vec<Felt252Wrapper>, DeserializeCallEntrypointError> = d
            .constructor_calldata
            .into_iter()
            .map(|hex_str| string_to_felt(&hex_str).map_err(DeserializeCallEntrypointError::InvalidCalldata))
            .collect();
        let constructor_calldata = BoundedVec::<Felt252Wrapper, MaxCalldataSize>::try_from(constructor_calldata?)
            .map_err(|_| DeserializeCallEntrypointError::CalldataExceedsMaxSize)?;

        let contract_address_salt: Felt252Wrapper = serde_json::from_str(&d.contract_address_salt)
            .map_err(|e| DeserializeTransactionError::FailedToParse(format!("{:?}", e)))?;

        Ok(DeployAccountTransaction {
            transaction_hash,
            max_fee,
            version: 1_u8,
            signature,
            nonce,
            contract_address_salt,
            constructor_calldata,
            class_hash,
            sender_address,
        })
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
        match d {
            DeserializeTransaction::Invoke(d_invoke_tx) => {
                let invoke_transaction = InvokeTransaction::try_from(d_invoke_tx)?;
                Ok(Transaction::Invoke(invoke_transaction))
            }
            DeserializeTransaction::Declare(d_declare_tx) => {
                let declare_transaction = DeclareTransaction::try_from(d_declare_tx)?;
                Ok(Transaction::Declare(declare_transaction))
            }
            DeserializeTransaction::DeployAccount(d_deploy_account_tx) => {
                let deploy_account = DeployAccountTransaction::try_from(d_deploy_account_tx)?;
                Ok(Transaction::DeployAccount(deploy_account))
            }
        }
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
pub fn transaction_from_json(json_str: &str) -> Result<Transaction, DeserializeTransactionError> {
    // Deserialize the JSON string into a DeserializeTransaction and convert it into a Transaction
    let deserialized_transaction: DeserializeTransaction =
        serde_json::from_str(json_str).map_err(|e| DeserializeTransactionError::FailedToParse(format!("{:?}", e)))?;
    let mut transaction = Transaction::try_from(deserialized_transaction)?;

    Ok(transaction)
}
