pub mod config;
pub mod blob;

use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::sync::Arc;
use reqwest::Error;
use reqwest::blocking::{Client, Response};
use anyhow::Result;
use serde_json;
use serde_json::{json, Value};
use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use ethers::providers::{Http, Provider};
use ethers::signers::{LocalWallet, Signer};
use ethers::prelude::{abigen, SignerMiddleware};
use ethers::types::{Address, I256, U256};
use crate::{DaClient, DaMode};
use crate::grpcurl_command;
use crate::eigenda::blob::{
    DisperseBlobPayload, DisperseBlobResponse, BlobStatusPayload, BlobStatusResponse, BlobStatus
};

#[macro_use]
mod macros;

const BATCH_CONFIRMED_EVENT_SIGNATURE: &str =  "0x2eaa707a79ac1f835863f5a6fdb5f27c0e295dc23adf970a445cd87d126c4d63"; // = keccak256(BatchConfirmed(bytes32,uint32,uint96))
const TIMEOUT_DURATION: u64 = 300; // can put into config later
const DEFAULT_STARKNET_CORE_CONTRACT: &str = "0xc662c410C0ECf747543f5bA90660f6ABeBD9C8c4"; // change to testnet?

pub struct EigenDaClient {
    config: config::EigenDaConfig,
    signer: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
}

#[async_trait]
impl DaClient for EigenDaClient {
    async fn publish_state_diff(&self, state_diff: Vec<U256>) -> Result<()> {
        // v1 (): 
        //      (disperser executes verifyBlob from EigenDABlobUtils.sol on Ethereum and emits batchConfirmed event)
        //      rollup queries for batchConfirmed event on a finalised Ethereum block
        //      rollup bridges batchConfirmed event to Starknet
        //      placeholder cairo contract to recieve message from L1 (TODO)
        // v2: 
        //      no bridging
        //      rollup executes verifyBlob from EigenDABlobUtils.cairo on Starknet ? or will the disperser also do this??
        //      (rollup verifier contract (on Starknet) will read from the contract storage)
        let from_block = self.get_block_num().unwrap().result;
        let disperse_blob_response = self.disperse_blob(state_diff)?;
        let timeout_duration = Duration::from_secs(TIMEOUT_DURATION);
        let start_time = Instant::now();
        loop {
            match self.get_blob_status(disperse_blob_response.request_id()).await {
                Ok(blob_status_response) => {
                    match blob_status_response.status() {
                        BlobStatus::Confirmed => {
                            let to_block = self.get_block_num().unwrap().result;
                            let event = self.get_batch_confirmed_event(
                                blob_status_response.info().blob_verification_proof().batch_metadata().batch_header_hash(),
                                &from_block,
                                &to_block
                            ).unwrap();
                            assert!(event.result.len() == 1);
                            // BRIDGE TO STARKNET
                            self.bridge_to_starknet(&event.result[0].transaction_hash);
                            // wait and query for confirmation?
                        }
                        BlobStatus::Processing => {
                            if start_time.elapsed() > timeout_duration {
                                return Err(anyhow::anyhow!("timeout error"))
                            }
                            tokio::time::sleep(Duration::from_secs(5)).await;
                        }
                        BlobStatus::Failed | BlobStatus::Other(_) => {
                            return Err(anyhow::anyhow!("blob not accepted by EigenDA: {:?}", blob_status_response));
                        }
                    }
                }
                Err(_) => {
                    return Err(anyhow::anyhow!("GRPC call failed"))
                    // should we retry if there is a grpc server error?
                }
            }
        }
    }

    async fn last_published_state(&self) -> Result<I256> {
        Ok(I256::from(1))
    }

    fn get_da_metric_labels(&self) -> HashMap<String, String> {
        [("name".into(), "eigenda".into())].iter().cloned().collect()
    }

    fn get_mode(&self) -> DaMode {
        self.config.mode
    }
}

impl EigenDaClient {
    // EigenDA gRPC server(s) are not currently working with tonic
    // instead we use a macro to fork a command to the command line to send the gRPC requests
    // TO DO: replace grpc call for dispere_blob and get_blob_status with tonic

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

    // TO DO: replace functionality for get_block_num and get_batch_confirmed_event using ethers-rs

    fn get_block_num(&self) -> Result<EthereumResponse, Error> {
        let request_data = json!({
            "jsonrpc": "2.0",
            "method": "eth_blockNumber",
            "id": 1
        });
        let response = self._send_request(request_data)?;
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
        let response = self._send_request(request_data)?;
        response.json::<GetLogsResponse>()
    }

    fn _send_request(
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

    async fn bridge_to_starknet(
        &self,
        tx_hash: &String,
    ) -> Result<()> {
        abigen!(
            STARKNET,
            r#"[
                function sendMessageToL2(uint256 toAddress, uint256 selector, uint256[] calldata payload) external payable returns (bytes32, uint256)
            ]"#,
        );
        let addr = DEFAULT_STARKNET_CORE_CONTRACT.parse::<Address>()?; // need to do conversion inside a function in case of an error
        let core_contract = STARKNET::new(addr, self.signer.clone());
        let fmt_tx = core_contract.send_message_to_l2(
            // !!!! placeholders for now, in the future we should use the state transition verifier contract on starknet
            U256::from(1),
            U256::from(1),
            // payload = parameters on l2
            vec![U256::from(1)], // tx_hash
        );
        fmt_tx
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("ethereum send update err: {e}"))?
            .await
            .map_err(|e| anyhow::anyhow!("ethereum poll update err: {e}"))?;
        Ok(())
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
    type Error = String;
    fn try_from(conf: config::EigenDaConfig) -> Result<Self, Self::Error> {
        let eth_rpc_provider = conf.eth_rpc_provider.clone();
        let provider = Provider::<Http>::try_from(eth_rpc_provider).map_err(|e| format!("ethereum error: {e}"))?;
        let wallet: LocalWallet = conf
            .sequencer_key
            .parse::<LocalWallet>()
            .map_err(|e| format!("ethereum error: {e}"))?
            .with_chain_id(conf.chain_id);
        let signer = Arc::new(SignerMiddleware::new(provider.clone(), wallet));
        Ok(Self{ config: conf.clone(), signer: signer })
    }
}    


mod test {
    // test #1

}