use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use ethers::types::{I256, U256};

use crate::{DaClient, DaMode};

pub mod config;
use config::NearConfig;

#[derive(Clone)]
pub struct NearClient {
    client: Arc<near_jsonrpc_client::JsonRpcClient>,
    signer: near_crypto::InMemorySigner,
    contract_account_id: near_primitives::account::id::AccountId,
    mode: DaMode,
}

impl std::fmt::Debug for NearClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NearClient")
            .field("client", &self.client)
            .field("signer.account_id", &self.signer.account_id)
            .field("signer.public_key", &self.signer.public_key)
            .field("mode", &self.mode)
            .finish()
    }
}

impl TryFrom<NearConfig> for NearClient {
    type Error = String;

    fn try_from(config: NearConfig) -> Result<Self, Self::Error> {
        let client = near_jsonrpc_client::JsonRpcClient::connect(&config.rpc_address);

        let sequencer_account_id: near_primitives::types::AccountId =
            config.sequencer_account_id.parse().map_err(|e| format!("Invalid sequencer account ID: {e}"))?;

        let sequencer_key: near_crypto::SecretKey =
            config.sequencer_key.parse().map_err(|e| format!("Invalid sequencer key: {e}"))?;

        let signer = near_crypto::InMemorySigner::from_secret_key(sequencer_account_id, sequencer_key);

        let contract_account_id =
            config.contract_account_id.parse().map_err(|e| format!("Invalid contract account ID: {e}"))?;

        Ok(Self { client: Arc::new(client), signer, mode: config.mode, contract_account_id })
    }
}

#[async_trait]
impl DaClient for NearClient {
    fn get_mode(&self) -> DaMode {
        self.mode
    }

    async fn last_published_state(&self) -> Result<I256> {
        todo!()
    }

    async fn publish_state_diff(&self, state_diff: Vec<U256>) -> Result<()> {
        todo!()
    }
}
