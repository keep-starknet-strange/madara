use std::sync::Arc;

use ethers::addressbook::Address;
use ethers::prelude::U256;
use ethers::types::Bytes;
use mp_felt::Felt252Wrapper;
use rand::Rng;
use starknet_accounts::ConnectedAccount;
use starknet_core::utils::get_selector_from_name;
use starknet_core_contract_client::clients::{
    DaiERC20ContractClient, StarkgateManagerContractClient, StarkgateRegistryContractClient,
    StarknetTokenBridgeContractClient,
};
use starknet_core_contract_client::interfaces::{
    DaiERC20TokenTrait, ProxySupportTrait, StarkgateManagerTrait, StarknetTokenBridgeTrait,
};
use starknet_core_contract_client::{LocalWalletSignerMiddleware, StarknetContractClient};
use starknet_ff::FieldElement;
use starknet_rpc_test::constants::{CAIRO_1_ACCOUNT_CONTRACT, SIGNER_PRIVATE};
use starknet_rpc_test::fixtures::ThreadSafeMadaraClient;
use starknet_rpc_test::utils::{build_single_owner_account, get_contract_address_from_deploy_tx, AccountActions};
use starknet_rpc_test::Transaction;
use zaun_sandbox::deploy::{
    deploy_dai_erc20_behind_unsafe_proxy, deploy_starkgate_manager_behind_unsafe_proxy,
    deploy_starkgate_registry_behind_unsafe_proxy, deploy_starknet_token_bridge_behind_unsafe_proxy,
};
use zaun_sandbox::EthereumSandbox;

use crate::utils::madara_contract_call;

pub struct StarknetTokenBridge {
    _sandbox: EthereumSandbox,
    manager: StarkgateManagerContractClient,
    registry: StarkgateRegistryContractClient,
    token_bridge: StarknetTokenBridgeContractClient,
    dai_erc20: DaiERC20ContractClient,
}

impl StarknetTokenBridge {
    pub fn manager_address(&self) -> Address {
        self.manager.address()
    }
    pub fn registry_address(&self) -> Address {
        self.registry.address()
    }
    pub fn bridge_address(&self) -> Address {
        self.token_bridge.address()
    }
    pub fn dai_address(&self) -> Address {
        self.dai_erc20.address()
    }

    pub fn manager_client(&self) -> Arc<LocalWalletSignerMiddleware> {
        self.manager.client()
    }
    pub fn registry_client(&self) -> Arc<LocalWalletSignerMiddleware> {
        self.registry.client()
    }
    pub fn bridge_client(&self) -> Arc<LocalWalletSignerMiddleware> {
        self.token_bridge.client()
    }
    pub fn dai_client(&self) -> Arc<LocalWalletSignerMiddleware> {
        self.dai_erc20.client()
    }

    /// Attach to an existing Anvil instance or spawn a new one
    /// and then deploy:
    ///     - Starknet core contract (sovereign mode)
    ///     - Unsafe delegate proxy (no access restrictions)
    /// All the following interactions will be made thorugh the proxy
    pub async fn deploy() -> Self {
        // Try to attach to an already running sandbox (GitHub CI case)
        // otherwise spawn new sandbox instance
        // let sandbox = if let Ok(endpoint) = std::env::var("ANVIL_ENDPOINT") {
        //     EthereumSandbox::attach(Some(endpoint)).expect("Failed to attach to sandbox")
        // } else {
        //     EthereumSandbox::spawn(None)
        // };
        let endpoint: String = String::from("http://localhost:8545");
        let sandbox = EthereumSandbox::attach(Some(endpoint)).expect("Failed to attach to sandbox");

        let manager = deploy_starkgate_manager_behind_unsafe_proxy(sandbox.client())
            .await
            .expect("Failed to deploy starkgate manager contract");
        let registry = deploy_starkgate_registry_behind_unsafe_proxy(sandbox.client())
            .await
            .expect("Failed to deploy starkgate registry");
        let token_bridge = deploy_starknet_token_bridge_behind_unsafe_proxy(sandbox.client())
            .await
            .expect("Failed to deploy starknet contract");
        let dai_erc20 =
            deploy_dai_erc20_behind_unsafe_proxy(sandbox.client()).await.expect("Failed to deploy dai erc20 contract");

        Self { _sandbox: sandbox, manager, registry, token_bridge, dai_erc20 }
    }

