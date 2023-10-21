#![allow(deprecated)]

use std::path::PathBuf;

use log::info;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use reqwest::{StatusCode, Url};
use serde_json::{json, Value};
use starknet_gateway::sequencer::models::BlockId;
use starknet_gateway::SequencerGatewayProvider;
use tokio::time;

mod convert;

async fn create_block(rpc_port: u16) -> Result<StatusCode, reqwest::Error> {
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "application/json ".parse().unwrap());

    let url = format!("http://localhost:{}/", rpc_port);
    let payload = json!({
        "id": 1,
        "jsonrpc": "2.0",
        "method": "engine_createBlock",
        "params": [true, true, null]
    });

    let response = client.post(url).headers(headers.clone()).json(&payload).send().await?;

    Ok(response.status())
}

async fn get_last_synced_block(rpc_port: u16) -> Result<Option<u64>, reqwest::Error> {
    let client = reqwest::Client::new();
    let headers = {
        let mut h = HeaderMap::new();
        h.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        h
    };

    let url = format!("http://localhost:{}/", rpc_port);
    let payload = json!({
        "id": 1,
        "jsonrpc": "2.0",
        "method": "chain_getBlock",
        "params": []
    });

    let response = client.post(&url).headers(headers).json(&payload).send().await?;

    let body: Value = response.json().await?;

    body["result"]["block"]["header"]["number"]
        .as_str()
        .and_then(|number_hex| u64::from_str_radix(&number_hex[2..], 16).ok())
        .map_or(Ok(None), |number_decimal| Ok(Some(number_decimal)))
}

const DEFAULT_CONFIG_FILE: &str = "config/execution_config/default_config.json";

pub struct ExecutionConfig {
    pub config_file_name: PathBuf,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        ExecutionConfig { config_file_name: PathBuf::from(DEFAULT_CONFIG_FILE) }
    }
}

pub async fn fetch_block(sender: async_channel::Sender<mp_block::Block>, uri: &str, rpc_port: u16) {
    let gateway_url = Url::parse(&format!("{uri}/gateway")).unwrap();
    let feeder_gateway_url = Url::parse(&format!("{uri}/feeder_gateway", uri = uri)).unwrap();
    let chain_id = starknet_ff::FieldElement::ZERO;
    let client = SequencerGatewayProvider::new(gateway_url, feeder_gateway_url, chain_id);

    let mut i = get_last_synced_block(rpc_port).await.unwrap().unwrap() + 1;
    loop {
        match client.get_block(BlockId::Number(i)).await {
            Ok(block) => {
                let starknet_block = convert::block(&block);
                sender.send(starknet_block).await.unwrap();
                match create_block(rpc_port).await {
                    Ok(status) => {
                        if status.is_success() {
                            info!("[👽] Block #{} synced correctly", i);
                            i += 1;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error processing RPC call: {:?}", e);
                    }
                }
            }
            Err(error) => {
                eprintln!("Error retrieving block: {:?}", error);
                time::sleep(time::Duration::from_secs(2)).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use starknet_ff::FieldElement;

    // Mocking StarknetFeederGatewayClient for testing
    mockito::mock! {
        StarknetFeederGatewayClient {
            fn new(url: &str, option: Option<&str>, version: &str, retry_config: RetryConfig) -> Self;
            async fn block(&self, block_number: BlockNumber) -> Result<Option<starknet_client::reader::Block>, starknet_client::Error>;
        }
    }

    #[test]
    fn test_get_header() {
        // Provide a mock starknet_client::reader::Block and BoundedVec<Transaction,
        // MaxTransactions> Then, call get_header with the mock data and check if the
        // resulting Header is as expected.
    }

    #[test]
    fn test_get_txs() {
        // Provide a mock starknet_client::reader::Block
        // Call get_txs with the mock data and verify if the resulting BoundedVec<Transaction,
        // MaxTransactions> is correct.
    }

    #[tokio::test]
    async fn test_from_gateway_to_starknet_block() {
        // Provide a mock starknet_client::reader::Block
        // Call from_gateway_to_starknet_block and verify the resulting Block.
    }

    #[tokio::test]
    async fn test_create_block() {
        // This test can check if an RPC call is made properly.
        // It can also verify if the function handles different response statuses correctly.
    }

    #[tokio::test]
    async fn test_fetch_block_success() {
        // Mock the StarknetFeederGatewayClient to return a successful block.
        // Run fetch_block and ensure the block is correctly added to the queue and RPC call is
        // made.
    }

    #[tokio::test]
    async fn test_fetch_block_error() {
        // Mock the StarknetFeederGatewayClient to return an error.
        // Run fetch_block and ensure the error is handled correctly.
    }

    #[test]
    fn test_into_mont() {
        // Constructing FieldElement from the hex representation of "SN_MAIN"
        let sn_main_hex = "00000000000000000000000000000000534e5f4d41494e";
        let sn_main_bytes = hex::decode(sn_main_hex).expect("Failed to decode hex");
        let sn_main_u64_array = bytes_to_u64_array(&sn_main_bytes);

        let fe = FieldElement::from_mont(sn_main_u64_array);
        let mont_representation = fe.into_mont();

        println!("{:?}", mont_representation);
        // Optionally, add assertions to check the correctness of mont_representation
    }

    // Helper function to convert a byte slice into [u64; 4]
    fn bytes_to_u64_array(bytes: &[u8]) -> [u64; 4] {
        assert_eq!(bytes.len(), 32, "Expected a 32-byte slice");

        let mut array = [0u64; 4];
        for (i, chunk) in bytes.chunks(8).enumerate() {
            array[i] = u64::from_be_bytes(chunk.try_into().expect("Failed to convert bytes to u64"));
        }
        array
    }
}
