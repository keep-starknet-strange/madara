pub mod config;
pub mod blob;

use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use anyhow::Result;
use async_trait::async_trait;
use ethers::types::{I256, U256};
use crate::{DaClient, DaMode};
use crate::grpcurl_command;
use crate::eigenda::blob::{DisperseBlobResponse, BlobStatusResponse, BlobStatus, BlobInfo};

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
        while blob_status != &BlobStatus::Confirmed {
            blob_status_response = self.get_blob_status(disperse_blob_response.request_id()).await?;
            blob_status = blob_status_response.status();
            thread::sleep(Duration::from_secs(5));
        }
        // TODO
        // confirm the blob against the EigenDA contracts with blob_status_response.info(): &BlobInfo { BlobHeader, BlobVerificationProof }
        // verifyBlob() is the primary function that needs to be invoked
        //      This function will take the blobHeader and blobVerificationProof as inputs and
        //      execute a series of checks to ensure the blob was signed for and stored properly in the EigenDA network.
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
    async fn disperse_blob(&self, state_diff: Vec<U256>) -> Result<DisperseBlobResponse, std::io::Error> {
        let payload = self.create_disperse_blob_payload(state_diff);
        let output = grpcurl_command!(
            "-proto", &self.config.proto_path,
            "-d", &payload,
            &self.config.grpc_provider,
            "disperser.Disperser/DisperseBlob"
        )?;
        if output.status.success() {
            let disperse_blob_response: DisperseBlobResponse = String::from_utf8(output.stdout).map_err(|err| {
                std::io::Error::new(std::io::ErrorKind::Other, err.to_string())
            })?.into();
            return Ok(disperse_blob_response)
        } else {
            let error_message = String::from_utf8(output.stderr).map_err(|err| {
                std::io::Error::new(std::io::ErrorKind::Other, err.to_string())
            })?;
            return Err(std::io::Error::new(std::io::ErrorKind::Other, error_message));
        }
    }

    async fn get_blob_status(&self, request_id: String) -> Result<BlobStatusResponse, std::io::Error> {
        let payload = Self::create_get_blob_status_payload(request_id);
        let output = grpcurl_command!(
            "-proto", &self.config.proto_path,
            "-d", &payload,
            &self.config.grpc_provider,
            "disperser.Disperser/GetBlobStatus"
        )?;
        if output.status.success() {
            let response: BlobStatusResponse = String::from_utf8(output.stdout).map_err(|err| {
                std::io::Error::new(std::io::ErrorKind::Other, err.to_string())
            })?.into();
            return Ok(response);
        } else {
            let error_message = String::from_utf8(output.stderr).map_err(|err| {
                std::io::Error::new(std::io::ErrorKind::Other, err.to_string())
            })?;
            return Err(std::io::Error::new(std::io::ErrorKind::Other, error_message))
        }
    }

    fn create_disperse_blob_payload(&self, state_diff: Vec<U256>) -> String {
        let mut payload = String::new();
        payload.push_str(r#"{"data":"#);
        payload.push_str(&format!(r#""{:?}""#, state_diff));
        payload.push_str(r#", "security_params":[{"quorum_id":"#);
        payload.push_str(&self.config.quorum_id.to_string());
        payload.push_str(r#","adversary_threshold":"#);
        payload.push_str(&self.config.adversary_threshold.to_string());
        payload.push_str(r#","quorum_threshold":"#);
        payload.push_str(&self.config.quorum_threshold.to_string());
        payload.push_str(r#"}]}"#);
        payload
    }

    fn create_get_blob_status_payload(request_id: String) -> String {
        let mut payload = String::new();
        payload.push_str(r#"{"#);
        payload.push_str(r#""request_id":"#);
        payload.push_str(&format!(r#""{}""#, request_id));
        payload.push_str(r#"}"#);
        payload
    }
}

impl TryFrom<config::EigenDaConfig> for EigenDaClient {
    type Error = anyhow::Error;

    fn try_from(conf: config::EigenDaConfig) -> Result<Self, Self::Error> {
        Ok(Self{ config: conf.clone(), mode: conf.mode })
    }
}    
