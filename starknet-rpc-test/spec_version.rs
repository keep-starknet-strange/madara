extern crate starknet_rpc_test;

use rstest::rstest;
use starknet_rpc_test::fixtures::{madara, ThreadSafeMadaraClient};

#[rstest]
#[tokio::test]
#[ignore = "Waiting for starknet_providers::jsonrpc upgrade to v0.6.0"]
async fn returns_hardcoded_spec_version(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    Ok(())
}
