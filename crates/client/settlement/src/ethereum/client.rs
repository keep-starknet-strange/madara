use std::sync::Arc;
use std::time::Duration;

use ethers::prelude::{abigen, SignerMiddleware};
use ethers::providers::{Http, Provider};
use ethers::signers::{LocalWallet, Signer};
use ethers::types::{Address, TransactionReceipt, I256, U256};
pub use mc_data_availability::ethereum::config::EthereumConfig;

use crate::ethereum::errors::{Error, Result};

// Starknet core contract is responsible for advancing the rollup state and l1<>l2 messaging.
// Check out https://l2beat.com/scaling/projects/starknet#contracts to get a big picture.
//
// In this scope we work with a subset of methods responsible for querying and updating chain state.
// Starknet state is basically block number + state root hash.
// In order to update the state we need to provide the output of the Starknet OS program, consisting
// of:
//      1. Main part: previous/next state root, block number/hash, config hash, list of l1<>l2
//         messages
//      2. Data availability part: hash and size of the DA blob (the actual data is submitted
//         onchain separately)
//
// NOTE that currently we are using a "validium" version of the core contract which does not
// require the DA part.
//
// Starknet OS program is a Cairo program run by the SHARP to prove Starknet state transition.
// SNOS program hash is registered on the Starknet core contract to lock the version:
//      * SNOS program sources: https://github.com/starkware-libs/cairo-lang/tree/27a157d761ae49b242026bcbe5fca6e60c1e98bd/src/starkware/starknet/core/os
//      * You can calculate program hash by running: cairo-hash-program --program
//        build/os_latest.json
//
// SNOS config consists of:
//      1. Config version
//      2. Starknet chain ID
//      3. Fee token address
//
// Read this great overview to learn more about SNOS:
// https://hackmd.io/@pragma/ByP-iux1T
abigen!(
    StarknetContract,
    r#"[
        function programHash() public view returns (uint256)
        function stateBlockNumber() external view returns (int256)
        function stateRoot() external view returns (uint256)
        function configHash() public view returns (uint256)
        function updateState(uint256[] calldata programOutput) external
    ]"#,
);

pub type LocalMiddleware = SignerMiddleware<Provider<Http>, LocalWallet>;

pub struct StarknetContractClient {
    contract: StarknetContract<LocalMiddleware>,
}

impl StarknetContractClient {
    pub fn new(address: Address, client: Arc<LocalMiddleware>) -> Self {
        Self { contract: StarknetContract::new(address, client) }
    }

    pub async fn state_block_number(&self) -> Result<I256> {
        self.contract.state_block_number().call().await.map_err(Into::into)
    }

    pub async fn state_root(&self) -> Result<U256> {
        self.contract.state_root().call().await.map_err(Into::into)
    }

    pub async fn config_hash(&self) -> Result<U256> {
        self.contract.config_hash().call().await.map_err(Into::into)
    }

    pub async fn program_hash(&self) -> Result<U256> {
        self.contract.program_hash().call().await.map_err(Into::into)
    }

    pub async fn update_state(&self, program_output: Vec<U256>) -> Result<TransactionReceipt> {
        self.contract
            .update_state(program_output)
            .send()
            .await?
            .inspect(|s| log::debug!("[ethereum client] pending update_state transaction: {:?}", **s))
            .await?
            .ok_or_else(|| Error::MissingTransactionRecepit)
    }
}

impl TryFrom<EthereumConfig> for StarknetContractClient {
    type Error = Error;

    fn try_from(config: EthereumConfig) -> Result<Self> {
        let mut provider = Provider::<Http>::try_from(config.http_provider)?;

        if let Some(poll_interval_ms) = config.poll_interval_ms {
            provider = provider.interval(Duration::from_millis(poll_interval_ms));
        }

        let wallet: LocalWallet = config.sequencer_key.parse::<LocalWallet>()?.with_chain_id(config.chain_id);
        let signer = Arc::new(SignerMiddleware::new(provider, wallet));
        let address: Address = config.core_contracts.parse()?;

        Ok(Self::new(address, signer))
    }
}
