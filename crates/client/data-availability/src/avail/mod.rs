pub mod config;

use anyhow::Result;
use async_trait::async_trait;
use avail_subxt::api::runtime_types::avail_core::AppId;
use avail_subxt::api::runtime_types::da_control::pallet::Call as DaCall;
use avail_subxt::api::runtime_types::sp_core::bounded::bounded_vec::BoundedVec;
use avail_subxt::avail::{AppUncheckedExtrinsic, Client as AvailSubxtClient};
use avail_subxt::primitives::AvailExtrinsicParams;
use avail_subxt::{api as AvailApi, build_client, AvailConfig, Call};
use ethers::types::{I256, U256};
use sp_core::H256;
use subxt::ext::sp_core::sr25519::Pair;

use crate::utils::get_bytes_from_state_diff;
use crate::{DaClient, DaMode};

type AvailPairSigner = subxt::tx::PairSigner<AvailConfig, Pair>;

#[derive(Clone)]
pub struct AvailClient {
    ws_client: AvailSubxtClient,
    app_id: AppId,
    signer: AvailPairSigner,
    mode: DaMode,
}

#[async_trait]
impl DaClient for AvailClient {
    async fn publish_state_diff(&self, state_diff: Vec<U256>) -> Result<()> {
        let bytes = get_bytes_from_state_diff(&state_diff);
        let bytes = BoundedVec(bytes);

        let submitted_block_hash = self.publish_data(&bytes).await?;

        self.verify_bytes_inclusion(submitted_block_hash, &bytes).await?;
        Ok(())
    }

    // state diff can be published w/o verification of last state for the time being
    // may change in subsequent DaMode implementations
    async fn last_published_state(&self) -> Result<I256> {
        Ok(I256::from(1))
    }

    fn get_mode(&self) -> DaMode {
        self.mode
    }
}

impl AvailClient {
    async fn publish_data(&self, bytes: &BoundedVec<u8>) -> Result<H256> {
        let data_transfer = AvailApi::tx().data_availability().submit_data(bytes.clone());
        let extrinsic_params = AvailExtrinsicParams::new_with_app_id(self.app_id);
        let events = self
            .ws_client
            .tx()
            .sign_and_submit_then_watch(&data_transfer, &self.signer, extrinsic_params)
            .await?
            .wait_for_finalized_success()
            .await?;

        Ok(events.block_hash())
    }

    async fn verify_bytes_inclusion(&self, block_hash: H256, bytes: &BoundedVec<u8>) -> Result<()> {
        let submitted_block = self
            .ws_client
            .rpc()
            .block(Some(block_hash))
            .await?
            .ok_or(anyhow::anyhow!("Invalid hash, block not found"))?;

        submitted_block
            .block
            .extrinsics
            .into_iter()
            .filter_map(|chain_block_ext| AppUncheckedExtrinsic::try_from(chain_block_ext).map(|ext| ext.function).ok())
            .find(|call| match call {
                Call::DataAvailability(DaCall::submit_data { data }) => data == bytes,
                _ => false,
            })
            .ok_or(anyhow::anyhow!("Bytes not found in specified block"))?;

        Ok(())
    }
}

impl TryFrom<config::AvailConfig> for AvailClient {
    type Error = anyhow::Error;

    fn try_from(conf: config::AvailConfig) -> Result<Self, Self::Error> {
        let signer = signer_from_seed(conf.seed.as_str())?;

        let app_id = AppId(conf.app_id);

        let ws_client =
            futures::executor::block_on(async { build_client(conf.ws_provider.as_str(), conf.validate_codegen).await })
                .map_err(|e| anyhow::anyhow!("could not initialize ws endpoint {e}"))?;

        Ok(Self { ws_client, app_id, signer, mode: conf.mode })
    }
}

fn signer_from_seed(seed: &str) -> Result<AvailPairSigner> {
    let pair = <Pair as subxt::ext::sp_core::Pair>::from_string(seed, None)?;
    let signer = AvailPairSigner::new(pair);
    Ok(signer)
}
