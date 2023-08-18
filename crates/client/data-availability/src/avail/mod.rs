use anyhow::Result;
use avail_subxt::api::runtime_types::avail_core::AppId;
use avail_subxt::api::runtime_types::da_control::pallet::Call as DaCall;
use avail_subxt::api::runtime_types::sp_core::bounded::bounded_vec::BoundedVec;
use avail_subxt::api::{self as AvailApi};
use avail_subxt::avail::{AppUncheckedExtrinsic, Client as AvailSubxtClient, PairSigner};
use avail_subxt::primitives::AvailExtrinsicParams;
use avail_subxt::{build_client, Call};
use ethers::types::U256;
use sp_core::H256;
use subxt::ext::sp_core::Pair;

use crate::DaClient;

pub struct AvailClient {
    ws_client: AvailSubxtClient,
    app_id: AppId,
    signer: PairSigner,
}

#[async_trait]
impl DaClient for CelestiaClient {
    pub async fn publish_state_diff(&self, state_diff: Vec<U256>) -> Result<bool, String> {
        let bytes = self.get_bytes_from_state_diff(state_diff)?;
        let bytes = BoundedVec(bytes);

        let submitted_block_hash = self.publish_data(&bytes).await?;

        self.verify_bytes_inclusion(submitted_block_hash, &bytes).await?;
        Ok(())
    }

    fn get_mode(&self) -> String {
        self.mode.clone()
    }
}

impl AvailClient {
    pub fn new(ws_endpoint: Option<&str>, app_id: Option<u32>, auth_token: Option<&str>) -> Result<Self> {
        let ws_endpoint = ws_endpoint.unwrap_or(AVAIL_WS);
        let app_id = AppId(app_id.unwrap_or(MADARA_DEFAULT_APP_ID));
        let seed = auth_token.unwrap_or(AVAIL_DEFAULT_SEED);
        let signer = signer_from_seed(seed)?;

        let ws_client = futures::executor::block_on(async { build_client(ws_endpoint, AVAIL_VALIDATE_CODEGEN).await })
            .map_err(|e| anyhow::anyhow!("Could not initialize ws endpoint {e}"))?;

        Ok(AvailClient { ws_client, app_id, signer })
    }



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

    fn get_bytes_from_state_diff(&self, state_diff: Vec<U256>) -> Result<Vec<u8>> {
        let state_diff_bytes: Vec<u8> = state_diff
            .iter()
            .flat_map(|item| {
                let mut bytes = [0_u8; 32];
                item.to_big_endian(&mut bytes);
                bytes.to_vec()
            })
            .collect();

        Ok(state_diff_bytes)
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
                Call::DataAvailability(da_call) => match da_call {
                    DaCall::submit_data { data } => data == bytes,
                    _ => false,
                },
                _ => false,
            })
            .ok_or(anyhow::anyhow!("Bytes not found in specified block"))?;

        Ok(())
    }
}

fn signer_from_seed(seed: &str) -> Result<PairSigner> {
    let pair = Pair::from_string(seed, None)?;
    Ok(PairSigner::new(pair))
}
