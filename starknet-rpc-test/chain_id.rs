use madara_node_runner::constants::SN_GOERLI_CHAIN_ID;
use madara_node_runner::fixtures::madara;
use madara_node_runner::MadaraClient;
use rstest::rstest;
use starknet_providers::Provider;

#[rstest]
#[tokio::test]
async fn returns_hardcoded_chain_id(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    assert_eq!(rpc.chain_id().await?, SN_GOERLI_CHAIN_ID);

    Ok(())
}
