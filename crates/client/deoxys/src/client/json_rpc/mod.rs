//! Provides a JSON-RPC client.

use std::sync::atomic::AtomicU64;
use std::fmt;

mod request;
mod transport;

pub use self::request::*;
pub use self::transport::*;

/// An error that might be returned by the JSON-RPC protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JsonRpcErrorCode(pub i64);

impl JsonRpcErrorCode {
    /// The error code for an parsing error.
    pub const PARSE_ERROR: Self = Self(-32700);
    /// The error code for an invalid request error.
    pub const INVALID_REQUEST: Self = Self(-32600);
    /// The error code for an invalid method error.
    pub const METHOD_NOT_FOUND: Self = Self(-32601);
    /// The error code for an invalid params error.
    pub const INVALID_PARAMS: Self = Self(-32602);
    /// The error code for an internal error.
    pub const INTERNAL_ERROR: Self = Self(-32603);
}

impl fmt::Display for JsonRpcErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

/// An error that might be returned by the JSON-RPC protocol.
#[derive(Debug, Clone, thiserror::Error)]
#[error("{message} (code {code})")]
pub struct JsonRpcError<Data = ()> {
    /// The code of the error.
    pub code: JsonRpcErrorCode,
    /// The message associated with the error.
    pub message: Box<str>,
    /// An optional data field associated with the error.
    pub data: Data,
}

/// A JSON-RPC client that uses a [`Transport`] to communicate over the network.
pub struct JsonRpcClient<T> {
    /// The transport layer used to communicate over the network.
    transport: T,
    /// The next identifier to use for a request.
    next_id: AtomicU64,
}

/// An error that might occur when interacting with the [`JsonRpcClient`].
#[derive(Debug, thiserror::Error)]
pub enum JsonRpcClientError<T, E = ()> {
    /// The transport layer returned an error.
    #[error("{0}")]
    Transport(T),
    /// The JSON-RPC protocol returned an error.
    #[error("{0}")]
    JsonRpc(JsonRpcError<E>),
    /// The received response was not a valid JSON-RPC response.
    #[error("invalid JSON-RPC response")]
    Protocol,
}

impl<T> JsonRpcClient<T> {
    /// Creates a new [`JsonRpcClient`] with the given transport layer.
    pub fn new(transport: T) -> Self {
        JsonRpcClient { transport, next_id: AtomicU64::new(0) }
    }
}

impl<T: Transport> JsonRpcClient<T> {
    /// Sends a JSON-RPC request and returns the response.
    pub async fn request<R: Request>(&self, request: R) -> Result<R::Response, JsonRpcClientError<T::Error>> {
        let id = self.next_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let body = create_json_rpc_request(R::METHOD, id, request.into_params());
        let response = self.transport.request(&body).await.map_err(JsonRpcClientError::Transport)?;
        parse_json_rpc_repsonse(&response, id)
    }
}

/// Creates a JSON-RPC request from the given parameters.
fn create_json_rpc_request<P: serde::Serialize>(
    method: &'static str,
    id: u64,
    params: P,
) -> Vec<u8> {
    #[derive(serde::Serialize)]
    struct RequestType<P> {
        pub jsonrpc: &'static str,
        pub method: &'static str,
        pub params: P,
        #[serde(serialize_with = "serialize_number_as_string")]
        pub id: u64,
    }

    let req = RequestType { jsonrpc: "2.0", method, params, id };

    // If this panics because the serialized failed, it's a bug in the user code
    // that comes from before this function is called.
    serde_json::to_vec(&req).unwrap()
}

/// Parses a JSON-RPC response.
fn parse_json_rpc_repsonse<T, B, E>(data: &[u8], expected_id: u64) -> Result<B, JsonRpcClientError<T, E>>
where
    B: for<'a> serde::Deserialize<'a>,
    E: for<'a> serde::Deserialize<'a>,
{
    #[derive(serde::Deserialize)]
    struct ErrorType<E> {
        #[serde(deserialize_with = "i64_or_string")]
        pub code: i64,
        pub message: Box<str>,
        pub data: E,
    }

    #[derive(serde::Deserialize)]
    struct ResponseType<T, E> {
        pub jsonrpc: Box<str>,
        pub result: Option<T>,
        pub error: Option<ErrorType<E>>,
        #[serde(deserialize_with = "u64_or_string")]
        pub id: u64,
    }

    let response: ResponseType<B, E> = serde_json::from_slice(data).map_err(|_| JsonRpcClientError::Protocol)?;

    if response.id != expected_id || &*response.jsonrpc != "2.0" {
        return Err(JsonRpcClientError::Protocol);
    }

    match (response.result, response.error) {
        (Some(result), None) => Ok(result),
        (None, Some(error)) => Err(JsonRpcClientError::JsonRpc(JsonRpcError {
            code: JsonRpcErrorCode(error.code),
            message: error.message,
            data: error.data,
        })),
        _ => Err(JsonRpcClientError::Protocol),
    }
}

/// A deserializer function that accepts either a number or a string that represents a number.
fn i64_or_string<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<i64, D::Error> {
    struct I64OrString;

    impl<'de> serde::de::Visitor<'de> for I64OrString {
        type Value = i64;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a number or a string that represents a number")
        }

        fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<Self::Value, E> {
            Ok(value)
        }

        fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
            value.parse().map_err(E::custom)
        }
    }

    deserializer.deserialize_any(I64OrString)
}


/// A deserializer function that accepts either a number or a string that represents a number.
fn u64_or_string<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
    struct I64OrString;

    impl<'de> serde::de::Visitor<'de> for I64OrString {
        type Value = u64;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a number or a string that represents a number")
        }

        fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<Self::Value, E> {
            Ok(value)
        }

        fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
            value.parse().map_err(E::custom)
        }
    }

    deserializer.deserialize_any(I64OrString)
}

/// Serializes number as strings of digits.
fn serialize_number_as_string<S: serde::Serializer>(number: &u64, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&number.to_string())
}