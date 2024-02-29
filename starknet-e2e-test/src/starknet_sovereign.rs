use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;

use ethers::providers::Middleware;
use ethers::types::{Address, I256, U256};
use ethers::utils::keccak256;
use mc_eth_client::config::{
    EthereumClientConfig, EthereumProviderConfig, EthereumWalletConfig, HttpProviderConfig, LocalWalletConfig,
    StarknetContracts,
};
use mc_settlement::ethereum::convert_felt_to_u256;
use mp_felt::Felt252Wrapper;
use mp_messages::{MessageL1ToL2, MessageL2ToL1};
use mp_snos_output::SnosCodec;
use starknet_api::hash::StarkFelt;
use starknet_api::serde_utils::hex_str_from_bytes;
use starknet_core_contract_client::clients::StarknetSovereignContractClient;
use starknet_core_contract_client::interfaces::{
    CoreContractInitData, CoreContractState, OperatorTrait, ProxyInitializeData, ProxySupportTrait,
    StarknetMessagingTrait,
};
use starknet_core_contract_client::{LocalWalletSignerMiddleware, StarknetCoreContractClient};
use starknet_ff::FieldElement;
use zaun_sandbox::unsafe_proxy::deploy_starknet_sovereign_behind_unsafe_proxy;
use zaun_sandbox::EthereumSandbox;

pub struct StarknetSovereign {
    _sandbox: EthereumSandbox,
    client: StarknetSovereignContractClient,
}

impl StarknetSovereign {
    pub fn address(&self) -> Address {
        self.client.address()
    }

    pub fn client(&self) -> Arc<LocalWalletSignerMiddleware> {
        self.client.client()
    }

    /// Attach to an existing Anvil instance or spawn a new one
    /// and then deploy:
    ///     - Starknet core contract (sovereign mode)
    ///     - Unsafe delegate proxy (no access restrictions)
    /// All the following interactions will be made thorugh the proxy
    pub async fn deploy() -> Self {
        // Try to attach to an already running sandbox (GitHub CI case)
        // otherwise spawn new sandbox instance
        let sandbox = if let Ok(endpoint) = std::env::var("ANVIL_ENDPOINT") {
            EthereumSandbox::attach(Some(endpoint)).expect("Failed to attach to sandbox")
        } else {
            EthereumSandbox::spawn(None)
        };

        let client = deploy_starknet_sovereign_behind_unsafe_proxy(sandbox.client())
            .await
            .expect("Failed to deploy starknet contract");

        Self { _sandbox: sandbox, client }
    }

    /// Write Ethereum settlement config to the specified Madara data folder
    /// (usually <base-path>/chains/<chain-id>/). The settlement config contains:
    ///     - Anvil endpoint and chain ID
    ///     - Delegate proxy contract address
    ///     - Sequencer private key (Anvil defaults)
    ///     - Transaction poll interval (reduced for testing purposes)
    ///
    /// Returns path to the settlement config (has to be passed as a Madara node argument)
    pub async fn create_settlement_conf(&self, data_path: PathBuf) -> PathBuf {
        let settlement_conf = EthereumClientConfig {
            provider: EthereumProviderConfig::Http(HttpProviderConfig {
                rpc_endpoint: self.client.client().provider().url().to_string(),
                tx_poll_interval_ms: Some(10u64), // Default is 7s, we need to speed things up
            }),
            wallet: Some(EthereumWalletConfig::Local(LocalWalletConfig {
                chain_id: self.client.client().get_chainid().await.expect("Failed to get sandbox chain ID").as_u64(),
                ..Default::default() // Use Anvil default account
            })),
            contracts: StarknetContracts {
                core_contract: hex_str_from_bytes::<20, true>(self.client.address().0),
                ..Default::default()
            },
        };

        let conf_path = data_path.join("eth-config.json");
        let conf_file = File::create(&conf_path).expect("Failed to open file for writing");
        serde_json::to_writer(conf_file, &settlement_conf).expect("Failed to write settlement config");

        conf_path
    }

