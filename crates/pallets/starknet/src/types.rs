//! Starknet pallet custom types.
use frame_support::codec::{Decode, Encode, MaxEncodedLen};
use frame_support::scale_info::TypeInfo;
use serde::Deserialize;
use sp_core::{H256, U256};
use sp_runtime::offchain::http::Error;
use sp_runtime::offchain::HttpError;
use sp_runtime::RuntimeDebug;

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use core::str::Utf8Error;
/// TODO: Replace with a proper type for field element.
/// The address of a Starknet contract.
pub type ContractAddress = [u8; 32];
/// The hash of a Starknet contract class.
pub type ContractClassHash = [u8; 32];
/// Nonce of a Starknet transaction.
pub type Nonce = U256;
/// Storage Key
pub type StorageKey = H256;
/// Contract Storage Key
pub type ContractStorageKey = (ContractAddress, StorageKey);
/// Felt
pub type StarkFelt = U256;

/// Representation of the origin of a Starknet transaction.
/// For now, we still don't know how to represent the origin of a Starknet transaction,
/// given that Starknet has native account abstraction.
/// For now, we just use a dummy origin.
/// See: `https://github.com/keep-starknet-strange/kaioshin/issues/21`
#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum RawOrigin {
    StarknetTransaction,
}

#[derive(Debug)]
pub enum OffchainWorkerError {
    HttpError(HttpError),
    RequestError(Error),
    SerdeError,
    ToBytesError(Utf8Error),
}

#[derive(Deserialize, Encode, Decode, Default, Debug)]
pub struct EthBlockNumber {
    pub result: NumberRes,
}
#[derive(Deserialize, Encode, Decode, Default, Debug)]
pub struct NumberRes {
    pub number: String,
}

#[derive(Deserialize, Encode, Decode, Default, Debug)]
pub struct EthLogs {
    pub result: Vec<Message>,
}
#[derive(Deserialize, Encode, Decode, Default, Debug)]
pub struct Message {
    pub topics: Vec<String>,
    pub data: String,
}
