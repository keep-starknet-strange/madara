pub mod config;

use async_trait::async_trait;
use celestia_rpc::client::new_http;
use celestia_rpc::BlobClient;
use celestia_types::nmt::Namespace;
use celestia_types::{Blob, Error as CelestiaError, Result as CelestiaTypesResult};
use ethers::types::U256;
use jsonrpsee::core::Error as JsonRpSeeError;
use jsonrpsee::http_client::HttpClient;

use crate::DaClient;

pub struct CelestiaClient {
    http_client: HttpClient,
    mode: String,
    nid: Namespace,
}

#[async_trait]
impl DaClient for CelestiaClient {
    async fn publish_state_diff(&self, state_diff: Vec<U256>) -> Result<bool, String> {
        let blob = self.get_blob_from_state_diff(state_diff).unwrap();
        let submitted_height = self.publish_data(&blob).await.unwrap();

        // blocking call, awaiting on server side (Celestia Node) that a block with our data is included
        // not clean split between ws and http endpoints, which is why this call is blocking in the first
        // place...
        // self.ws_client.header_wait_for_height(submitted_height).await.unwrap();
        self.verify_blob_was_included(submitted_height, blob).await.unwrap();

        Ok(true)
    }

    fn get_mode(&self) -> String {
        self.mode.clone()
    }
}

impl CelestiaClient {
    pub fn new(conf: config::CelestiaConfig) -> Result<Self, celestia_rpc::Error> {
        let http_client = new_http(conf.http_provider.clone().as_str(), conf.auth_token.as_deref())?;

        // Convert the input string to bytes
        let bytes = conf.nid.as_bytes();

        // Create a new Namespace from these bytes
        let nid = Namespace::new_v0(bytes).unwrap();

        Ok(Self { http_client, mode: conf.mode, nid })
    }

    async fn publish_data(&self, blob: &Blob) -> Result<u64, JsonRpSeeError> {
        self.http_client.blob_submit(&[blob.clone()]).await
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

    async fn verify_blob_was_included(&self, submitted_height: u64, blob: Blob) -> Result<(), CelestiaError> {
        let received_blob = self.http_client.blob_get(submitted_height, self.nid, blob.commitment).await.unwrap();
        received_blob.validate()?;
        Ok::<(), CelestiaError>(())
    }
}
