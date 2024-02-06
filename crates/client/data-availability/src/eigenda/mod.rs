pub mod config;
pub mod blob;

use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use tokio::time::timeout;
use anyhow::Result;
use serde_json;
use async_trait::async_trait;
use ethers::types::{I256, U256};
use crate::{DaClient, DaMode};
use crate::grpcurl_command;
use crate::eigenda::blob::{
    DisperseBlobPayload, DisperseBlobResponse, BlobStatusPayload, BlobStatusResponse, 
    BlobStatus, //BlobInfo
};

#[macro_use]
mod macros;

pub struct EigenDaClient {
    config: config::EigenDaConfig,
    mode: DaMode,
}

#[async_trait]
impl DaClient for EigenDaClient {
    async fn publish_state_diff(&self, state_diff: Vec<U256>) -> Result<()> {
        let disperse_blob_response = self.disperse_blob(state_diff).await?;
        let mut blob_status = disperse_blob_response.status();
        // Best practice is for users to poll the GetBlobStatus service to monitor status of the Blobs as needed. 
        // Rollups may Polling once every 5-10 seconds to monitor the status of a blob until it has successfully dispersed on the network with status of CONFIRMED. 
        // Confirmation can take up to a few minutes after the blob has been initially sent to the disperser, depending on network conditions.
        // for more info see https://docs.eigenlayer.xyz/eigenda-guides/eigenda-rollup-user-guides/building-on-top-of-eigenda
        let mut blob_status_response: BlobStatusResponse;
        let timeout_duration = Duration::from_secs(300);
        while blob_status != &BlobStatus::Confirmed {
            match timeout(timeout_duration, self.get_blob_status(disperse_blob_response.request_id())).await {
                Ok(Ok(response)) => {
                    blob_status_response = response;
                    blob_status = blob_status_response.status();
                    if blob_status == &BlobStatus::Failed || matches!(blob_status, &BlobStatus::Other(_)) {
                        return Err(anyhow::anyhow!("Blob rejected by EigenDA {:?}", blob_status));
                    }
                    thread::sleep(Duration::from_secs(5));
                },
                Ok(Err(_)) => {
                    // get_blob_status gRPC call fail
                    // forward error to stderr?
                    // should we just return the error and give up on eigenda if the grpc call fails?
                    thread::sleep(Duration::from_secs(5));
                },
                Err(elapsed) => {
                    return Err(anyhow::anyhow!("timeout error: {}", elapsed))
                }
            }
        }
        Ok(())
    }

    async fn last_published_state(&self) -> Result<I256> {
        Ok(I256::from(1))
    }

    fn get_da_metric_labels(&self) -> HashMap<String, String> {
        [("name".into(), "eigenda".into())].iter().cloned().collect()
    }

    fn get_mode(&self) -> DaMode {
        self.mode
    }
}

impl EigenDaClient {
    // EigenDA gRPC server(s) are not currently working with tonic
    // instead we use a macro to fork a command to the command line to send the gRPC requests
    async fn disperse_blob(&self, state_diff: Vec<U256>) -> Result<DisperseBlobResponse> {
        let payload = serde_json::to_string(&DisperseBlobPayload::new(
            state_diff, 
            &self.config.quorum_id,
            &self.config.adversary_threshold,
            &self.config.quorum_threshold,
        ))?;
        let output = grpcurl_command!(
            "-proto", &self.config.proto_path,
            "-d", &payload,
            &self.config.grpc_provider,
            "disperser.Disperser/DisperseBlob"
        )?;
        if output.status.success() {
            let response: DisperseBlobResponse = serde_json::from_slice(&output.stdout)?;
            return Ok(response)
        } else {
            let error_message = String::from_utf8(output.stderr)?;
            return Err(anyhow::anyhow!("disperse_blob gRPC call failed: {}", error_message));
        }
    }

    async fn get_blob_status(&self, request_id: String) -> Result<BlobStatusResponse> {
        let payload = serde_json::to_string(&BlobStatusPayload::new(request_id))?;
        let output = grpcurl_command!(
            "-proto", &self.config.proto_path,
            "-d", &payload,
            &self.config.grpc_provider,
            "disperser.Disperser/GetBlobStatus"
        )?;
        if output.status.success() {
            let response: BlobStatusResponse = serde_json::from_slice(&output.stdout)?;
            return Ok(response);
        } else {
            let error_message = String::from_utf8(output.stderr)?;
            return Err(anyhow::anyhow!("get_blob_status gRPC call failed: {}", error_message))
        }
    }
}

impl TryFrom<config::EigenDaConfig> for EigenDaClient {
    type Error = anyhow::Error;

    fn try_from(conf: config::EigenDaConfig) -> Result<Self, Self::Error> {
        Ok(Self{ config: conf.clone(), mode: conf.mode })
    }
}    
