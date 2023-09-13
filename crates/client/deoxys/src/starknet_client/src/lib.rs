// config compiler to support no_coverage feature when running coverage in nightly mode within this
// crate
#![cfg_attr(coverage_nightly, feature(no_coverage))]

//! This crate contains clients that can communicate with [`Starknet`] through the various
//! endpoints [`Starknet`] has.
//!
//!
//! [`Starknet`]: https://starknet.io/

pub mod reader;
pub mod retry;
#[cfg(test)]
mod starknet_client_test;
pub mod starknet_error;
#[cfg(test)]
mod test_utils;
pub mod writer;

use std::collections::HashMap;

use reqwest::header::HeaderMap;
use reqwest::{Client, RequestBuilder, StatusCode};
use tracing::warn;

use self::retry::Retry;
pub use self::retry::RetryConfig;
pub use self::starknet_error::{KnownStarknetErrorCode, StarknetError, StarknetErrorCode};

/// A [`Result`] in which the error is a [`ClientError`].
type ClientResult<T> = Result<T, ClientError>;

/// A starknet client.
struct StarknetClient {
    http_headers: HeaderMap,
    pub internal_client: Client,
    retry_config: RetryConfig,
}

/// Errors that might be encountered while creating the client.
#[derive(thiserror::Error, Debug)]
pub enum ClientCreationError {
    #[error(transparent)]
    BadUrl(#[from] url::ParseError),
    #[error(transparent)]
    BuildError(#[from] reqwest::Error),
    #[error(transparent)]
    HttpHeaderError(#[from] http::Error),
}

/// Errors that might be solved by retrying mechanism.
#[derive(Debug, Eq, PartialEq)]
pub enum RetryErrorCode {
    Redirect,
    Timeout,
    TooManyRequests,
    ServiceUnavailable,
    Disconnect,
}

/// Errors that may be returned by a reader or writer client.
#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    /// A client error representing bad status http responses.
    #[error("Bad response status code: {:?} message: {:?}.", code, message)]
    BadResponseStatus { code: StatusCode, message: String },
    /// A client error representing http request errors.
    #[error(transparent)]
    RequestError(#[from] reqwest::Error),
    /// A client error representing errors that might be solved by retrying mechanism.
    #[error("Retry error code: {:?}, message: {:?}.", code, message)]
    RetryError { code: RetryErrorCode, message: String },
    /// A client error representing deserialization errors.
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
    /// A client error representing errors returned by the starknet client.
    #[error(transparent)]
    StarknetError(#[from] StarknetError),
}

// A wrapper error for request_with_retry to handle the case that clone failed.
#[derive(thiserror::Error, Debug)]
enum RequestWithRetryError {
    #[error("Request is unclonable.")]
    CloneError,
    #[error(transparent)]
    ClientError(#[from] ClientError),
}

impl StarknetClient {
    /// Creates a new client for a starknet gateway at `url_str` with retry_config [`RetryConfig`].
    pub fn new(
        http_headers: Option<HashMap<String, String>>,
        node_version: &'static str,
        retry_config: RetryConfig,
    ) -> Result<Self, ClientCreationError> {
        let header_map = match http_headers {
            Some(inner) => (&inner).try_into()?,
            None => HeaderMap::new(),
        };
        let info = os_info::get();
        let system_information =
            format!("{}; {}; {}", info.os_type(), info.version(), info.bitness());
        let app_user_agent = format!(
            "{product_name}/{product_version} ({system_information})",
            product_name = "papyrus",
            product_version = node_version,
            system_information = system_information
        );
        Ok(StarknetClient {
            http_headers: header_map,
            internal_client: Client::builder().user_agent(app_user_agent).build()?,
            retry_config,
        })
    }

    fn get_retry_error_code(err: &ClientError) -> Option<RetryErrorCode> {
        match err {
            ClientError::BadResponseStatus { code, message: _ } => match *code {
                StatusCode::TEMPORARY_REDIRECT => Some(RetryErrorCode::Redirect),
                StatusCode::REQUEST_TIMEOUT | StatusCode::GATEWAY_TIMEOUT => {
                    Some(RetryErrorCode::Timeout)
                }
                StatusCode::TOO_MANY_REQUESTS => Some(RetryErrorCode::TooManyRequests),
                StatusCode::SERVICE_UNAVAILABLE => Some(RetryErrorCode::ServiceUnavailable),
                _ => None,
            },

            ClientError::RequestError(internal_err) => {
                if internal_err.is_timeout() {
                    Some(RetryErrorCode::Timeout)
                } else if internal_err.is_request() {
                    None
                } else if internal_err.is_connect() {
                    Some(RetryErrorCode::Disconnect)
                } else if internal_err.is_redirect() {
                    Some(RetryErrorCode::Redirect)
                } else {
                    None
                }
            }

            ClientError::StarknetError(StarknetError {
                code:
                    StarknetErrorCode::KnownErrorCode(KnownStarknetErrorCode::TransactionLimitExceeded),
                message: _,
            }) => Some(RetryErrorCode::TooManyRequests),
            _ => None,
        }
    }

    fn should_retry(err: &RequestWithRetryError) -> bool {
        match err {
            RequestWithRetryError::ClientError(err) => Self::get_retry_error_code(err).is_some(),
            RequestWithRetryError::CloneError => false,
        }
    }

    // If the request_builder is unclonable, the function will not retry the request upon failure.
    pub async fn request_with_retry(
        &self,
        request_builder: RequestBuilder,
    ) -> ClientResult<String> {
        let res = Retry::new(&self.retry_config)
            .start_with_condition(
                || async {
                    match request_builder.try_clone() {
                        Some(request_builder) => self
                            .request(request_builder)
                            .await
                            .map_err(RequestWithRetryError::ClientError),
                        None => Err(RequestWithRetryError::CloneError),
                    }
                },
                Self::should_retry,
            )
            .await;

        match res {
            Ok(string) => Ok(string),
            Err(RequestWithRetryError::ClientError(err)) => Err(Self::get_retry_error_code(&err)
                .map(|code| ClientError::RetryError { code, message: err.to_string() })
                .unwrap_or(err)),
            Err(RequestWithRetryError::CloneError) => {
                warn!("Starknet client got an unclonable request. Can't retry upon failure.");
                self.request(request_builder).await
            }
        }
    }

    async fn request(&self, request_builder: RequestBuilder) -> ClientResult<String> {
        let res = request_builder.headers(self.http_headers.clone()).send().await;
        let (code, message) = match res {
            Ok(response) => (response.status(), response.text().await?),
            Err(err) => {
                let msg = err.to_string();
                (err.status().ok_or(err)?, msg)
            }
        };
        match code {
            StatusCode::OK => Ok(message),
            // TODO(Omri): The error code returned from SN changed from error 500 to error 400. For
            // now, keeping both options. In the future, remove the '500' (INTERNAL_SERVER_ERROR)
            // option.
            StatusCode::INTERNAL_SERVER_ERROR | StatusCode::BAD_REQUEST => {
                let starknet_error: StarknetError = serde_json::from_str(&message)?;
                Err(ClientError::StarknetError(starknet_error))
            }
            _ => Err(ClientError::BadResponseStatus { code, message }),
        }
    }
}
