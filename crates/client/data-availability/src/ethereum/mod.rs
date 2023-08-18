pub mod config;

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use ethers::prelude::{abigen, SignerMiddleware};
use ethers::providers::{Http, Middleware, Provider};
use ethers::signers::{LocalWallet, Signer};
use ethers::types::{Address, I256, U256};

use crate::{DaClient, DaMode};

#[derive(Clone)]
pub struct EthereumClient {
    http_provider: Provider<Http>,
    wallet: LocalWallet,
    cc_address: Address,
    mode: DaMode,
}

#[async_trait]
impl DaClient for EthereumClient {
    async fn publish_state_diff(&self, state_diff: Vec<U256>) -> Result<()> {
        let bal = self.http_provider.get_balance(self.wallet.address(), None).await.unwrap();
        println!("BALANCE: {:?} {:?}", state_diff, bal);

        abigen!(
            STARKNET,
            r#"[
                function updateState(uint256[] calldata programOutput, uint256 onchainDataHash, uint256 onchainDataSize) external
            ]"#,
        );
        let signer = Arc::new(SignerMiddleware::new(self.http_provider.clone(), self.wallet.clone()));
        let _core_contracts = STARKNET::new(self.cc_address, signer);

        // let tx = contract.update_state(state_diff, U256::default(), U256::default());
        // let pending_tx = tx.send().await.unwrap();
        // let minted_tx = pending_tx.await.unwrap();
        // log::info!("State Update: {:?}", minted_tx);
        Ok(())
    }

    async fn last_state(&self) -> Result<I256> {
        abigen!(
            STARKNET,
            r#"[
                function stateBlockNumber() external view returns (int256)
            ]"#,
        );

        let contract = STARKNET::new(self.cc_address, self.http_provider.clone().into());
        contract.state_block_number().call().await.map_err(|e| anyhow::anyhow!("ethereum contract err: {e}"))
    }

    fn get_mode(&self) -> DaMode {
        self.mode
    }
}

impl EthereumClient {
    pub fn new(conf: config::EthereumConfig) -> Self {
        let provider = Provider::<Http>::try_from(conf.http_provider).unwrap();

        let wallet: LocalWallet = conf.sequencer_key.parse::<LocalWallet>().unwrap().with_chain_id(conf.chain_id);

        let cc_address: Address = conf.core_contracts.parse().unwrap();

        Self { http_provider: provider, wallet, cc_address, mode: conf.mode }
    }
}
