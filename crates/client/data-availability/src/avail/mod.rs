pub mod config;

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use avail_subxt::api::runtime_types::avail_core::AppId;
use avail_subxt::api::runtime_types::bounded_collections::bounded_vec::BoundedVec;
use avail_subxt::avail::Client as AvailSubxtClient;
use avail_subxt::primitives::AvailExtrinsicParams;
use avail_subxt::{api as AvailApi, build_client, AvailConfig};
use ethers::types::{I256, U256};
use futures::lock::Mutex;
use subxt::ext::sp_core::sr25519::Pair;
use subxt::OnlineClient;

use crate::utils::get_bytes_from_state_diff;
use crate::{DaClient, DaError, DaLayer, DaMode};

type AvailPairSigner = subxt::tx::PairSigner<AvailConfig, Pair>;

#[derive(Clone)]
pub struct AvailClient {
    ws_client: Arc<Mutex<SubxtClient>>,
    app_id: AppId,
    signer: AvailPairSigner,
    mode: DaMode,
}

pub struct SubxtClient {
    client: AvailSubxtClient,
    config: config::AvailConfig,
}

pub fn try_build_avail_subxt(conf: &config::AvailConfig) -> Result<OnlineClient<AvailConfig>, DaError> {
    let client =
        futures::executor::block_on(async { build_client(conf.ws_provider.as_str(), conf.validate_codegen).await })
            .map_err(|e| DaError::FailedBuildingClient(e.into()))?;

    Ok(client)
}

impl SubxtClient {
    pub async fn restart(&mut self) -> Result<(), DaError> {
        self.client = match build_client(self.config.ws_provider.as_str(), self.config.validate_codegen).await {
            Ok(i) => i,
            Err(e) => return DaError::FailedBuildingClient(e.into()),
        };

        Ok(())
    }

    pub fn client(&self) -> &OnlineClient<AvailConfig> {
        &self.client
    }
}

impl TryFrom<config::AvailConfig> for SubxtClient {
    type Error = DaError;

    fn try_from(conf: config::AvailConfig) -> Result<Self, Self::Error> {
        Ok(Self { client: try_build_avail_subxt(&conf)?, config: conf })
    }
}

#[async_trait]
impl DaClient for AvailClient {
    async fn publish_state_diff(&self, state_diff: Vec<U256>) -> Result<(), DaError> {
        let bytes = get_bytes_from_state_diff(&state_diff);
        let bytes = BoundedVec(bytes);
        self.publish_data(&bytes).await?;

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

    fn get_da_metric_labels(&self) -> HashMap<String, String> {
        [("name".into(), "avail".into()), ("app_id".into(), self.app_id.0.to_string())].iter().cloned().collect()
    }
}

impl AvailClient {
    async fn publish_data(&self, bytes: &BoundedVec<u8>) -> Result<(), DaError> {
        let mut ws_client = self.ws_client.lock().await;

        let data_transfer = AvailApi::tx().data_availability().submit_data(bytes.clone());
        let extrinsic_params = AvailExtrinsicParams::new_with_app_id(self.app_id);

        match ws_client.client().tx().sign_and_submit(&data_transfer, &self.signer, extrinsic_params).await {
            Ok(i) => i,
            Err(e) => {
                if e.to_string().contains("restart required") {
                    let _ = ws_client.restart().await;
                }

                return DaError::FailedBuildingClient(e.into());
            }
        };

        Ok(())
    }
}

impl TryFrom<config::AvailConfig> for AvailClient {
    type Error = DaError;

    fn try_from(conf: config::AvailConfig) -> Result<Self, Self::Error> {
        let signer = signer_from_seed(conf.seed.as_str())?;

        let app_id = AppId(conf.app_id);

        Ok(Self {
            ws_client: Arc::new(Mutex::new(SubxtClient::try_from(conf.clone())?)),
            app_id,
            signer,
            mode: conf.mode,
        })
    }
}

fn signer_from_seed(seed: &str) -> Result<AvailPairSigner, DaError> {
    let pair = <Pair as subxt::ext::sp_core::Pair>::from_string(seed, None)
        .map_err(|e| DaError::FailedConversion(e.into()))?;

    let signer = AvailPairSigner::new(pair);
    Ok(signer)
}
