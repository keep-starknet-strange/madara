extern crate starknet_rpc_test;

use rstest::rstest;
use starknet_core::types::BlockId;
use starknet_providers::Provider;
use starknet_rpc_test::fixtures::{madara, ThreadSafeMadaraClient};

#[rstest]
#[tokio::test]
async fn work_ok_at_start_and_with_new_blocks(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    {
        let _madara_write_lock = madara.write();
        let block_number = rpc.block_number().await?;
        let (hash, number) = match rpc.get_block_with_tx_hashes(BlockId::Number(block_number)).await.unwrap() {
            starknet_core::types::MaybePendingBlockWithTxHashes::Block(b) => (b.block_hash, b.block_number),
            _ => panic!(),
        };

        let res = rpc.block_hash_and_number().await?;
        assert_eq!(res.block_hash, hash);
        assert_eq!(res.block_number, number);
    }

    Ok(())
}
