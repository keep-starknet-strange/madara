//! This module contains all the different transactions that can be added to [`Starknet`] via the
//! gateway.
//!
//! Each transaction can be serialized into a JSON object that the gateway can receive through the
//! `add_transaction` HTTP method.
//!
//! [`Starknet`]: https://starknet.io/

#[cfg(test)]
#[path = "transaction_test.rs"]
mod transaction_test;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use starknet_api::api_core::{ClassHash, Nonce, ContractAddress, CompiledClassHash};
use starknet_api::deprecated_contract_class::{
    ContractClassAbiEntry as DeprecatedContractClassAbiEntry,
    EntryPoint as DeprecatedEntryPoint,
    EntryPointType as DeprecatedEntryPointType,
};
use starknet_api::state::{EntryPoint, EntryPointType};
use starknet_api::transaction::{
    Calldata,
    ContractAddressSalt,
    Fee,
    TransactionSignature,
    TransactionVersion,
};

// Each transaction type has a field called `type`. This field needs to be of a type that
// serializes to/deserializes from a constant string.
//
// The reason we don't solve this by having an enum of a generic transaction and let serde generate
// the `type` field through #[serde(tag)] is because we want to serialize/deserialize from the
// structs of the specific transaction types.

/// The type field of a deploy account transaction. This enum serializes/deserializes into a
/// constant string.
#[derive(Debug, Deserialize, Serialize, Default, Clone, Copy, Eq, PartialEq)]
pub enum DeployAccountType {
    #[serde(rename = "DEPLOY_ACCOUNT")]
    #[default]
    DeployAccount,
}

/// The type field of an invoke transaction. This enum serializes/deserializes into a constant
/// string.
#[derive(Debug, Deserialize, Serialize, Default, Clone, Copy, Eq, PartialEq)]
pub enum InvokeType {
    #[serde(rename = "INVOKE_FUNCTION")]
    #[default]
    Invoke,
}

/// The type field of a declare V1 transaction. This enum serializes/deserializes into a constant
/// string.
#[derive(Debug, Deserialize, Serialize, Default, Clone, Copy, Eq, PartialEq)]
pub enum DeclareV1Type {
    #[serde(rename = "DEPRECATED_DECLARE")]
    #[default]
    DeclareV1,
}

/// The type field of a declare V2 transaction. This enum serializes/deserializes into a constant
/// string.
#[derive(Debug, Deserialize, Serialize, Default, Clone, Copy, Eq, PartialEq)]
pub enum DeclareV2Type {
    #[serde(rename = "DECLARE")]
    #[default]
    DeclareV2,
}

/// A deploy account transaction that can be added to Starknet through the Starknet gateway.
/// It has a serialization format that the Starknet gateway accepts in the `add_transaction`
/// HTTP method.
#[derive(Debug, Default, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DeployAccountTransaction {
    pub contract_address_salt: ContractAddressSalt,
    pub class_hash: ClassHash,
    pub constructor_calldata: Calldata,
    pub nonce: Nonce,
    pub max_fee: Fee,
    pub signature: TransactionSignature,
    pub version: TransactionVersion,
    pub r#type: DeployAccountType,
}

/// An invoke account transaction that can be added to Starknet through the Starknet gateway.
/// The invoke is a V1 transaction.
/// It has a serialization format that the Starknet gateway accepts in the `add_transaction`
/// HTTP method.
#[derive(Debug, Default, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct InvokeTransaction {
    pub calldata: Calldata,
    pub sender_address: ContractAddress,
    pub nonce: Nonce,
    pub max_fee: Fee,
    pub signature: TransactionSignature,
    pub version: TransactionVersion,
    pub r#type: InvokeType,
}

/// A declare transaction of a Cairo-v0 (deprecated) contract class that can be added to Starknet
/// through the Starknet gateway.
/// It has a serialization format that the Starknet gateway accepts in the `add_transaction`
/// HTTP method.
#[derive(Debug, Default, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DeclareV1Transaction {
    pub contract_class: DeprecatedContractClass,
    pub sender_address: ContractAddress,
    pub nonce: Nonce,
    pub max_fee: Fee,
    pub version: TransactionVersion,
    pub signature: TransactionSignature,
    pub r#type: DeclareV1Type,
}

/// A declare transaction of a Cairo-v1 contract class that can be added to Starknet through the
/// Starknet gateway.
/// It has a serialization format that the Starknet gateway accepts in the `add_transaction`
/// HTTP method.
#[derive(Debug, Default, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DeclareV2Transaction {
    pub contract_class: ContractClass,
    pub compiled_class_hash: CompiledClassHash,
    pub sender_address: ContractAddress,
    pub nonce: Nonce,
    pub max_fee: Fee,
    pub version: TransactionVersion,
    pub signature: TransactionSignature,
    pub r#type: DeclareV2Type,
}

/// A declare transaction that can be added to Starknet through the Starknet gateway.
/// It has a serialization format that the Starknet gateway accepts in the `add_transaction`
/// HTTP method.
#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(untagged)]
pub enum DeclareTransaction {
    DeclareV1(DeclareV1Transaction),
    DeclareV2(DeclareV2Transaction),
}

// The structs that are implemented here are the structs that have deviations from starknet_api.

#[derive(Debug, Clone, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct DeprecatedContractClass {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub abi: Option<Vec<DeprecatedContractClassAbiEntry>>,
    #[serde(rename = "program")]
    // TODO(shahak): Create a struct for a compressed base64 value.
    pub compressed_program: String,
    pub entry_points_by_type: HashMap<DeprecatedEntryPointType, Vec<DeprecatedEntryPoint>>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct ContractClass {
    // TODO(shahak): Create a struct for a compressed base64 value.
    #[serde(rename = "sierra_program")]
    pub compressed_sierra_program: String,
    pub contract_class_version: String,
    pub entry_points_by_type: HashMap<EntryPointType, Vec<EntryPoint>>,
    pub abi: String,
}

// The conversion is done here and not in papyrus_rpc because the gateway uses starknet_api for
// DeployAccountTransaction.
impl From<starknet_api::transaction::DeployAccountTransaction> for DeployAccountTransaction {
    fn from(tx: starknet_api::transaction::DeployAccountTransaction) -> Self {
        Self {
            contract_address_salt: tx.contract_address_salt,
            class_hash: tx.class_hash,
            constructor_calldata: tx.constructor_calldata,
            nonce: tx.nonce,
            max_fee: tx.max_fee,
            signature: tx.signature,
            version: tx.version,
            r#type: DeployAccountType::default(),
        }
    }
}
