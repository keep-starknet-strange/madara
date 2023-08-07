extern crate starknet_rpc_test;

use rstest::{fixture, rstest};
use starknet_providers::Provider;
use starknet_rpc_test::constants::SN_GOERLI_CHAIN_ID;
use starknet_rpc_test::{ExecutionStrategy, MadaraClient};

#[fixture]
async fn madara() -> MadaraClient {
    MadaraClient::new(ExecutionStrategy::Native).await
}

#[rstest]
#[tokio::test]
async fn returns_hardcoded_chain_id(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    assert_eq!(rpc.chain_id().await?, SN_GOERLI_CHAIN_ID);

    Ok(())
}
