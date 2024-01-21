extern crate starknet_e2e_test;

use std::fs::{create_dir_all, File};
use std::time::Duration;

use assert_matches::assert_matches;
use async_trait::async_trait;
use ethers::providers::Middleware;
use madara_runtime::opaque::Block;
use madara_test_runner::{MadaraArgs, MadaraRunner, Settlement};
use mc_settlement::ethereum::client::EthereumConfig;
use mc_settlement::ethereum::StarknetContractClient;
use mc_settlement::{SettlementProvider, StarknetState};
use rstest::rstest;
use starknet_api::serde_utils::hex_str_from_bytes;
use starknet_e2e_test::starknet_sovereign::StarknetSovereign;
use tempfile::TempDir;
use test_context::{test_context, AsyncTestContext};
use tokio::time::sleep;

struct Context {
    pub madara_path: TempDir,
    pub starknet_sovereign: StarknetSovereign,
}

impl Context {
    pub async fn launch_madara(&self) -> MadaraRunner {
        MadaraRunner::new(MadaraArgs {
            settlement: Some(Settlement::Ethereum),
            settlement_conf: Some(self.madara_path.path().join("chains/dev/eth-config.json")),
            base_path: Some(self.madara_path.path().to_path_buf()),
        })
        .await
    }

    pub async fn read_state(&self) -> StarknetState {
        let client = StarknetContractClient::new(self.starknet_sovereign.address(), self.starknet_sovereign.client());
        SettlementProvider::<Block>::get_state(&client).await.expect("Failed to get state")
    }
}

#[async_trait]
impl AsyncTestContext for Context {
    async fn setup() -> Self {
        let starknet_sovereign = StarknetSovereign::deploy().await;

        let madara_path = TempDir::with_prefix("madara").expect("Failed to create Madara path");
        let config_dir = madara_path.path().join("chains/dev"); // data path
        create_dir_all(&config_dir).unwrap();

        let config = EthereumConfig {
            http_provider: starknet_sovereign.client().provider().url().to_string(),
            core_contracts: hex_str_from_bytes::<20, true>(starknet_sovereign.address().0),
            chain_id: starknet_sovereign.client().get_chainid().await.expect("Failed to get sandbox chain ID").as_u64(),
            poll_interval_ms: Some(10u64), // Default is 7s, we need to speed things up
            ..Default::default()
        };

        let config_file = File::create(config_dir.join("eth-config.json")).expect("Failed to open file for writing");
        serde_json::to_writer(config_file, &config).expect("Failed to write eth config");

        Self { madara_path, starknet_sovereign }
    }

    async fn teardown(self) {
        self.madara_path.close().expect("Failed to clean up");
    }
}

#[test_context(Context)]
#[rstest]
#[tokio::test]
async fn works_with_initialized_contract(ctx: &mut Context) -> Result<(), anyhow::Error> {
    // Troubleshooting:
    // RUST_LOG=mc_settlement=trace MADARA_LOG=1 cargo test --package starknet-e2e-test
    // works_with_initialized_contract -- --nocapture

    // At this point we have:
    //   * spawned Ethereum sandbox
    //   * deployed settlement contract (not initialized yet)
    //   * temp Madara path with correct ethereum config
    ctx.starknet_sovereign.initialize_for_goerli(0u64.into(), 0u64.into()).await;

    let mut madara = ctx.launch_madara().await;

    madara.create_n_blocks(3).await?;
    sleep(Duration::from_millis(300)).await;

    assert_eq!(ctx.read_state().await, StarknetState { block_number: 3u64.into(), state_root: 0u64.into() });

    Ok(())
}

#[test_context(Context)]
#[rstest]
#[tokio::test]
async fn recovers_from_non_initialized_state(ctx: &mut Context) -> Result<(), anyhow::Error> {
    let mut madara = ctx.launch_madara().await;

    madara.create_empty_block().await?;
    // Give the client thread some time to handle the finalized block
    sleep(Duration::from_millis(100)).await;

    ctx.starknet_sovereign.initialize_for_goerli(0u64.into(), 0u64.into()).await;

    madara.create_empty_block().await?;
    // Give the client thread some time to recover
    sleep(Duration::from_millis(100)).await;

    madara.create_empty_block().await?;
    sleep(Duration::from_millis(100)).await;

    assert_eq!(ctx.read_state().await, StarknetState { block_number: 3u64.into(), state_root: 0u64.into() });

    Ok(())
}

#[test_context(Context)]
#[rstest]
#[tokio::test]
async fn catches_up_with_the_state_in_the_future(ctx: &mut Context) -> Result<(), anyhow::Error> {
    ctx.starknet_sovereign.initialize_for_goerli(1u64.into(), 0u64.into()).await;

    let mut madara = ctx.launch_madara().await;

    // Unless state root calculation is enabled (not by default), we should be fine
    madara.create_n_blocks(2).await?;
    // Give the client thread some time to handle the finalized block
    sleep(Duration::from_millis(200)).await;

    assert_eq!(ctx.read_state().await, StarknetState { block_number: 2u64.into(), state_root: 0u64.into() });

    Ok(())
}

#[test_context(Context)]
#[rstest]
#[tokio::test]
async fn fails_with_inconsistent_state_in_the_future(ctx: &mut Context) -> Result<(), anyhow::Error> {
    ctx.starknet_sovereign.initialize_for_goerli(1u64.into(), 12345u64.into()).await;

    let mut madara = ctx.launch_madara().await;

    madara.create_empty_block().await?;
    // Give the client thread some time to handle the finalized block
    sleep(Duration::from_millis(100)).await;

    // Expected connection refused because Madara is shut down at this point
    assert_matches!(madara.create_empty_block().await, Err(err) => assert!(err.downcast_ref::<reqwest::Error>().is_some()));

    Ok(())
}

#[test_context(Context)]
#[rstest]
#[tokio::test]
async fn fails_with_inconsistent_starknet_spec(ctx: &mut Context) -> Result<(), anyhow::Error> {
    ctx.starknet_sovereign.initialize(1u64.into(), 0u64.into()).await;

    let mut madara = ctx.launch_madara().await;

    madara.create_empty_block().await?;
    // Give the client thread some time to handle the finalized block
    sleep(Duration::from_millis(100)).await;

    // Expected connection refused because Madara is shut down at this point
    assert_matches!(madara.create_empty_block().await, Err(err) => assert!(err.downcast_ref::<reqwest::Error>().is_some()));

    Ok(())
}
