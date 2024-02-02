pub mod config;

use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use celestia_rpc::{BlobClient, HeaderClient};
use celestia_types::blob::SubmitOptions;
use celestia_types::nmt::Namespace;
use celestia_types::{Blob, Result as CelestiaTypesResult};
use ethers::types::{I256, U256};
use jsonrpsee::http_client::{HeaderMap, HeaderValue, HttpClient, HttpClientBuilder};
use reqwest::header;

use crate::{DaClient, DaError, DaMode};

#[derive(Clone, Debug)]
pub struct CelestiaClient {
    http_client: HttpClient,
    nid: Namespace,
    mode: DaMode,
}

#[async_trait]
impl DaClient for CelestiaClient {
    async fn publish_state_diff(&self, state_diff: Vec<U256>) -> Result<(), DaError> {
        let blob = self.get_blob_from_state_diff(state_diff).map_err(|e| DaError::FailedDataFetching(e.into()))?;

        let submitted_height = self.publish_data(&blob).await.map_err(|e| DaError::FailedDataSubmission(e.into()))?;

        // blocking call, awaiting on server side (Celestia Node) that a block with our data is included
        self.http_client
            .header_wait_for_height(submitted_height)
            .await
            .map_err(|e| DaError::FailedDataFetching(e.into()))?;

        self.verify_blob_was_included(submitted_height, blob).await?;

        Ok(())
    }

    async fn last_published_state(&self) -> Result<I256> {
        Ok(I256::from(1))
    }

    fn get_mode(&self) -> DaMode {
        self.mode
    }

    fn get_da_metric_labels(&self) -> HashMap<String, String> {
        [("name".into(), "celestia".into())].iter().cloned().collect()
    }
}

impl CelestiaClient {
    async fn publish_data(&self, blob: &Blob) -> Result<u64, DaError> {
        self.http_client
            .blob_submit(&[blob.clone()], SubmitOptions::default())
            .await
            .map_err(|e| DaError::FailedDataSubmission(e.into()))
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

    async fn verify_blob_was_included(&self, submitted_height: u64, blob: Blob) -> Result<(), DaError> {
        let received_blob = self
            .http_client
            .blob_get(submitted_height, self.nid, blob.commitment)
            .await
            .map_err(|e| DaError::FailedDataFetching(e.into()))?;
        received_blob.validate().map_err(|e| DaError::FailedDataValidation(e.into()))?;
        Ok(())
    }
}

impl TryFrom<config::CelestiaConfig> for CelestiaClient {
    type Error = DaError;

    fn try_from(conf: config::CelestiaConfig) -> Result<Self, Self::Error> {
        // Borrowed the below code from https://github.com/eigerco/lumina/blob/ccc5b9bfeac632cccd32d35ecb7b7d51d71fbb87/rpc/src/client.rs#L41.
        // Directly calling the function wasn't possible as the function is async. Since
        // we only need to initiate the http provider and not the ws provider, we don't need async
        let mut headers = HeaderMap::new();
        if let Some(auth_token) = conf.auth_token {
            let val = HeaderValue::from_str(&format!("Bearer {}", auth_token))?;
            headers.insert(header::AUTHORIZATION, val);
        }

        let http_client = HttpClientBuilder::default()
            .set_headers(headers)
            .build(conf.http_provider.as_str())
            .map_err(|e| DaError::FailedBuildingClient(e.into()))?;

        // Convert the input string to bytes
        let bytes = conf.nid.as_bytes();

        // Create a new Namespace from these bytes
        let nid = Namespace::new_v0(bytes).map_err(|e| DaError::FailedBuildingClient(e.into()))?;

        Ok(Self { http_client, nid, mode: conf.mode })
    }
}