    pub async fn deploy_l2_contracts(madara: &ThreadSafeMadaraClient) -> FieldElement {
        let rpc = madara.get_starknet_client().await;
        let account = build_single_owner_account(&rpc, SIGNER_PRIVATE, CAIRO_1_ACCOUNT_CONTRACT, false);
        let mut madara_write_lock = madara.write().await;

        let (erc20_declare_tx, _, _) = account.declare_contract(
            "../starknet-e2e-test/contracts/erc20.sierra.json",
            "../starknet-e2e-test/contracts/erc20.casm.json",
        );

        let (bridge_declare_tx, _, _) = account.declare_contract(
            "../starknet-e2e-test/contracts/token_bridge.sierra.json",
            "../starknet-e2e-test/contracts/token_bridge.casm.json",
        );

        let nonce = account.get_nonce().await.unwrap();
        madara_write_lock
            .create_block_with_txs(vec![
                Transaction::Declaration(erc20_declare_tx.nonce(nonce)),
                Transaction::Declaration(bridge_declare_tx.nonce(nonce + FieldElement::ONE)),
            ])
            .await
            .expect("Failed to declare token bridge contract on l2");

        let mut rng = rand::thread_rng();
        let random: u32 = rng.gen();

        let deploy_tx = account.invoke_contract(
            FieldElement::from_hex_be("0x5").unwrap(),
            "deploy_contract",
            vec![
                FieldElement::from_hex_be("0x0469e62e2db14948db2579f7e0271759c9944937ac56b0a3698f386b22e49363")
                    .unwrap(), // class_hash
                FieldElement::from_dec_str(&*random.to_string()).unwrap(), // contract_address_salt
                FieldElement::ONE,                                         // constructor_calldata_len
                FieldElement::ZERO,                                        // constructor_calldata (upgrade_delay)
            ],
            None,
        );

        let mut txs = madara_write_lock.create_block_with_txs(vec![Transaction::Execution(deploy_tx)]).await.unwrap();

        let deploy_tx_result = txs.pop().unwrap();
        get_contract_address_from_deploy_tx(&rpc, deploy_tx_result).await.unwrap()
    }

    /// Initialize Starknet core contract with the specified data.
    ///
    /// Also register Anvil default account as an operator.
    pub async fn initialize(&self, messaging_contract: Address) {
        let empty_bytes = [0u8; 32];

        let mut manager_calldata = Vec::new();
        manager_calldata.extend(empty_bytes);
        manager_calldata.extend(pad_bytes(self.registry_address()));
        manager_calldata.extend(pad_bytes(self.bridge_address()));

        let mut registry_calldata = Vec::new();
        registry_calldata.extend(empty_bytes);
        registry_calldata.extend(pad_bytes(self.manager_address()));

        let mut bridge_calldata = Vec::new();
        bridge_calldata.extend(empty_bytes);
        bridge_calldata.extend(pad_bytes(self.manager_address()));
        bridge_calldata.extend(pad_bytes(messaging_contract));

        self.manager.initialize(Bytes::from(manager_calldata)).await.expect("Failed to initialize Starkgate Manager");
        self.registry
            .initialize(Bytes::from(registry_calldata))
            .await
            .expect("Failed to initialize Starkgate Registry");
        self.token_bridge
            .initialize(Bytes::from(bridge_calldata))
            .await
            .expect("Failed to initialize Starknet Token Bridge");
    }

