use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use ethers::abi::Tokenize;
use ethers::contract::{ContractFactory, ContractInstance};
use ethers::core::utils::Anvil;
use ethers::prelude::SignerMiddleware;
use ethers::providers::{Http, Provider};
use ethers::signers::{LocalWallet, Signer};
use ethers::types::Address;
use ethers::utils::AnvilInstance;
use ethers_solc::artifacts::contract::ContractBytecode;

type AnvilClient = SignerMiddleware<Provider<Http>, LocalWallet>;

pub struct EthereumSandbox {
    _anvil: AnvilInstance,
    client: Arc<AnvilClient>,
}

impl EthereumSandbox {
    pub fn new() -> Self {
        let anvil_path: PathBuf = env::var("ANVIL_PATH")
            .map(Into::into)
            .unwrap_or_else(|_| home::home_dir().unwrap().join(".foundry/bin/anvil"));
        let anvil = Anvil::at(anvil_path).spawn();

        let provider = Provider::<Http>::try_from(anvil.endpoint())
            .expect("Failed to connect to Anvil")
            .interval(Duration::from_millis(10u64));

        let wallet: LocalWallet = anvil.keys()[0].clone().into();
        let client = SignerMiddleware::new(provider.clone(), wallet.with_chain_id(anvil.chain_id()));

        Self { _anvil: anvil, client: Arc::new(client) }
    }

    pub fn client(&self) -> Arc<AnvilClient> {
        self.client.clone()
    }

    pub fn address(&self) -> Address {
        self.client.address()
    }

    pub async fn deploy<T: Tokenize>(
        &self,
        build_artifacts: &str,
        contructor_args: T,
    ) -> ContractInstance<Arc<AnvilClient>, AnvilClient> {
        let artifacts: ContractBytecode =
            serde_json::from_str(build_artifacts).expect("Failed to parse build artifacts");
        let abi = artifacts.abi.unwrap();
        let bytecode = artifacts.bytecode.unwrap().object.into_bytes().unwrap();

        let factory = ContractFactory::new(abi, bytecode, self.client.clone());

        factory
            .deploy(contructor_args)
            .expect("Failed to deploy contract")
            .send()
            .await
            .expect("Ethereum polling error")
    }
}
