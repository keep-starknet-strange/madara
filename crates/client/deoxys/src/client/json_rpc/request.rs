//! Defines the generic JSON-RPC [`Request`].

use starknet_core::types::requests::*;
use starknet_core::types::*;

/// Represents a JSON-RPC request.
pub trait Request {
    /// The JSON-RPC method name.
    const METHOD: &'static str;

    /// The response type expected for the request.
    type Response: for<'de> serde::Deserialize<'de>;

    /// A type that, when serialized, represents the parameters of the request.
    /// 
    /// This type must be directly serialized into an array.
    type Params: serde::Serialize;

    /// Converts this [`Request`] into its parameters.
    fn into_params(self) -> Self::Params;
}

// =======================================================
// IMPLEMENTATIONS OF `Request` FOR COMMON STARKNET TYPES
// =======================================================

macro_rules! impl_Request {
    ($method:literal, $req:ty, $res:ty) => {
        impl Request for $req {
            const METHOD: &'static str = $method;
            type Response = $res;
            type Params = Self;

            #[inline(always)]
            fn into_params(self) -> Self::Params {
                self
            }
        }

        impl<'a> Request for &'a $req {
            const METHOD: &'static str = $method;
            type Response = $res;
            type Params = Self;

            #[inline(always)]
            fn into_params(self) -> Self::Params {
                self
            }
        }
    };
}

// https://github.com/starkware-libs/starknet-specs/blob/master/api/starknet_api_openrpc.json

impl_Request!("starknet_getBlockWithTxHashes", GetBlockWithTxHashesRequest, MaybePendingBlockWithTxHashes);
impl_Request!("starknet_getBlockWithTxs", GetBlockWithTxsRequest, MaybePendingBlockWithTxs);
impl_Request!("starknet_getStateUpdate", GetStateUpdateRequest, MaybePendingStateUpdate);
impl_Request!("starknet_getStorageAt", GetStorageAtRequest, FieldElement);
impl_Request!("starknet_getTransactionByHash", GetTransactionByHashRequest, Transaction);
impl_Request!("starknet_getTransactionByBlockIdAndIndex", GetTransactionByBlockIdAndIndexRequest, Transaction);
impl_Request!("starknet_getTransactionReceipt", GetTransactionReceiptRequest, TransactionReceipt);
impl_Request!("starknet_getClass", GetClassRequest, ContractClass);
impl_Request!("starknet_getClassHashAt", GetClassHashAtRequest, FieldElement);
impl_Request!("starknet_getClassAt", GetClassAtRequest, ContractClass);
impl_Request!("starknet_getBlockTransactionCount", GetBlockTransactionCountRequest, u64);
impl_Request!("starknet_call", CallRequest, FieldElement);
impl_Request!("starknet_estimateFee", EstimateFeeRequest, FeeEstimate);
impl_Request!("starknet_estimateMessageFee", EstimateMessageFeeRequest, FeeEstimate);
impl_Request!("starknet_blockNumber", BlockNumberRequest, u64);
impl_Request!("starknet_blockHashAndNumber", BlockHashAndNumberRequest, BlockHashAndNumber);
impl_Request!("starknet_syncing", SyncingRequest, SyncStatusType);
impl_Request!("starknet_getEvents", GetEventsRequest, EventsPage);
impl_Request!("starknet_getNonce", GetNonceRequest, FieldElement);