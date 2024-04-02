use std::sync::Arc;

use async_trait::async_trait;
use ethers::addressbook::Address;
use ethers::prelude::U256;
use ethers::types::Bytes;
use mp_felt::Felt252Wrapper;
use rand::Rng;
use starknet_accounts::ConnectedAccount;
use starknet_core::utils::get_selector_from_name;
use starknet_erc20_client::clients::erc20::ERC20ContractClient;
use starkgate_manager_client::clients::starkgate_manager::StarkgateManagerContractClient;
use starkgate_registry_client::clients::starkgate_registry::StarkgateRegistryContractClient;
use starknet_erc20_client::interfaces::erc20::ERC20TokenTrait;
use starknet_token_bridge_client::clients::token_bridge::StarknetTokenBridgeContractClient;
use starknet_proxy_client::proxy_support::ProxySupportTrait;
use starknet_token_bridge_client::interfaces::token_bridge::StarknetTokenBridgeTrait;
use starkgate_manager_client::interfaces::manager::StarkgateManagerTrait;
use zaun_utils::{LocalWalletSignerMiddleware, StarknetContractClient};
use starknet_ff::FieldElement;
use starknet_test_utils::constants::{
    CAIRO_1_ACCOUNT_CONTRACT, ERC20_CASM_PATH, ERC20_SIERRA_PATH, SIGNER_PRIVATE, TOKEN_BRIDGE_CASM_PATH,
    TOKEN_BRIDGE_SIERRA_PATH,
};
use starknet_test_utils::fixtures::ThreadSafeMadaraClient;
use starknet_test_utils::utils::{build_single_owner_account, get_contract_address_from_deploy_tx, AccountActions};
use starknet_test_utils::Transaction;
use starknet_erc20_client::deploy_dai_erc20_behind_unsafe_proxy;
use starkgate_manager_client::deploy_starkgate_manager_behind_unsafe_proxy;
use starkgate_registry_client::deploy_starkgate_registry_behind_unsafe_proxy;
use starknet_token_bridge_client::deploy_starknet_token_bridge_behind_unsafe_proxy;
use crate::utils::{invoke_contract, pad_bytes};
use crate::BridgeDeployable;

pub struct StarknetTokenBridge {
    manager: StarkgateManagerContractClient,
    registry: StarkgateRegistryContractClient,
    token_bridge: StarknetTokenBridgeContractClient,
    dai_erc20: ERC20ContractClient,
}

#[async_trait]
impl BridgeDeployable for StarknetTokenBridge {
    async fn deploy(client: Arc<LocalWalletSignerMiddleware>) -> Self {
        let manager = deploy_starkgate_manager_behind_unsafe_proxy(client.clone())
            .await
            .expect("Failed to deploy starkgate manager contract");
        let registry = deploy_starkgate_registry_behind_unsafe_proxy(client.clone())
            .await
            .expect("Failed to deploy starkgate registry");
        let token_bridge = deploy_starknet_token_bridge_behind_unsafe_proxy(client.clone())
            .await
            .expect("Failed to deploy starknet contract");
        let dai_erc20 =
            deploy_dai_erc20_behind_unsafe_proxy(client.clone()).await.expect("Failed to deploy dai erc20 contract");

        Self { manager, registry, token_bridge, dai_erc20 }
    }
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

