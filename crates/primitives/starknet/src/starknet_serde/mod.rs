//! This module contains the serialization and deserialization functions for the StarkNet types.
use core::str::FromStr;

use blockifier::test_utils::get_contract_class;
use frame_support::BoundedVec;
use hex::FromHex;
use sp_core::{H256, U256};

use crate::execution::{CallEntryPointWrapper, ContractAddressWrapper, ContractClassWrapper, EntryPointTypeWrapper};
use crate::transaction::types::{EventWrapper, MaxArraySize, Transaction};

// Deserialization and Conversion for JSON Transactions, Events, and CallEntryPoints
/// Struct for deserializing CallEntryPoint from JSON
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
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

/// Struct for deserializing Event from JSON
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct DeserializeEventWrapper {
    /// The keys (topics) of the event.
    pub keys: Vec<String>,
    /// The data of the event.
    pub data: Vec<String>,
    /// The address that emited the event
    pub from_address: String,
}

/// Struct for deserializing Transaction from JSON
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
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

// Implement TryFrom trait to convert DeserializeTransaction to Transaction
impl TryFrom<DeserializeTransaction> for Transaction {
    type Error = String;

    fn try_from(d: DeserializeTransaction) -> Result<Self, Self::Error> {
        let version = U256::from(d.version);
        let hash = H256::from_str(&d.hash.as_str()).map_err(|_| "Invalid hash")?;
        let signature = d
            .signature
            .into_iter()
            .map(|s| H256::from_str(&s).map_err(|_| "Invalid signature"))
            .collect::<Result<Vec<H256>, &str>>()?;
        let signature =
            BoundedVec::<H256, MaxArraySize>::try_from(signature).map_err(|_| "Signature exceeds maximum size")?;
        let events = d
            .events
            .into_iter()
            .map(EventWrapper::try_from)
            .collect::<Result<Vec<EventWrapper>, String>>()
            .map_err(|_| "Invalid events")?;
        let events =
            BoundedVec::<EventWrapper, MaxArraySize>::try_from(events).map_err(|_| "Events exceed maximum size")?;
        let sender_address =
            ContractAddressWrapper::from_hex(&d.sender_address).map_err(|_| "Invalid sender address")?;
        let nonce = U256::from(d.nonce);
        let call_entrypoint = CallEntryPointWrapper::try_from(d.call_entrypoint)?;

        Ok(Self { version, hash, signature, events, sender_address, nonce, call_entrypoint, ..Transaction::default() })
    }
}

/// Implement TryFrom trait to convert DeserializeCallEntrypoint to CallEntryPointWrapper
impl TryFrom<DeserializeCallEntrypoint> for CallEntryPointWrapper {
    type Error = String;

    fn try_from(d: DeserializeCallEntrypoint) -> Result<Self, Self::Error> {
        let class_hash = match d.class_hash {
            Some(hash) => Some(<[u8; 32]>::from_hex(&hash).map_err(|_| "Invalid class_hash")?),
            None => None,
        };

        let entrypoint_type = match d.entrypoint_type.as_str() {
            "Constructor" => EntryPointTypeWrapper::Constructor,
            "External" => EntryPointTypeWrapper::External,
            "L1Handler" => EntryPointTypeWrapper::L1Handler,
            _ => return Err("Invalid entrypoint_type".to_string()),
        };

        let entrypoint_selector = match d.entrypoint_selector {
            Some(selector) => Some(H256::from_str(&selector).map_err(|_| "Invalid entrypoint_selector")?),
            None => None,
        };

        let calldata: Result<Vec<H256>, &str> =
            d.calldata.into_iter().map(|hex_str| H256::from_str(&hex_str).map_err(|_| "Invalid calldata")).collect();
        let calldata = BoundedVec::<H256, MaxArraySize>::try_from(calldata?).map_err(|_| "Exceeded max array size")?;

        let storage_address = <[u8; 32]>::from_hex(&d.storage_address).map_err(|_| "Invalid storage_address")?;

        let caller_address = <[u8; 32]>::from_hex(&d.caller_address).map_err(|_| "Invalid caller_address")?;

        Ok(Self { class_hash, entrypoint_type, entrypoint_selector, calldata, storage_address, caller_address })
    }
}

// Implement TryFrom trait to convert DeserializeEventWrapper to EventWrapper
impl TryFrom<DeserializeEventWrapper> for EventWrapper {
    type Error = String;

    fn try_from(d: DeserializeEventWrapper) -> Result<Self, Self::Error> {
        let keys: Result<Vec<H256>, &str> =
            d.keys.into_iter().map(|s| H256::from_str(&s).map_err(|_| "Invalid key")).collect();
        let keys = BoundedVec::<H256, MaxArraySize>::try_from(keys?).map_err(|_| "Exceeded max array size")?;

        let data: Result<Vec<H256>, &str> =
            d.data.into_iter().map(|s| H256::from_str(s.as_str()).map_err(|_| "Invalid data")).collect();
        let data = BoundedVec::<H256, MaxArraySize>::try_from(data?).map_err(|_| "Exceeded max array size")?;

        let from_address =
            H256::from_str(&d.from_address.as_str()).map_err(|_| "Invalid caller_address")?.to_fixed_bytes();

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
pub fn transaction_from_json(json_str: &str, contract_content: &'static [u8]) -> Result<Transaction, String> {
    let deserialized_transaction: DeserializeTransaction = serde_json::from_str(json_str).map_err(|e| {
        let error_message = format!("Failed to convert deserialized transaction: {:?}", e);
        println!("{}", error_message);
        error_message
    })?;
    let mut transaction = Transaction::try_from(deserialized_transaction).map_err(|e| {
        let error_message = format!("Failed to convert deserialized transaction: {:?}", e);
        println!("{}", error_message);
        error_message
    })?;

    if !contract_content.is_empty() {
        transaction.contract_class = Some(ContractClassWrapper::from(get_contract_class(contract_content)));
    } else {
        transaction.contract_class = None;
    }

    Ok(transaction)
}
