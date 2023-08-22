pub mod config;

use anyhow::Result;
use async_trait::async_trait;
use celestia_rpc::client::new_http;
use celestia_rpc::{BlobClient, HeaderClient};
use celestia_types::nmt::Namespace;
use celestia_types::{Blob, Result as CelestiaTypesResult};
use ethers::types::{I256, U256};
use jsonrpsee::http_client::HttpClient;

use crate::{DaClient, DaMode};

#[derive(Clone, Debug)]
pub struct CelestiaClient {
    http_client: HttpClient,
    nid: Namespace,
    mode: DaMode,
}

#[async_trait]
impl DaClient for CelestiaClient {
    async fn publish_state_diff(&self, state_diff: Vec<U256>) -> Result<()> {
        let blob = self.get_blob_from_state_diff(state_diff).map_err(|e| anyhow::anyhow!("celestia error: {e}"))?;
        let submitted_height = self.publish_data(&blob).await.map_err(|e| anyhow::anyhow!("celestia error: {e}"))?;

        // blocking call, awaiting on server side (Celestia Node) that a block with our data is included
        // not clean split between ws and http endpoints, which is why this call is blocking in the first
        // place...
        self.http_client
            .header_wait_for_height(submitted_height)
            .await
            .map_err(|e| anyhow::anyhow!("celestia da error: {e}"))?;
        self.verify_blob_was_included(submitted_height, blob)
            .await
            .map_err(|e| anyhow::anyhow!("celestia error: {e}"))?;

        Ok(())
    }

    async fn last_published_state(&self) -> Result<I256> {
        Ok(I256::from(1))
    }

    fn get_mode(&self) -> DaMode {
        self.mode
    }
}

impl CelestiaClient {
    pub fn try_from_config(conf: config::CelestiaConfig) -> Result<Self> {
        let http_client = new_http(conf.http_provider.clone().as_str(), conf.auth_token.as_deref())?;

        // Convert the input string to bytes
        let bytes = conf.nid.as_bytes();

        // Create a new Namespace from these bytes
        let nid = Namespace::new_v0(bytes).unwrap();

        Ok(Self { http_client, nid, mode: conf.mode })
    }

    async fn publish_data(&self, blob: &Blob) -> Result<u64> {
        self.http_client.blob_submit(&[blob.clone()]).await.map_err(|e| anyhow::anyhow!("could not submit blob {e}"))
    }

    fn get_blob_from_state_diff(&self, state_diff: Vec<U256>) -> CelestiaTypesResult<Blob> {
        let state_diff_bytes: Vec<u8> = state_diff
            .iter()
            .flat_map(|item| {
                let mut bytes = [0_u8; 32];
                item.to_big_endian(&mut bytes);
                bytes.to_vec()
            })
            .collect();

        Blob::new(self.nid, state_diff_bytes)
    }

    async fn verify_blob_was_included(&self, submitted_height: u64, blob: Blob) -> Result<()> {
        let received_blob = self.http_client.blob_get(submitted_height, self.nid, blob.commitment).await.unwrap();
        received_blob.validate()?;
        Ok(())
    }
}
