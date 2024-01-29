extern crate starknet_e2e_test;

use std::time::Duration;

use assert_matches::assert_matches;
use async_trait::async_trait;
use madara_runtime::opaque::Block;
use madara_test_runner::node::MadaraTempDir;
use madara_test_runner::{MadaraArgs, MadaraRunner, Settlement};
use mc_settlement::ethereum::StarknetContractClient;
use mc_settlement::{SettlementProvider, StarknetState};
use rstest::rstest;
use starknet_e2e_test::starknet_sovereign::StarknetSovereign;
use test_context::{test_context, AsyncTestContext};
use tokio::time::sleep;

struct Context {
    pub madara_temp_dir: MadaraTempDir,
    pub starknet_sovereign: StarknetSovereign,
}

impl Context {
    pub async fn launch_madara(&self) -> MadaraRunner {
        let settlement_conf = self.starknet_sovereign.create_settlement_conf(self.madara_temp_dir.data_path()).await;
        MadaraRunner::new(MadaraArgs {
            settlement: Some(Settlement::Ethereum),
            settlement_conf: Some(settlement_conf),
            base_path: Some(self.madara_temp_dir.base_path()),
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
        let madara_temp_dir = MadaraTempDir::default();
        Self { madara_temp_dir, starknet_sovereign }
    }

    async fn teardown(self) {
        self.madara_temp_dir.clear();
    }
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
