use rstest::rstest;
use starknet_providers::Provider;
use starknet_test_utils::constants::SN_GOERLI_CHAIN_ID;
use starknet_test_utils::fixtures::{madara, ThreadSafeMadaraClient};

#[rstest]
#[tokio::test]
async fn returns_hardcoded_chain_id(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    assert_eq!(rpc.chain_id().await?, SN_GOERLI_CHAIN_ID);

    Ok(())
}
