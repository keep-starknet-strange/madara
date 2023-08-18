use std::sync::Arc;

pub mod config;

use async_trait::async_trait;
use ethers::prelude::{abigen, SignerMiddleware};
use ethers::providers::{Http, Middleware, Provider};
use ethers::signers::{LocalWallet, Signer};
use ethers::types::{Address, I256, U256};

use crate::DaClient;

#[derive(Debug, Clone)]
pub struct EthereumClient {
    http_provider: Provider<Http>,
    mode: String,
    wallet: LocalWallet,
    cc_address: Address,
}

#[async_trait]
impl DaClient for EthereumClient {
    async fn publish_state_diff(&self, state_diff: Vec<U256>) -> Result<bool, String> {
        let bal = self.http_provider.get_balance(self.wallet.address(), None).await.unwrap();
        println!("BALANCE: {:?}", bal);

        abigen!(
            STARKNET,
            r#"[
                function updateState(uint256[] calldata programOutput, uint256 onchainDataHash, uint256 onchainDataSize) external
            ]"#,
        );
        let signer = Arc::new(SignerMiddleware::new(self.http_provider.clone(), self.wallet.clone()));
        let core_contracts = STARKNET::new(self.cc_address, signer);

        // let tx = contract.update_state(state_diff, U256::default(), U256::default());
        // let pending_tx = tx.send().await.unwrap();
        // let minted_tx = pending_tx.await.unwrap();
        // log::info!("State Update: {:?}", minted_tx);
        Ok(true)
    }

    fn get_mode(&self) -> String {
        self.mode.clone()
    }
}

impl EthereumClient {
    pub fn new(conf: config::EthereumConfig) -> Self {
        let provider = Provider::<Http>::try_from(conf.http_provider).unwrap();

        let wallet: LocalWallet = conf.sequencer_key.parse::<LocalWallet>().unwrap().with_chain_id(conf.chain_id);

        let address: Address = conf.core_contracts.parse().unwrap();

        Self { http_provider: provider, mode: conf.mode, wallet, cc_address: address }
    }
}

// pub async fn last_proven_block() -> Result<I256, String> {
//     abigen!(
//         STARKNET,
//         r#"[
//             function stateBlockNumber() external view returns (int256)
//         ]"#,
//     );

//     let provider = Provider::<Http>::try_from(STARKNET_NODE).unwrap();
//     let client = Arc::new(provider);

//     let address: Address = STARKNET_GOERLI_CC_ADDRESS.parse().unwrap();
//     let contract = STARKNET::new(address, client);
//     contract.state_block_number().call().await.map_err(|e| format!("ethereum contract err: {e}"))
// }
