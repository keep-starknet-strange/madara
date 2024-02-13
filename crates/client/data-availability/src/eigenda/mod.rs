pub mod config;
pub mod blob;

use std::collections::HashMap;
use std::time::{Duration, Instant};
use reqwest::Error;
use reqwest::blocking::{Client, Response};
use anyhow::Result;
use serde_json;
use serde_json::{json, Value};
use serde::{Serialize, Deserialize};
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

const BATCH_CONFIRMED_EVENT_SIGNATURE: &str =  "0x2eaa707a79ac1f835863f5a6fdb5f27c0e295dc23adf970a445cd87d126c4d63"; // = keccak256(BatchConfirmed(bytes32,uint32,uint96))
const TIMEOUT_DURATION: u64 = 300;

pub struct EigenDaClient {
    config: config::EigenDaConfig,
    mode: DaMode,
}

#[async_trait]
impl DaClient for EigenDaClient {
    async fn publish_state_diff(&self, state_diff: Vec<U256>) -> Result<()> {
        let from_block = self.get_block_num().unwrap().result;
        let disperse_blob_response = self.disperse_blob(state_diff)?;
        let timeout_duration = Duration::from_secs(TIMEOUT_DURATION);
        let start_time = Instant::now();
        loop {
            match self.get_blob_status(disperse_blob_response.request_id()).await {
                Ok(blob_status_response) => {
                    match blob_status_response.status() {
                        BlobStatus::Processing => {
                            if start_time.elapsed() > timeout_duration {
                                return Err(anyhow::anyhow!("timeout error"))
                            }
                            tokio::time::sleep(Duration::from_secs(5)).await;
                        }
                        BlobStatus::Confirmed => {
                            let to_block = self.get_block_num().unwrap().result;
                            let event = self.get_batch_confirmed_event(
                                blob_status_response.info().blob_verification_proof().batch_metadata().batch_header_hash(),
                                &from_block,
                                &to_block
                            ).unwrap();
                            assert!(event.result.len() == 1);
                            // BRIDGE TO STARKNET
                            // calling the sendMessageToL2 function on the Starknet Core
                            // L1 → L2 messages are payable on L1, by sending ETH with the call to the payable function sendMessageToL2 on the Starknet Core Contract.
                            // DATA - The compiled code of a contract OR the hash of the invoked method signature and encoded parameters.
                            self.bridge_to_starknet(&event.result[0].transaction_hash);
                            // wait and query for confirmation?
                        }
                        BlobStatus::Failed | BlobStatus::Other(_) => {
                            return Err(anyhow::anyhow!("blob not accepted by EigenDA: {:?}", blob_status_response));
                        }
                    }
                }
                Err(_) => {
                    // should we retry if there is a grpc server error?
                    return Err(anyhow::anyhow!("GRPC call failed"))
                }
            }
        }
        // v1 (completed): 
        //      (disperser executes verifyBlob from EigenDABlobUtils.sol on Ethereum and emits batchConfirmed event)
        //      rollup queries for batchConfirmed event on a finalised Ethereum block
        //      rollup bridges batchConfirmed event to Starknet
        // v2: 
        //      rollup executes verifyBlob from EigenDABlobUtils.cairo on Starknet ? or will the disperser also do this??
        //      (rollup verifier contract (on Starknet) will read from the contract storage)
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
    fn disperse_blob(&self, state_diff: Vec<U256>) -> Result<DisperseBlobResponse> {
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

    fn get_block_num(&self) -> Result<EthereumResponse, Error> {
        let request_data = json!({
            "jsonrpc": "2.0",
            "method": "eth_blockNumber",
            "id": 1
        });
        let response = self.send_request(request_data)?;
        response.json::<EthereumResponse>()
    }

    fn get_batch_confirmed_event(
        &self, 
        batch_header_hash: &String,
        from_block: &String,
        to_block: &String,
    ) -> Result<GetLogsResponse, Error> {
        let request_data = json!({
            "jsonrpc": "2.0",
            "method": "eth_getLogs",
            "params": [
                {
                    "fromBlock": from_block,
                    "toBlock": to_block,
                    "address": self.config.eigenda_contract,
                    "topics": [
                        BATCH_CONFIRMED_EVENT_SIGNATURE,
                        batch_header_hash
                    ]
                }
            ],
            "id": 1
        });
        let response = self.send_request(request_data)?;
        response.json::<GetLogsResponse>()
    }

    fn bridge_to_starknet(
        &self,
        input: &String,
    ) -> Result<EthereumResponse, Error> {
        let request_data = json!({
            "jsonrpc": "2.0",
            "method": "eth_sendTransaction",
            "params": [
                {
                    "from": "0x1", // !!!
                    "input": input
                }
            ],
            "id": 1
        });
        let response = self.send_request(request_data)?;
        response.json::<EthereumResponse>()
    }

    fn send_request(
        &self,
        request_data: Value,
    ) -> Result<Response, Error> {
        let client = Client::new();
        let response = client
            .post(&self.config.eth_rpc_provider)
            .body(request_data.to_string())
            .send();
        response
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct EthereumResponse {
    jsonrpc: String,
    id: i32,
    pub result: String
}

#[derive(Serialize, Deserialize, Debug)]
struct GetLogsResponse {
    jsonrpc: String,
    id: i32,
    result: Vec<Event>      
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Event {
    block_hash: String,
    block_number: String,
    transaction_index: String,
    address: String,
    log_index: String,
    data: String,
    removed: bool,
    topics: Vec<String>,
    transaction_hash: String,
}

impl TryFrom<config::EigenDaConfig> for EigenDaClient {
    type Error = anyhow::Error;

    fn try_from(conf: config::EigenDaConfig) -> Result<Self, Self::Error> {
        Ok(Self{ config: conf.clone(), mode: conf.mode })
    }
}    
