//! Starknet pallet custom types.
use frame_support::codec::{Decode, Encode, MaxEncodedLen};
use frame_support::scale_info::TypeInfo;
use kp_starknet::execution::ContractAddressWrapper;
use serde::Deserialize;
use sp_core::{H256, U256};
use sp_runtime::offchain::http::Error;
use sp_runtime::offchain::HttpError;
use sp_runtime::{DispatchError, RuntimeDebug};

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use core::str::Utf8Error;
/// TODO: Replace with a proper type for field element.

/// Nonce of a Starknet transaction.
pub type NonceWrapper = U256;
/// Storage Key
pub type StorageKey = H256;
/// Contract Storage Key
pub type ContractStorageKeyWrapper = (ContractAddressWrapper, StorageKey);
/// Felt
pub type StarkFeltWrapper = U256;

/// Representation of the origin of a Starknet transaction.
/// For now, we still don't know how to represent the origin of a Starknet transaction,
/// given that Starknet has native account abstraction.
/// For now, we just use a dummy origin.
/// See: `https://github.com/keep-starknet-strange/kaioshin/issues/21`
#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum RawOrigin {
    StarknetTransaction,
}

/// Error enum wrapper for offchain worker tasks.
#[derive(Debug)]
pub enum OffchainWorkerError {
    HttpError(HttpError),
    RequestError(Error),
    SerdeError,
    ToBytesError(Utf8Error),
    ConsumeMessageError(DispatchError),
    ToTransactionError,
    U256ConversionError,
    HexDecodeError,
}

/// Struct that represents the response fields that we need of the eth node for
/// `eth_getBlockByNumber`.
#[derive(Deserialize, Encode, Decode, Default, Debug)]
pub struct EthBlockNumber {
    /// Object that contains the block number.
    pub result: NumberRes,
}

/// Inner struct for block number.
#[derive(Deserialize, Encode, Decode, Default, Debug)]
pub struct NumberRes {
    /// Block number.
    pub number: String,
}

/// Struct that represents an Ethereum event for a message sent to starknet.
#[derive(Deserialize, Encode, Decode, Default, Debug)]
pub struct EthLogs {
    /// Array that contains the events.
    pub result: Vec<Message>,
}

/// Inner struct for messages.
#[derive(Deserialize, Encode, Decode, Default, Debug)]
pub struct Message {
    /// Topics of the event.
    pub topics: Vec<String>,
    /// Data of the event.
    pub data: String,
}
