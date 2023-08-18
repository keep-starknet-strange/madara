use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use avail_subxt::api::runtime_types::avail_core::AppId;
use avail_subxt::api::runtime_types::da_control::pallet::Call as DaCall;
use avail_subxt::api::runtime_types::sp_core::bounded::bounded_vec::BoundedVec;
use avail_subxt::avail::{AppUncheckedExtrinsic, Client as AvailSubxtClient};
use avail_subxt::primitives::AvailExtrinsicParams;
use avail_subxt::{api as AvailApi, build_client, AvailConfig, Call};
use ethers::types::U256;
use subxt::ext::sp_core::sr25519::Pair;

use crate::da::{DAArgs, DAInput, DAOutput, DataAvailability};
use crate::utils::{get_bytes_from_state_diff, is_valid_ws_endpoint};

const AVAIL_VALIDATE_CODEGEN: bool = true;

type AvailPairSigner = subxt::tx::PairSigner<AvailConfig, Pair>;

fn signer_from_seed(seed: &str) -> Result<AvailPairSigner> {
    let pair = <Pair as subxt::ext::sp_core::Pair>::from_string(seed, None)?;
    let signer = AvailPairSigner::new(pair);
    Ok(signer)
}

pub struct AvailClient {
    ws_client: AvailSubxtClient,
    app_id: AppId,
    signer: AvailPairSigner,
}

#[async_trait]
impl DataAvailability for AvailClient {
    fn new(da_config: &HashMap<String, String>) -> Result<Self> {
        if let DAArgs::Avail { ws_endpoint, app_id, avail_seed } = Self::validate_args(da_config)? {
            let signer = signer_from_seed(&avail_seed)?;
            let ws_client =
                futures::executor::block_on(async { build_client(ws_endpoint, AVAIL_VALIDATE_CODEGEN).await })
                    .map_err(|e| anyhow::anyhow!("Could not initialize ws endpoint {e}"))?;

            Ok(AvailClient { ws_client, app_id, signer })
        } else {
            Err(anyhow::anyhow!("Invalid parameters"))
        }
    }

    fn validate_args(da_config: &HashMap<String, String>) -> Result<DAArgs> {
        if da_config.len() != 4 {
            return Err(anyhow::anyhow!("Expected 4 arguments for Avail but received {}", da_config.len()));
        }

        let app_id: u32 = da_config
            .get("app_id")
            .ok_or_else(|| anyhow::anyhow!("Missing app_id."))?
            .clone()
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid app_id {e}."))?;
        let app_id = AppId(app_id);
        let ws_endpoint = da_config.get("ws_endpoint").ok_or_else(|| anyhow::anyhow!("Missing ws_endpoint."))?.clone();
        let avail_seed = da_config.get("avail_seed").ok_or_else(|| anyhow::anyhow!("Missing avail_seed."))?.clone();

        if !is_valid_ws_endpoint(&ws_endpoint) {
            return Err(anyhow::anyhow!("Invalid ws endpoint, received {}", ws_endpoint));
        }

        Ok(DAArgs::Avail { ws_endpoint, app_id, avail_seed })
    }

    fn format_state_diff(&self, state_diff: &[U256]) -> Result<DAInput> {
        Ok(DAInput::Avail(BoundedVec(get_bytes_from_state_diff(state_diff))))
    }

    async fn publish_data(&self, data: &DAInput) -> Result<DAOutput> {
        if let DAInput::Avail(bytes) = data {
            let data_transfer = AvailApi::tx().data_availability().submit_data(bytes.clone());
            let extrinsic_params = AvailExtrinsicParams::new_with_app_id(self.app_id);
            let events = self
                .ws_client
                .tx()
                .sign_and_submit_then_watch(&data_transfer, &self.signer, extrinsic_params)
                .await?
                .wait_for_finalized_success()
                .await?;

            Ok(DAOutput::Avail { block_hash: events.block_hash() })
        } else {
            Err(anyhow::anyhow!("Invalid input data"))
        }
    }

    async fn verify_inclusion(&self, da_input: &DAInput, da_output: &DAOutput) -> Result<()> {
        match (da_input, da_output) {
            (DAInput::Avail(data), DAOutput::Avail { block_hash }) => {
                let submitted_block = self
                    .ws_client
                    .rpc()
                    .block(Some(*block_hash))
                    .await?
                    .ok_or(anyhow::anyhow!("Invalid hash, block not found"))?;

                submitted_block
                    .block
                    .extrinsics
                    .into_iter()
                    .filter_map(|chain_block_ext| {
                        AppUncheckedExtrinsic::try_from(chain_block_ext).map(|ext| ext.function).ok()
                    })
                    .find(|call| match call {
                        Call::DataAvailability(DaCall::submit_data { data: chain_data }) => chain_data == data,
                        _ => false,
                    })
                    .ok_or(anyhow::anyhow!("Bytes not found in specified block"))?;

                Ok(())
            }
            _ => Err(anyhow::anyhow!("Invalid input or output")),
        }
    }
}

impl AvailClient {
    #[cfg(test)]
    async fn new_test() -> Result<Self> {
        let mut avail_map: HashMap<String, String> = HashMap::new();
        avail_map.insert("da_type".to_string(), "Avail".to_string());
        avail_map.insert("ws_endpoint".to_string(), "ws://127.0.0.1:9945".to_string());
        avail_map.insert("app_id".to_string(), "0".to_string());
        avail_map.insert("avail_seed".to_string(), "//Bob".to_string());

        if let DAArgs::Avail { ws_endpoint, app_id, avail_seed } = Self::validate_args(&avail_map)? {
            let signer = signer_from_seed(&avail_seed)?;
            let ws_client = build_client(ws_endpoint, AVAIL_VALIDATE_CODEGEN).await.unwrap();

            Ok(AvailClient { ws_client, app_id, signer })
        } else {
            Err(anyhow::anyhow!("Invalid parameters"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_publish_data_and_verify_publication() -> Result<()> {
        let da_client = AvailClient::new_test().await.unwrap();
        let state_diff = vec![ethers::types::U256::from(0)];
        let da_input = da_client.format_state_diff(&state_diff).unwrap();
        let da_output = da_client.publish_data(&da_input).await.unwrap();
        let is_included = da_client.verify_inclusion(&da_input, &da_output).await;
        assert!(is_included.is_ok());

        Ok(())
    }
}
