extern crate starknet_rpc_test;

use rstest::rstest;
use starknet_providers::Provider;
use starknet_rpc_test::fixtures::{madara, ThreadSafeMadaraClient};

#[rstest]
#[tokio::test]
async fn work_ok_up_to_1000(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    {
        let mut madara_write_lock = madara.write().await;
        let block_number = rpc.block_number().await?;

        madara_write_lock.create_empty_block().await?;
        assert_eq!(rpc.block_number().await?, 1 + block_number);

        madara_write_lock.run_to_block(block_number + 20).await?;
        assert_eq!(rpc.block_number().await?, 20 + block_number);

        madara_write_lock.create_n_blocks(4).await?;
        assert_eq!(rpc.block_number().await?, 24 + block_number);
    }

    Ok(())
}
