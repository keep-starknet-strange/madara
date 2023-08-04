extern crate starknet_rpc_test;

use starknet_ff::FieldElement;
use starknet_providers::Provider;
use starknet_rpc_test::{ExecutionStrategy, MadaraClient};

#[tokio::test]
async fn work_ok_up_to_1000() -> Result<(), anyhow::Error> {
    let madara = MadaraClient::new(ExecutionStrategy::Native).await;
    let rpc = &madara.starknet_client;

    assert_eq!(
        rpc.block_hash_and_number().await?.block_hash,
        FieldElement::from_hex_be("0x031ebd02657f940683ae7bddf19716932c56d463fc16662d14031f8635df52ad").unwrap()
    );
    assert_eq!(rpc.block_hash_and_number().await?.block_number, 0);

    // madara.create_block().await?;
    // assert_eq!(rpc.block_hash_and_number().await?, (0, 1));

    // madara.run_to_block(20).await?;
    // assert_eq!(rpc.block_hash_and_number().await?, (0, 20));

    // madara.create_n_blocks(4).await?;
    // assert_eq!(rpc.block_hash_and_number().await?, (0, 24));

    // madara.run_to_block(1000).await?;
    // assert_eq!(rpc.block_hash_and_number().await?, (0, 1000));

    Ok(())
}
