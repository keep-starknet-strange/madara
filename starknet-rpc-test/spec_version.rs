extern crate starknet_rpc_test;

use rstest::rstest;
use starknet_providers::Provider;
use starknet_rpc_test::constants::SPEC_VERSION;
use starknet_rpc_test::fixtures::{madara, ThreadSafeMadaraClient};

#[rstest]
#[tokio::test]
async fn returns_hardcoded_spec_version(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    // TODO: test it when starknet_providers::jsonrpc upgrades to v0.6.0
    assert_eq!(rpc.spec_version().await?, SPEC_VERSION);

    Ok(())
}
