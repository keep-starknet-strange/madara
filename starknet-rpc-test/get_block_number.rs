extern crate starknet_rpc_test;

use starknet_rpc_test::{ExecutionStrategy, MadaraClient};

#[tokio::test]
async fn work_ok_up_to_1000() -> Result<(), anyhow::Error> {
    let madara = MadaraClient::new(ExecutionStrategy::Native).await;

    assert_eq!(madara.get_block_number().await?, 0);

    madara.create_block().await?;
    assert_eq!(madara.get_block_number().await?, 1);

    madara.run_to_block(20).await?;
    assert_eq!(madara.get_block_number().await?, 20);

    madara.create_n_blocks(4).await?;
    assert_eq!(madara.get_block_number().await?, 24);

    madara.run_to_block(1000).await?;
    assert_eq!(madara.get_block_number().await?, 1000);

    Ok(())
}
