extern crate starknet_rpc_test;

use starknet_ff::FieldElement;
use starknet_providers::Provider;
use starknet_rpc_test::{ExecutionStrategy, MadaraClient};

#[tokio::test]
async fn work_ok_at_initialization() -> Result<(), anyhow::Error> {
    let madara = MadaraClient::new(ExecutionStrategy::Native).await;
    let rpc = madara.get_starknet_client();

    assert_eq!(
        rpc.block_hash_and_number().await?.block_hash,
        FieldElement::from_hex_be("0x031ebd02657f940683ae7bddf19716932c56d463fc16662d14031f8635df52ad").unwrap()
    );
    assert_eq!(rpc.block_hash_and_number().await?.block_number, 0);

    Ok(())
}
