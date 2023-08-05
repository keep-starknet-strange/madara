use std::sync::Arc;
use eyre::Result;

use ethers::types::U256;

mod network;
use network::Network;
mod utils;
use utils::string_to_namespace;

use celestia_types::Blob;
use celestia_types::nmt::Namespace;
use celestia_types::Result as CelestiaTypesResult;
use celestia_rpc::client::{new_http, new_websocket};
use celestia_rpc::{BlobClient, HeaderClient};
use celestia_rpc::Result as CelestiaRpcResult;
use jsonrpsee::http_client::HttpClient;
use jsonrpsee::ws_client::WsClient;
use jsonrpsee::core::Error as JsonRpSeeError;

const MADARA: &str = "Madara";

#[derive(Debug, Clone)]
pub struct CelestiaClient {
    http_client: HttpClient,
    ws_client: Arc<WsClient>,
    nid: Namespace
}

impl CelestiaClient {
    pub fn new(http_client: HttpClient, ws_client: WsClient) -> CelestiaRpcResult<Self> {
        Ok(CelestiaClient { 
            http_client,
            ws_client: Arc::new(ws_client),
            nid:  string_to_namespace(MADARA).unwrap()
        }
        )
    }

    pub async fn publish_state_diff_and_verify_inclusion(&self, state_diff: Vec<U256>) -> eyre::Result<()> {
        let blob = self.get_blob_from_state_diff(state_diff)?;
        let submitted_height = self.publish_data(&blob).await?;
        
        //blocking call, awaiting on server side (Celestia Node) that a block with our data is included
        //not clean split between ws and http endpoints, which is why this call is blocking in the first place...
        self.ws_client.header_wait_for_height(submitted_height).await?;

        self.verify_blob_was_included(submitted_height, blob).await?;

        Ok(())
    }

    async fn publish_data(&self, blob: &Blob) -> Result<u64, JsonRpSeeError> {
        self.http_client.blob_submit(&[blob.clone()]).await
    }

    fn get_blob_from_state_diff(&self, state_diff: Vec<U256>) -> CelestiaTypesResult<Blob> {
        
        let state_diff_bytes: Vec<u8> = state_diff.iter().flat_map(|item| {
            let mut bytes = [0_u8; 32];
            item.to_big_endian(&mut bytes);
            bytes.to_vec()
        }).collect();

        Blob::new(self.nid, state_diff_bytes)
    }

    async fn verify_blob_was_included(&self, submitted_height:u64, blob: Blob) -> eyre::Result<()> {
        let received_blob = self.http_client.blob_get(submitted_height, self.nid, blob.commitment).await?;
        received_blob.validate()?;
        Ok(())
    }
}

#[derive(Default)]
pub struct CelestiaClientBuilder {
    http_endpoint: Option<String>,
    ws_endpoint: Option<String>,
    auth_token: Option<String>,
}

impl CelestiaClientBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn http_endpoint(mut self, endpoint: Option<&str>) -> Self {
        self.http_endpoint = endpoint.map(|s| s.to_string());
        self
    }

    pub fn ws_endpoint(mut self, endpoint: Option<&str>) -> Self {
        self.ws_endpoint = endpoint.map(|s| s.to_string());
        self
    }

    pub fn auth_token(mut self, auth_token: Option<&str>) -> Self {
        self.auth_token = auth_token.map(|s| s.to_string());
        self
    }

    pub fn build(self) -> CelestiaRpcResult<CelestiaClient> {
        let base_config = Network::LOCAL.to_base_config();
        let http_endpoint = self.http_endpoint.unwrap_or_else(|| base_config.http_endpoint.unwrap());
        let ws_endpoint = self.ws_endpoint.unwrap_or_else(|| base_config.ws_endpoint.unwrap());
        let auth_token = self.auth_token.unwrap_or_else(|| base_config.auth_token.unwrap_or_default());
        
        let http_client = CelestiaClientBuilder::get_http_client(&http_endpoint, &auth_token)?;
        
        //https://greptime.com/blogs/2023-03-09-bridging-async-and-sync-rust
        let ws_client = futures::executor::block_on(async {
            CelestiaClientBuilder::get_ws_client(&ws_endpoint, &auth_token).await.unwrap()
        });

        CelestiaClient::new(http_client, ws_client)
    }

    #[cfg(test)]
    pub async fn build_test(self) -> CelestiaRpcResult<CelestiaClient> {
        let base_config = Network::LOCAL.to_base_config();
        let http_endpoint = self.http_endpoint.unwrap_or_else(|| base_config.http_endpoint.unwrap());
        let ws_endpoint = self.ws_endpoint.unwrap_or_else(|| base_config.ws_endpoint.unwrap());
        let auth_token = self.auth_token.unwrap_or_else(|| base_config.auth_token.unwrap_or_default());

        let http_client = CelestiaClientBuilder::get_http_client(&http_endpoint, &auth_token)?;
        let ws_client = CelestiaClientBuilder::get_ws_client(&ws_endpoint, &auth_token).await?;

        CelestiaClient::new(http_client, ws_client)
    }

    fn get_http_client(endpoint: &str, auth_token: &str) -> CelestiaRpcResult<HttpClient> {
        Ok(new_http(endpoint, Some(auth_token)).unwrap())
    }

    async fn get_ws_client(endpoint: &str, auth_token: &str) -> CelestiaRpcResult<WsClient> {
        Ok(new_websocket(endpoint, Some(auth_token)).await.unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_publish_data_and_verify_publication() -> Result<()> {
        let mut path = env::current_dir().unwrap(); // get current directory
        
        path.push(".env"); // add .env to the path

        dotenvy::from_path(path).ok();

        let auth_token = env::var("AUTH_TOKEN")
        .expect("AUTH_TOKEN must be set"); // Get the auth_token

        let client: CelestiaClient = CelestiaClientBuilder::new()
        .auth_token(Some(&auth_token))
        .build_test()
        .await?;

        let state_diff = vec![ethers::types::U256::from(0)];

        let blob = client.get_blob_from_state_diff(state_diff).unwrap();

        let submitted_height = client.publish_data(&blob).await.unwrap();

        println!("Submitted height deterministic: {:?}, now waiting for block conf", submitted_height);

        let block_conf = client.http_client.header_wait_for_height(submitted_height).await?;

        println!("block_conf: {:?}", block_conf.header);

        let dah = client
        .http_client
        .header_get_by_height(submitted_height)
        .await
        .unwrap()
        .dah;
        let root_hash = dah.row_root(0).unwrap();

        println!("root_hash: {:?}", root_hash);

        let received_blob = client
        .http_client
        .blob_get(submitted_height, client.nid, blob.commitment)
        .await
        .unwrap();

        received_blob.validate().unwrap();
        assert_eq!(received_blob, blob);

        Ok(())
    }
}