extern crate starknet_rpc_test;

use rstest::rstest;
use starknet_providers::Provider;
use starknet_rpc_test::fixtures::madara;
use starknet_rpc_test::MadaraClient;

#[rstest]
#[tokio::test]
async fn work_ok_up_to_1000(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    assert_eq!(rpc.block_number().await?, 0);

    madara.create_empty_block().await?;
    assert_eq!(rpc.block_number().await?, 1);

    madara.run_to_block(20).await?;
    assert_eq!(rpc.block_number().await?, 20);

    madara.create_n_blocks(4).await?;
    assert_eq!(rpc.block_number().await?, 24);

    madara.run_to_block(1000).await?;
    assert_eq!(rpc.block_number().await?, 1000);

    Ok(())
}