    /// Sets up the Token bridge with the specified data
    pub async fn setup_l1_bridge(&self, governor: Address, l2_bridge: FieldElement, fee: U256) {
        self.token_bridge.register_app_role_admin(governor).await.unwrap();
        self.token_bridge.register_app_governor(governor).await.unwrap();
        self.token_bridge.set_l2_token_bridge(U256::from(Felt252Wrapper(l2_bridge))).await.unwrap();
        self.manager.enroll_token_bridge(self.dai_address(), fee).await.unwrap();
    }

    pub async fn setup_l2_bridge(&self, madara: &ThreadSafeMadaraClient, l2_bridge: FieldElement) {
        madara_contract_call(
            madara,
            FieldElement::from_hex_be("0x5").unwrap(),
            "__execute__",
            vec![
                l2_bridge,                                                  // contract_address
                get_selector_from_name("register_app_role_admin").unwrap(), // selector
                FieldElement::ONE,                                          // calldata_len
                FieldElement::from_hex_be("0x4").unwrap(),                  // calldata (upgrade_delay)
            ],
        )
        .await;

        madara_contract_call(
            madara,
            l2_bridge,
            "register_app_governor",
            vec![FieldElement::from_hex_be(CAIRO_1_ACCOUNT_CONTRACT).unwrap()],
        )
        .await;

        madara_contract_call(
            madara,
            l2_bridge,
            "set_l2_token_governance",
            vec![FieldElement::from_hex_be(CAIRO_1_ACCOUNT_CONTRACT).unwrap()],
        )
        .await;

        madara_contract_call(
            madara,
            l2_bridge,
            "set_erc20_class_hash",
            vec![
                FieldElement::from_hex_be("0x008b150cfa4db35ed9d685d79f6daa590ff2bb10c295cd656fcbf176c4bd8365")
                    .unwrap(),
            ],
        )
        .await;

        madara_contract_call(
            madara,
            l2_bridge,
            "set_l1_bridge",
            vec![FieldElement::from_byte_slice_be(self.token_bridge.address().as_bytes()).unwrap()],
        )
        .await;
    }

    pub async fn register_app_role_admin(&self, address: Address) {
        self.token_bridge
            .register_app_role_admin(address)
            .await
            .expect("Failed to register app role admin in Starknet Token Bridge");
    }

    pub async fn register_app_governor(&self, address: Address) {
        self.token_bridge
            .register_app_governor(address)
            .await
            .expect("Failed to register app governor in Starknet Token Bridge");
    }

    pub async fn set_l2_token_bridge(&self, l2_bridge: U256) {
        self.token_bridge
            .set_l2_token_bridge(l2_bridge)
            .await
            .expect("Failed to set l2 bridge in Starknet Token Bridge");
    }

    pub async fn deposit(&self, token: Address, amount: U256, l2address: U256, fee: U256) {
        self.token_bridge.deposit(token, amount, l2address, fee).await.expect("Failed to bridge funds from L1 to L2");
    }

    pub async fn withdraw(&self, l1_token: Address, amount: U256, l1_recipient: Address) {
        self.token_bridge
            .withdraw(l1_token, amount, l1_recipient)
            .await
            .expect("Failed to withdraw from Starknet Token Bridge");
    }

    pub async fn enroll_token_bridge(&self, address: Address, fee: U256) {
        self.manager.enroll_token_bridge(address, fee).await.expect("Failed to enroll token in Starknet Token Bridge");
    }

    pub async fn approve(&self, address: Address, amount: U256) {
        self.dai_erc20
            .approve(address, amount)
            .await
            .expect("Failed to approve dai transfer for Starknet Token Bridge");
    }

    pub async fn token_balance(&self, address: Address) -> U256 {
        self.dai_erc20.balance_of(address).await.unwrap()
    }
}

fn pad_bytes(address: Address) -> Vec<u8> {
    let address_bytes = address.as_bytes();
    let mut padded_address_bytes = Vec::with_capacity(32);
    padded_address_bytes.extend(vec![0u8; 32 - address_bytes.len()]);
    padded_address_bytes.extend_from_slice(&address_bytes);
    padded_address_bytes
}
