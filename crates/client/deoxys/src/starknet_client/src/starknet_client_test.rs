use assert_matches::assert_matches;
use mockito::mock;
use reqwest::StatusCode;

use crate::starknet_error::{KnownStarknetErrorCode, StarknetError, StarknetErrorCode};
use crate::test_utils::retry::{get_test_config, MAX_RETRIES};
use crate::{ClientError, RetryErrorCode, StarknetClient};

const NODE_VERSION: &str = "NODE VERSION";
const URL_SUFFIX: &str = "/query";

#[tokio::test]
async fn request_with_retry_positive_flow() {
    const BODY: &str = "body";
    let starknet_client = StarknetClient::new(None, NODE_VERSION, get_test_config()).unwrap();
    let mock = mock("GET", URL_SUFFIX).with_status(200).with_body(BODY).create();
    let mut url = mockito::server_url();
    url.push_str(URL_SUFFIX);
    let result =
        starknet_client.request_with_retry(starknet_client.internal_client.get(&url)).await;
    assert_eq!(result.unwrap(), BODY);
    mock.assert();
}

#[tokio::test]
async fn request_with_retry_bad_response_status() {
    let error_code = StatusCode::NOT_FOUND;
    let starknet_client = StarknetClient::new(None, NODE_VERSION, get_test_config()).unwrap();
    let mock = mock("GET", URL_SUFFIX).with_status(error_code.as_u16().into()).create();
    let mut url = mockito::server_url();
    url.push_str(URL_SUFFIX);
    let result =
        starknet_client.request_with_retry(starknet_client.internal_client.get(&url)).await;
    assert_matches!(
        result,
        Err(ClientError::BadResponseStatus { code, message: _ }) if code == error_code
    );
    mock.assert();
}

#[tokio::test]
async fn request_with_retry_starknet_error_no_retry() {
    let starknet_client = StarknetClient::new(None, NODE_VERSION, get_test_config()).unwrap();
    let expected_starknet_error = StarknetError {
        code: StarknetErrorCode::KnownErrorCode(KnownStarknetErrorCode::UndeclaredClass),
        message: "message".to_string(),
    };
    let mock = mock("GET", URL_SUFFIX)
        .with_status(StatusCode::BAD_REQUEST.as_u16().into())
        .with_body(serde_json::to_string(&expected_starknet_error).unwrap())
        .create();
    let mut url = mockito::server_url();
    url.push_str(URL_SUFFIX);
    let result =
        starknet_client.request_with_retry(starknet_client.internal_client.get(&url)).await;
    let Err(ClientError::StarknetError(starknet_error)) = result else {
        panic!("Did not get a StarknetError.");
    };
    assert_eq!(starknet_error, expected_starknet_error);
    mock.assert();
}

#[tokio::test]
async fn request_with_retry_serde_error_in_starknet_error() {
    let starknet_client = StarknetClient::new(None, NODE_VERSION, get_test_config()).unwrap();
    let mock = mock("GET", URL_SUFFIX)
        .with_status(StatusCode::BAD_REQUEST.as_u16().into())
        .with_body("body")
        .create();
    let mut url = mockito::server_url();
    url.push_str(URL_SUFFIX);
    let result =
        starknet_client.request_with_retry(starknet_client.internal_client.get(&url)).await;
    assert_matches!(result, Err(ClientError::SerdeError(_)));
    mock.assert();
}

#[tokio::test]
async fn request_with_retry_max_retries_reached() {
    let starknet_client = StarknetClient::new(None, NODE_VERSION, get_test_config()).unwrap();
    for (status_code, error_code) in [
        (StatusCode::TEMPORARY_REDIRECT, RetryErrorCode::Redirect),
        (StatusCode::REQUEST_TIMEOUT, RetryErrorCode::Timeout),
        (StatusCode::TOO_MANY_REQUESTS, RetryErrorCode::TooManyRequests),
        (StatusCode::SERVICE_UNAVAILABLE, RetryErrorCode::ServiceUnavailable),
        (StatusCode::GATEWAY_TIMEOUT, RetryErrorCode::Timeout),
    ] {
        let mock = mock("GET", URL_SUFFIX)
            .with_status(status_code.as_u16().into())
            .expect(MAX_RETRIES + 1)
            .create();
        let mut url = mockito::server_url();
        url.push_str(URL_SUFFIX);
        let result =
            starknet_client.request_with_retry(starknet_client.internal_client.get(&url)).await;
        assert_matches!(
            result, Err(ClientError::RetryError { code, message: _ }) if code == error_code
        );
        mock.assert();
    }
}

