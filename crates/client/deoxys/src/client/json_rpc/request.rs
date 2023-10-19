//! Defines the generic JSON-RPC [`Request`].

use starknet_core::types::requests::GetBlockWithTxHashesRequest;
use starknet_core::types::MaybePendingBlockWithTxHashes;

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
    };
}

impl_Request!("starknet_getBlockWithTxHashes", GetBlockWithTxHashesRequest, MaybePendingBlockWithTxHashes);