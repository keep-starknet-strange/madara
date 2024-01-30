use std::sync::Arc;
use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use near_da_primitives::Blob;
use near_da_rpc::near::config::KeyType;
use near_da_rpc::near::Client;
use near_da_rpc::DataAvailability;
use tokio::sync::RwLock;

use crate::{DaClient, DaMode};

pub mod config;
use config::NearConfig;

#[derive(Clone)]
pub struct NearClient {
    client: Arc<Client>,
    last_published_txid: Arc<RwLock<Option<String>>>,
    mode: DaMode,
}

impl NearClient {
    pub fn new_blocking(config: NearConfig) -> Result<Self> {
        let client_config = near_da_rpc::near::config::Config {
            key: KeyType::SecretKey(config.account_id, config.secret_key),
            contract: config.contract_id,
            network: config.network,
            namespace: config.namespace,
        };

        Ok(Self {
            client: Arc::new(Client::new(&client_config)),
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

        let res = self
            .client
            .submit(&[Blob::new_v0(self.client.config.namespace, state_diff.to_vec())])
            .await
            .map_err(|e| anyhow::anyhow!("failed to submit blobs: {e}"))?;

        *self.last_published_txid.write().await = Some(res.0);

        Ok(())
    }

    async fn last_published_state(&self) -> Result<bytes::Bytes> {
        // getter

        if let Some(transaction_id) = self.last_published_txid.read().await.as_ref() {
            let blob = self
                .client
                .get(transaction_id.parse().map_err(|e| anyhow::anyhow!("failed to parse txid: {e}"))?)
                .await
                .map_err(|e| anyhow::anyhow!("failed to get blob: {e}"))?
                .0;

            Ok(bytes::Bytes::from(blob.data))
        } else {
            // There is no known last-published state
            Ok(bytes::Bytes::new())
        }
    }

    fn get_da_metric_labels(&self) -> HashMap<String, String> {
        HashMap::from([("name".to_string(), "near".to_string())])
    }
}