#[tokio::test]
async fn request_with_retry_success_on_retry() {
    const BODY: &str = "body";
    assert_ne!(0, MAX_RETRIES);
    let starknet_client = StarknetClient::new(None, NODE_VERSION, get_test_config()).unwrap();
    for status_code in [
        StatusCode::TEMPORARY_REDIRECT,
        StatusCode::REQUEST_TIMEOUT,
        StatusCode::TOO_MANY_REQUESTS,
        StatusCode::SERVICE_UNAVAILABLE,
        StatusCode::GATEWAY_TIMEOUT,
    ] {
        let mock_failure = mock("GET", URL_SUFFIX)
            .with_status(status_code.as_u16().into())
            .expect(MAX_RETRIES)
            .create();
        let mock_success = mock("GET", URL_SUFFIX).with_status(200).with_body(BODY).create();
        let mut url = mockito::server_url();
        url.push_str(URL_SUFFIX);
        let result =
            starknet_client.request_with_retry(starknet_client.internal_client.get(&url)).await;
        assert_eq!(result.unwrap(), BODY);
        mock_failure.assert();
        mock_success.assert();
    }
}

#[tokio::test]
async fn request_with_retry_starknet_error_max_retries_reached() {
    let starknet_client = StarknetClient::new(None, NODE_VERSION, get_test_config()).unwrap();
    let starknet_error = StarknetError {
        code: StarknetErrorCode::KnownErrorCode(KnownStarknetErrorCode::TransactionLimitExceeded),
        message: "message".to_string(),
    };
    let starknet_error_str = serde_json::to_string(&starknet_error).unwrap();
    let mock = mock("GET", URL_SUFFIX)
        .with_status(StatusCode::BAD_REQUEST.as_u16().into())
        .with_body(starknet_error_str)
        .expect(MAX_RETRIES + 1)
        .create();
    let mut url = mockito::server_url();
    url.push_str(URL_SUFFIX);
    let result =
        starknet_client.request_with_retry(starknet_client.internal_client.get(&url)).await;
    assert_matches!(
        result,
        Err(ClientError::RetryError { code, message: _ }) if code == RetryErrorCode::TooManyRequests
    );
    mock.assert();
}

#[tokio::test]
async fn request_with_retry_starknet_error_success_on_retry() {
    const BODY: &str = "body";
    assert_ne!(0, MAX_RETRIES);
    let starknet_client = StarknetClient::new(None, NODE_VERSION, get_test_config()).unwrap();
    let starknet_error = StarknetError {
        code: StarknetErrorCode::KnownErrorCode(KnownStarknetErrorCode::TransactionLimitExceeded),
        message: "message".to_string(),
    };
    let starknet_error_str = serde_json::to_string(&starknet_error).unwrap();
    let mock_failure = mock("GET", URL_SUFFIX)
        .with_status(StatusCode::BAD_REQUEST.as_u16().into())
        .with_body(starknet_error_str)
        .expect(MAX_RETRIES)
        .create();
    let mock_success = mock("GET", URL_SUFFIX).with_status(200).with_body(BODY).create();
    let mut url = mockito::server_url();
    url.push_str(URL_SUFFIX);
    let result =
        starknet_client.request_with_retry(starknet_client.internal_client.get(&url)).await;
    assert_eq!(result.unwrap(), BODY);
    mock_failure.assert();
    mock_success.assert();
}

#[test]
fn serialization_precision() {
    let input =
        "{\"value\":244116128358498188146337218061232635775543270890529169229936851982759783745}";
    let serialized = serde_json::from_str::<serde_json::Value>(input).unwrap();
    let deserialized = serde_json::to_string(&serialized).unwrap();
    assert_eq!(input, deserialized);
}
