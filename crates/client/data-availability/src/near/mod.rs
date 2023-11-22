use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::RwLock;
use url::Url;

use crate::{DaClient, DaMode};

pub mod config;
use config::NearConfig;

#[derive(Clone, Debug)]
pub struct NearClient {
    client: Arc<reqwest::Client>,
    last_published_txid: Arc<RwLock<Option<String>>>,
    da_server_address: Url,
    mode: DaMode,
}

impl NearClient {
    pub fn new_blocking(config: NearConfig) -> Result<Self> {
        let blocking_client = reqwest::blocking::Client::new();

        let da_server_address = Url::parse(&config.da_server_address)
            .map_err(|e| anyhow::anyhow!("error parsing NEAR DA server address: {e}"))?;

        let da_server_is_alive = blocking_client.get(da_server_address.join("/health")?).send();

        let ok = da_server_is_alive.and_then(|r| r.text()).is_ok();

        if !ok {
            return Err(anyhow::anyhow!("Could not access NEAR DA server at {da_server_address}"));
        }

        let res = blocking_client.put(da_server_address.join("/configure")?).json(&config.da_server_config).send();

        if res.is_err() {
            log::warn!("A configuration for the NEAR DA server was provided, but the server was already configured");
        }

        Ok(Self {
            client: Arc::new(reqwest::Client::new()),
            da_server_address,
            last_published_txid: Arc::new(RwLock::new(None)),
            mode: config.mode,
        })
    }
}

#[async_trait]
impl DaClient for NearClient {
    fn get_mode(&self) -> DaMode {
        self.mode
    }

    async fn publish_state_diff(&self, state_diff: bytes::Bytes) -> Result<()> {
        // setter

        // the NEAR DA server handles most of the heavy lifting for us

        let res = self
            .client
            .post(self.da_server_address.join("/blob")?)
            .json(&near_da_http_api_data::SubmitRequest { data: state_diff.to_vec() })
            .send()
            .await?
            .text()
            .await?;

        *self.last_published_txid.write().await = Some(res);

        Ok(())
    }

    async fn last_published_state(&self) -> Result<bytes::Bytes> {
        // getter

        // just remember the last published state from the last publish_state_diff call and return that,
        // otherwise noop

        if let Some(transaction_id) = self.last_published_txid.read().await.as_ref() {
            let blob = self
                .client
                .get(self.da_server_address.join("/blob")?)
                .query(&near_da_http_api_data::BlobRequest { transaction_id: transaction_id.clone() })
                .send()
                .await?
                .json::<near_da_http_api_data::Blob>()
                .await?;

            Ok(bytes::Bytes::from(blob.data))
        } else {
            // There is no last-published state
            Ok(bytes::Bytes::new())
        }
    }
}
