extern crate starknet_e2e_test;

use std::time::Duration;

use madara_runtime::opaque::Block;
use madara_test_runner::node::MadaraTempDir;
use madara_test_runner::{MadaraArgs, MadaraRunner, Settlement};
use mc_settlement::ethereum::StarknetContractClient;
use mc_settlement::{SettlementProvider, StarknetState};
use rstest::rstest;
use starknet_e2e_test::starknet_sovereign::StarknetSovereign;
use tokio::time::sleep;

#[rstest]
#[tokio::test]
async fn madara_advances_ethereum_settlement_contract_state_in_sovereign_mode() -> Result<(), anyhow::Error> {
    // Troubleshooting:
    // RUST_LOG=mc_settlement=trace MADARA_LOG=1 cargo test --package starknet-e2e-test
    // works_with_initialized_contract -- --nocapture

    // Run or attach to Anvil sandbox, deploy & initialize core contract
    let starknet_sovereign = StarknetSovereign::deploy().await;
    starknet_sovereign.initialize_for_goerli(0u64.into(), 0u64.into()).await;

    // Create tmp Madara path and write settlement config
    let madara_temp_dir = MadaraTempDir::default();
    let settlement_conf = starknet_sovereign.create_settlement_conf(madara_temp_dir.data_path()).await;

    // Launch new Madara instance and connect to it
    let mut madara = MadaraRunner::new(MadaraArgs {
        settlement: Some(Settlement::Ethereum),
        settlement_conf: Some(settlement_conf),
        base_path: Some(madara_temp_dir.base_path()),
    })
    .await;

    madara.create_n_blocks(3).await?;
    sleep(Duration::from_millis(300)).await;

    let client = StarknetContractClient::new(starknet_sovereign.address(), starknet_sovereign.client());
    let state = SettlementProvider::<Block>::get_state(&client).await?;

    assert_eq!(state, StarknetState { block_number: 3u64.into(), state_root: 0u64.into() });

    Ok(())
}
