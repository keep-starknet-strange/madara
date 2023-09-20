use alloc::string::String;
use alloc::vec::Vec;
use core::str::Utf8Error;

use parity_scale_codec::{Decode, Encode};
use serde::Deserialize;
use sp_runtime::offchain::http::Error;
use sp_runtime::offchain::HttpError;
use sp_runtime::DispatchError;

use crate::message::Message;

/// Error enum wrapper for offchain worker tasks.
#[derive(Debug, Eq, PartialEq)]
pub enum OffchainWorkerError {
    HttpError(HttpError),
    RequestError(Error),
    SerdeError,
    ToBytesError(Utf8Error),
    ConsumeMessageError(DispatchError),
    ToTransactionError,
    U256ConversionError,
    HexDecodeError,
    EmptyData,
    NoLastKnownEthBlock,
    GetStorageFailed,
    EthRpcNotSet,
    FormatBytesFailed,
}

/// Struct that represents the response fields that we need of the eth node for
/// `eth_getBlockByNumber`.
#[derive(Deserialize, Encode, Decode, Default, Debug)]
pub struct EthGetBlockByNumberResponse {
    /// Object that contains the block number.
    pub result: NumberRes,
}

impl TryFrom<EthGetBlockByNumberResponse> for u64 {
    type Error = OffchainWorkerError;

    fn try_from(value: EthGetBlockByNumberResponse) -> Result<Self, Self::Error> {
        u64::from_str_radix(&value.result.number[2..], 16).map_err(|_| OffchainWorkerError::HexDecodeError)
    }
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
