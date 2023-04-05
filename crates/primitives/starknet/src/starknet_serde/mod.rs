//! This module contains the serialization and deserialization functions for the StarkNet types.
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use blockifier::test_utils::get_contract_class;
use frame_support::BoundedVec;
use hex::FromHex;
use serde::{Deserialize, Serialize};
use sp_core::{H256, U256};

use crate::alloc::string::ToString;
use crate::execution::{CallEntryPointWrapper, ContractAddressWrapper, ContractClassWrapper, EntryPointTypeWrapper};
use crate::transaction::types::{EventWrapper, MaxArraySize, Transaction};

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

fn remove_prefix(input: &str) -> &str {
    if input.starts_with("0x") { &input[2..] } else { input }
}

fn string_to_h256(hex_str: &str) -> Result<H256, String> {
    let bytes =
        Vec::from_hex(remove_prefix(hex_str)).map_err(|e| format!("Failed to convert hex string to bytes: {:?}", e))?;
    if bytes.len() == 32 { Ok(H256::from_slice(&bytes)) } else { Err(format!("Invalid input length: {}", bytes.len())) }
}

// Implement TryFrom trait to convert DeserializeTransaction to Transaction
impl TryFrom<DeserializeTransaction> for Transaction {
    type Error = String;

    fn try_from(d: DeserializeTransaction) -> Result<Self, Self::Error> {
        let version = U256::from(d.version);
        let hash =
            string_to_h256(&d.hash).map_err(|e| format!("Failed to convert hash hex string to H256: {:?}", e))?;
        let signature = d
            .signature
            .into_iter()
            .map(|s| string_to_h256(&s).map_err(|e| format!("Invalid signature: {}", e)))
            .collect::<Result<Vec<H256>, String>>()?;
        let signature =
            BoundedVec::<H256, MaxArraySize>::try_from(signature).map_err(|_| "Signature exceeds maximum size")?;
        let events = d
            .events
            .into_iter()
            .map(EventWrapper::try_from)
            .collect::<Result<Vec<EventWrapper>, String>>()
            .map_err(|e| format!("Invalid events: {:?}", e))?;
        let events =
            BoundedVec::<EventWrapper, MaxArraySize>::try_from(events).map_err(|_| "Events exceed maximum size")?;
        let sender_address = ContractAddressWrapper::from_hex(remove_prefix(&d.sender_address))
            .map_err(|e| format!("Invalid sender_address: {:?}", e))?;
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
            Some(hash) => {
                Some(<[u8; 32]>::from_hex(remove_prefix(&hash)).map_err(|e| format!("Invalid class_hash: {:?}", e))?)
            }
            None => None,
        };

        let entrypoint_type = match d.entrypoint_type.as_str() {
            "Constructor" => EntryPointTypeWrapper::Constructor,
            "External" => EntryPointTypeWrapper::External,
            "L1Handler" => EntryPointTypeWrapper::L1Handler,
            _ => return Err("Invalid entrypoint_type".to_string()),
        };

        let entrypoint_selector = match d.entrypoint_selector {
            Some(selector) => Some(
                string_to_h256(&selector)
                    .map_err(|e| format!("Failed to convert entrypoint_selector hex string to H256: {:?}", e))?,
            ),
            None => None,
        };

        let calldata: Result<Vec<H256>, String> =
            d.calldata.into_iter().map(|hex_str| string_to_h256(&hex_str)).collect();
        let calldata = BoundedVec::<H256, MaxArraySize>::try_from(calldata?).map_err(|_| "Exceeded max array size")?;

        let storage_address = <[u8; 32]>::from_hex(remove_prefix(&d.storage_address))
            .map_err(|e| format!("Invalid storage_address: {:?}", e))?;

        let caller_address = <[u8; 32]>::from_hex(remove_prefix(&d.caller_address))
            .map_err(|e| format!("Invalid caller_address: {:?}", e))?;

        Ok(Self { class_hash, entrypoint_type, entrypoint_selector, calldata, storage_address, caller_address })
    }
}

// Implement TryFrom trait to convert DeserializeEventWrapper to EventWrapper
impl TryFrom<DeserializeEventWrapper> for EventWrapper {
    type Error = String;

    fn try_from(d: DeserializeEventWrapper) -> Result<Self, Self::Error> {
        let keys: Result<Vec<H256>, String> = d
            .keys
            .into_iter()
            .map(|hex_str| {
                string_to_h256(&hex_str).map_err(|e| format!("Failed to convert keys hex string to H256: {:?}", e))
            })
            .collect();
        let keys = BoundedVec::<H256, MaxArraySize>::try_from(keys?).map_err(|_| "Exceeded max array size")?;

        let data: Result<Vec<H256>, String> = d
            .data
            .into_iter()
            .map(|hex_str| {
                string_to_h256(&hex_str).map_err(|e| format!("Failed to convert data hex string to H256: {:?}", e))
            })
            .collect();
        let data = BoundedVec::<H256, MaxArraySize>::try_from(data?).map_err(|_| "Exceeded max array size")?;

        let from_address: [u8; 32] = <[u8; 32]>::from_hex(remove_prefix(&d.from_address))
            .map_err(|e| format!("Failed to convert from_address hex string to bytes: {:?}", e))?;

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
    let deserialized_transaction: DeserializeTransaction =
        serde_json::from_str(json_str).map_err(|e| format!("Failed to convert deserialized transaction: {:?}", e))?;
    let mut transaction = Transaction::try_from(deserialized_transaction)
        .map_err(|e| format!("Failed to convert deserialized transaction: {:?}", e))?;

    if !contract_content.is_empty() {
        transaction.contract_class = Some(ContractClassWrapper::from(get_contract_class(contract_content)));
    } else {
        transaction.contract_class = None;
    }

    Ok(transaction)
}
