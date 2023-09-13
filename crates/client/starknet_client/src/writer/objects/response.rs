//! This module contains the response returned by the [`Starknet`] gateway on the successful flow.
//!
//! [`Starknet`]: https://starknet.io/

#[cfg(test)]
#[path = "response_test.rs"]
mod response_test;

use serde::{Deserialize, Serialize};
use starknet_api::core::{ClassHash, ContractAddress};
use starknet_api::transaction::TransactionHash;

/// A Starknet error code that reports success.
#[derive(Debug, Default, Deserialize, Serialize, Clone, Eq, PartialEq)]
pub enum SuccessfulStarknetErrorCode {
    #[serde(rename = "TRANSACTION_RECEIVED")]
    #[default]
    TransactionReceived,
}

/// The response of adding a declare transaction through the Starknet gateway successfully.
#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DeclareResponse {
    pub code: SuccessfulStarknetErrorCode,
    pub transaction_hash: TransactionHash,
    pub class_hash: ClassHash,
}

/// The response of adding a deploy account transaction through the Starknet gateway successfully.
#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DeployAccountResponse {
    pub code: SuccessfulStarknetErrorCode,
    pub transaction_hash: TransactionHash,
    pub address: ContractAddress,
}

/// The response of adding an invoke transaction through the Starknet gateway successfully.
#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct InvokeResponse {
    pub code: SuccessfulStarknetErrorCode,
    pub transaction_hash: TransactionHash,
}
