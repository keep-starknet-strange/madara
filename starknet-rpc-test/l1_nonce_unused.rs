extern crate starknet_rpc_test;

use rstest::rstest;
use starknet_providers::Provider;
use starknet_rpc_test::constants::SN_GOERLI_CHAIN_ID;
use starknet_rpc_test::fixtures::madara;
use starknet_rpc_test::MadaraClient;

#[rstest]
#[tokio::test]
async fn returns_unused_l1_nonce(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    assert!(rpc.l1_unused_nonce().await?);

    Ok(())
}