    pub async fn deploy_l2_contracts(madara: &ThreadSafeMadaraClient) -> FieldElement {
        let rpc = madara.get_starknet_client().await;
        let account = build_single_owner_account(&rpc, SIGNER_PRIVATE, CAIRO_1_ACCOUNT_CONTRACT, false);
        let mut madara_write_lock = madara.write().await;

        let (erc20_declare_tx, _, _) = account.declare_contract(ERC20_SIERRA_PATH, ERC20_CASM_PATH);

        let (bridge_declare_tx, _, _) = account.declare_contract(TOKEN_BRIDGE_SIERRA_PATH, TOKEN_BRIDGE_CASM_PATH);

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
                FieldElement::from_hex_be("0x0358663e6ed9d37efd33d4661e20b2bad143e0f92076b0c91fe65f31ccf55046")
                    .unwrap(), // class_hash
                FieldElement::from_dec_str(&random.to_string()).unwrap(), // contract_address_salt
                FieldElement::ONE,                                        // constructor_calldata_len
                FieldElement::ZERO,                                       // constructor_calldata (upgrade_delay)
            ],
            None,
        );

        let mut txs = madara_write_lock.create_block_with_txs(vec![Transaction::Execution(deploy_tx)]).await.unwrap();

        let deploy_tx_result = txs.pop().unwrap();
        get_contract_address_from_deploy_tx(&rpc, deploy_tx_result).await.unwrap()
    }

    /// Initialize Starknet Token Bridge.
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

        self.manager.initialize(Bytes::from(manager_calldata)).await.expect("Failed to initialize starkgate manager");
        self.registry
            .initialize(Bytes::from(registry_calldata))
            .await
            .expect("Failed to initialize starkgate registry");
        self.token_bridge
            .initialize(Bytes::from(bridge_calldata))
            .await
            .expect("Failed to initialize starknet token bridge");
    }

    /// Sets up the Token bridge with the specified data
    pub async fn setup_l1_bridge(&self, governor: Address, l2_bridge: FieldElement, fee: U256) {
        self.token_bridge.register_app_role_admin(governor).await.unwrap();
        self.token_bridge.register_app_governor(governor).await.unwrap();
        self.token_bridge.set_l2_token_bridge(U256::from(Felt252Wrapper(l2_bridge))).await.unwrap();
        self.manager.enroll_token_bridge(self.dai_address(), fee).await.unwrap();
    }

    pub async fn setup_l2_bridge(&self, madara: &ThreadSafeMadaraClient, l2_bridge: FieldElement) {
        invoke_contract(
            madara,
            FieldElement::from_hex_be("0x5").unwrap(),
            "__execute__",
            vec![
                l2_bridge,                                                  // contract_address
                get_selector_from_name("register_app_role_admin").unwrap(), // selector
                FieldElement::ONE,                                          // calldata_len
                FieldElement::from_hex_be("0x4").unwrap(),                  // admin_address
            ],
        )
        .await;

        invoke_contract(
            madara,
            l2_bridge,
            "register_app_governor",
            vec![FieldElement::from_hex_be(CAIRO_1_ACCOUNT_CONTRACT).unwrap()],
        )
        .await;

        invoke_contract(
            madara,
            l2_bridge,
            "set_l2_token_governance",
            vec![FieldElement::from_hex_be(CAIRO_1_ACCOUNT_CONTRACT).unwrap()],
        )
        .await;

        invoke_contract(
            madara,
            l2_bridge,
            "set_erc20_class_hash",
            vec![
                FieldElement::from_hex_be("0x008b150cfa4db35ed9d685d79f6daa590ff2bb10c295cd656fcbf176c4bd8365")
                    .unwrap(), // class hash
            ],
        )
        .await;

        invoke_contract(
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
            .expect("Failed to register app role admin in starknet token bridge");
    }

    pub async fn register_app_governor(&self, address: Address) {
        self.token_bridge
            .register_app_governor(address)
            .await
            .expect("Failed to register app governor in starknet token bridge");
    }

    pub async fn set_l2_token_bridge(&self, l2_bridge: U256) {
        self.token_bridge
            .set_l2_token_bridge(l2_bridge)
            .await
            .expect("Failed to set l2 bridge in starknet token bridge");
    }

    pub async fn deposit(&self, token: Address, amount: U256, l2address: U256, fee: U256) {
        self.token_bridge.deposit(token, amount, l2address, fee).await.expect("Failed to bridge funds from l1 to l2");
    }

    pub async fn withdraw(&self, l1_token: Address, amount: U256, l1_recipient: Address) {
        self.token_bridge
            .withdraw(l1_token, amount, l1_recipient)
            .await
            .expect("Failed to withdraw from starknet token bridge");
    }

    pub async fn enroll_token_bridge(&self, address: Address, fee: U256) {
        self.manager.enroll_token_bridge(address, fee).await.expect("Failed to enroll token in starknet token bridge");
    }

    pub async fn approve(&self, address: Address, amount: U256) {
        self.dai_erc20
            .approve(address, amount)
            .await
            .expect("Failed to approve dai transfer for starknet token bridge");
    }

    pub async fn token_balance(&self, address: Address) -> U256 {
        self.dai_erc20.balance_of(address).await.unwrap()
    }
}
