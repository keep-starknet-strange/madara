pub mod config;

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use ethers::prelude::{abigen, SignerMiddleware};
use ethers::providers::{Http, Provider};
use ethers::signers::{LocalWallet, Signer};
use ethers::types::{Address, U256};

use crate::utils::{is_valid_http_endpoint, try_bytes_to_vec_u256, u256_to_bytes};
use crate::{DaClient, DaMode};

#[derive(Clone, Debug)]
pub struct EthereumClient {
    http_provider: Provider<Http>,
    signer: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
    cc_address: Address,
    mode: DaMode,
}

#[async_trait]
impl DaClient for EthereumClient {
    async fn publish_state_diff(&self, state_diff: bytes::Bytes) -> Result<()> {
        log::debug!("State Update: {:?}", state_diff);
        let state_diff = try_bytes_to_vec_u256(state_diff)?;

        let fmt_tx = match self.mode {
            DaMode::Sovereign => {
                abigen!(
                    STARKNET,
                    r#"[
                        function updateState(uint256[] calldata programOutput) external
                    ]"#,
                );

                let core_contracts = STARKNET::new(self.cc_address, self.signer.clone());
                core_contracts.update_state(state_diff)
            }
            _ => {
                abigen!(
                    STARKNET,
                    r#"[
                        function updateState(uint256[] calldata programOutput, uint256 onchainDataHash, uint256 onchainDataSize) external
                    ]"#,
                );

                let core_contracts = STARKNET::new(self.cc_address, self.signer.clone());
                core_contracts.update_state(state_diff, U256::default(), U256::default())
            }
        };

        let tx = fmt_tx
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("ethereum send update err: {e}"))?
            .await
            .map_err(|e| anyhow::anyhow!("ethereum poll update err: {e}"))?;

        log::debug!("State Update: {:?}", tx);
        Ok(())
    }

    async fn last_published_state(&self) -> Result<bytes::Bytes> {
        abigen!(
            STARKNET,
            r#"[
                function stateBlockNumber() external view returns (int256)
            ]"#,
        );

        let contract = STARKNET::new(self.cc_address, self.http_provider.clone().into());
        contract
            .state_block_number()
            .call()
            .await
            .map(|i| u256_to_bytes(i.into_raw()))
            .map_err(|e| anyhow::anyhow!("ethereum contract err: {e}"))
    }

    fn get_mode(&self) -> DaMode {
        self.mode
    }

    fn get_da_metric_labels(&self) -> HashMap<String, String> {
        [("name".into(), "ethereum".into())].iter().cloned().collect()
    }
}

impl TryFrom<config::EthereumConfig> for EthereumClient {
    type Error = String;

    fn try_from(conf: config::EthereumConfig) -> Result<Self, Self::Error> {
        if !is_valid_http_endpoint(&conf.http_provider) {
            return Err(format!("invalid http endpoint, received {}", &conf.http_provider));
        }

        let provider = Provider::<Http>::try_from(conf.http_provider).map_err(|e| format!("ethereum error: {e}"))?;

        let wallet: LocalWallet = conf
            .sequencer_key
            .parse::<LocalWallet>()
            .map_err(|e| format!("ethereum error: {e}"))?
            .with_chain_id(conf.chain_id);

        let signer = Arc::new(SignerMiddleware::new(provider.clone(), wallet));

        let cc_address: Address = conf.core_contracts.parse().map_err(|e| format!("ethereum error: {e}"))?;

        Ok(Self { http_provider: provider, signer, cc_address, mode: conf.mode })
    }
}
