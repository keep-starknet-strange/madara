use std::collections::HashMap;
use std::sync::Arc;

use ethers::types::U256;
mod utils;
use anyhow::Result;
use async_trait::async_trait;
use celestia_rpc::client::{new_http, new_websocket};
use celestia_rpc::{BlobClient, HeaderClient};
use celestia_types::nmt::Namespace;
use celestia_types::Blob;
use jsonrpsee::http_client::HttpClient;
use jsonrpsee::ws_client::WsClient;
use utils::string_to_namespace;

use crate::da::{DAArgs, DAInput, DAOutput, DataAvailability};
use crate::utils::{get_bytes_from_state_diff, is_valid_http_endpoint, is_valid_ws_endpoint};

#[derive(Debug, Clone)]
pub struct CelestiaClient {
    http_client: HttpClient,
    ws_client: Arc<WsClient>,
    nid: Namespace,
}

#[async_trait]
impl DataAvailability for CelestiaClient {
    fn new(da_config: &HashMap<String, String>) -> Result<Self> {
        if let DAArgs::Celestia { http_endpoint, ws_endpoint, auth_token, namespace } = Self::validate_args(da_config)?
        {
            let http_client = new_http(&http_endpoint, Some(&auth_token))?;

            // https://greptime.com/blogs/2023-03-09-bridging-async-and-sync-rust
            let ws_client = futures::executor::block_on(async { new_websocket(&ws_endpoint, Some(&auth_token)).await })
                .map_err(|e| anyhow::anyhow!("Could not initialize ws endpoint {e}"))?;

            let nid = string_to_namespace(&namespace)?;

            Ok(CelestiaClient { http_client, ws_client: Arc::new(ws_client), nid })
        } else {
            Err(anyhow::anyhow!("Invalid parameters"))
        }
    }

    fn validate_args(da_config: &HashMap<String, String>) -> Result<DAArgs> {
        if da_config.len() != 5 {
            return Err(anyhow::anyhow!("Expected 5 arguments for Celestia but received {}", da_config.len()));
        }

        let http_endpoint =
            da_config.get("http_endpoint").ok_or_else(|| anyhow::anyhow!("Missing 'http_endpoint'"))?.clone();
        let ws_endpoint = da_config.get("ws_endpoint").ok_or_else(|| anyhow::anyhow!("Missing 'ws_endpoint'"))?.clone();
        let auth_token = da_config.get("auth_token").ok_or_else(|| anyhow::anyhow!("Missing 'auth_token'"))?.clone();
        let namespace = da_config.get("namespace").ok_or_else(|| anyhow::anyhow!("Missing 'namespace'"))?.clone();

        if !is_valid_http_endpoint(&http_endpoint) {
            return Err(anyhow::anyhow!("Invalid http endpoint, received {}", http_endpoint));
        }
        if !is_valid_ws_endpoint(&ws_endpoint) {
            return Err(anyhow::anyhow!("Invalid ws endpoint, received {}", ws_endpoint));
        }

        Ok(DAArgs::Celestia { http_endpoint, ws_endpoint, auth_token, namespace })
    }

    fn format_state_diff(&self, state_diff: &[U256]) -> Result<DAInput> {
        let blob = Blob::new(self.nid, get_bytes_from_state_diff(state_diff))?;
        Ok(DAInput::Celestia(blob))
    }

    async fn publish_data(&self, data: &DAInput) -> Result<DAOutput> {
        if let DAInput::Celestia(blob) = data {
            let submitted_height = self.http_client.blob_submit(&[blob.clone()]).await?;

            // blocking call, awaiting on server side (Celestia Node) that a block with our data is included
            // not clean split between ws and http endpoints, which is why this call is blocking in the first
            // place...
            self.ws_client.header_wait_for_height(submitted_height).await?;

            Ok(DAOutput::Celestia { submitted_height })
        } else {
            Err(anyhow::anyhow!("Invalid input data"))
        }
    }

    async fn verify_inclusion(&self, da_input: &DAInput, da_output: &DAOutput) -> Result<()> {
        match (da_input, da_output) {
            (DAInput::Celestia(blob), DAOutput::Celestia { submitted_height }) => {
                let received_blob = self.http_client.blob_get(*submitted_height, self.nid, blob.commitment).await?;
                received_blob.validate()?;
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Invalid input or output")),
        }
    }
}

impl CelestiaClient {
    #[cfg(test)]
    pub async fn new_test() -> Result<Self> {
        use std::env;

        let mut path = env::current_dir().unwrap(); // get current directory
        path.push(".env"); // add .env to the path
        dotenvy::from_path(path).ok();
        let auth_token = env::var("AUTH_TOKEN").expect("AUTH_TOKEN must be set"); // Get the auth_token

        let mut celestia_map: HashMap<String, String> = HashMap::new();
        celestia_map.insert("da_type".to_string(), "Celestia".to_string());
        celestia_map.insert("ws_endpoint".to_string(), "ws://127.0.0.1:26658".to_string());
        celestia_map.insert("http_endpoint".to_string(), "http://127.0.0.1:26658".to_string());
        celestia_map.insert("auth_token".to_string(), auth_token); // Assuming this variable exists elsewhere in your code.
        celestia_map.insert("namespace".to_string(), "Madara".to_string());

        if let DAArgs::Celestia { http_endpoint, ws_endpoint, auth_token, namespace } =
            Self::validate_args(&celestia_map)?
        {
            let http_client = new_http(&http_endpoint, Some(&auth_token)).unwrap();
            let ws_client = new_websocket(&ws_endpoint, Some(&auth_token)).await.unwrap();
            let nid = string_to_namespace(&namespace).unwrap();

            Ok(CelestiaClient { http_client, ws_client: Arc::new(ws_client), nid })
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
        let da_client = CelestiaClient::new_test().await.unwrap();
        let state_diff = vec![ethers::types::U256::from(0)];

        let da_input = da_client.format_state_diff(&state_diff).unwrap();
        let da_output = da_client.publish_data(&da_input).await.unwrap();

        match (da_input, da_output) {
            (DAInput::Celestia(blob), DAOutput::Celestia { submitted_height }) => {
                println!("Submitted height deterministic: {:?}, now waiting for block conf", submitted_height);

                let block_conf = da_client.http_client.header_wait_for_height(submitted_height).await?;

                println!("block_conf: {:?}", block_conf.header);

                let dah = da_client.http_client.header_get_by_height(submitted_height).await.unwrap().dah;
                let root_hash = dah.row_root(0).unwrap();

                println!("root_hash: {:?}", root_hash);

                let received_blob =
                    da_client.http_client.blob_get(submitted_height, da_client.nid, blob.commitment).await.unwrap();

                received_blob.validate().unwrap();
                assert_eq!(received_blob, blob);

                Ok(())
            }
            _ => Err(anyhow::anyhow!("Invalid input or output")),
        }
    }
}