    /// Initialize Starknet core contract with the specified data.
    ///
    /// Also register Anvil default account as an operator.
    pub async fn initialize_with(&self, init_data: CoreContractInitData) {
        let data = ProxyInitializeData::<0> { sub_contract_addresses: [], eic_address: Default::default(), init_data };

        self.client.initialize_with(data).await.expect("Failed to initialize");

        self.client.register_operator(self.client.client().address()).await.expect("Failed to register operator");
    }

    /// Initialize Starknet core contract with the specified program and config hashes.
    /// The rest of parameters will be left default.
    ///
    /// Also register Anvil default account as an operator.
    pub async fn initialize(&self, program_hash: StarkFelt, config_hash: StarkFelt) {
        self.initialize_with(CoreContractInitData {
            program_hash: convert_felt_to_u256(program_hash),
            config_hash: convert_felt_to_u256(config_hash),
            ..Default::default()
        })
        .await;
    }

    /// Initialize Starknet core contract with the specified block number and state root hash.
    /// The program and config hashes will be set according to the Madara Goerli configuration.
    ///
    /// Also register Anvil default account as an operator.
    pub async fn initialize_for_goerli(&self, block_number: StarkFelt, state_root: StarkFelt) {
        // See SN_OS_PROGRAM_HASH constant
        let program_hash = StarkFelt::from(Felt252Wrapper::from(
            FieldElement::from_hex_be("0x41fc2a467ef8649580631912517edcab7674173f1dbfa2e9b64fbcd82bc4d79").unwrap(),
        ));

        // Hash version:        SN_OS_CONFIG_HASH_VERSION (settlement)
        // Chain ID:            SN_GOERLI_CHAIN_ID (pallet config)
        // Fee token address:   0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7 (genesis
        // config)
        let config_hash = StarkFelt::from(Felt252Wrapper::from(
            FieldElement::from_hex_be("0x036f5e4ea4dd042801c8841e3db8e654124305da0f11824fc1db60c405dbb39f").unwrap(),
        ));

        let init_data = CoreContractInitData {
            program_hash: convert_felt_to_u256(program_hash), // zero program hash would be deemed invalid
            config_hash: convert_felt_to_u256(config_hash),
            initial_state: CoreContractState {
                block_number: I256::from_raw(convert_felt_to_u256(block_number)),
                state_root: convert_felt_to_u256(state_root),
                ..Default::default()
            },
            ..Default::default()
        };

        self.initialize_with(init_data).await;
    }

    pub async fn send_message_to_l2(&self, message: &MessageL1ToL2) {
        self.client
            .send_message_to_l2(
                convert_felt_to_u256(message.to_address.0.0),
                convert_felt_to_u256(message.selector),
                message.payload.clone().into_iter().map(convert_felt_to_u256).collect(),
                1.into(),
            )
            .await
            .expect("Failed to call `send_message_to_l2`");
    }

    pub async fn message_to_l1_exists(&self, message: &MessageL2ToL1) -> bool {
        let mut payload: Vec<u8> = Vec::new();
        message.clone().into_encoded_vec().into_iter().for_each(|felt| payload.append(&mut felt.bytes().to_vec()));

        let msg_hash = keccak256(payload);
        let res = self.client.l2_to_l1_messages(msg_hash).await.expect("Failed to call `l2_to_l1_messages`");

        res != U256::zero()
    }

    pub async fn message_to_l2_exists(&self, message: &MessageL1ToL2) -> bool {
        let mut payload: Vec<u8> = Vec::new();
        message.clone().into_encoded_vec().into_iter().for_each(|felt| payload.append(&mut felt.bytes().to_vec()));

        let msg_hash = keccak256(payload);
        let res = self.client.l1_to_l2_messages(msg_hash).await.expect("Failed to call `l2_to_l1_messages`");

        res != U256::zero()
    }
}
