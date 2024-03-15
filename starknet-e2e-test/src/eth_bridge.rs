use std::sync::Arc;
use std::time::Duration;

use ethers::addressbook::Address;
use ethers::prelude::{Http, Provider};
use ethers::providers::Middleware;
use ethers::types::{Bytes, U256};
use mp_felt::Felt252Wrapper;
use starknet_accounts::Execution;
use starknet_contract::ContractFactory;
use starknet_core_contract_client::clients::StarknetEthBridgeContractClient;
use starknet_core_contract_client::interfaces::{ProxySupportTrait, StarknetEthBridgeTrait};
use starknet_core_contract_client::{LocalWalletSignerMiddleware, StarknetContractClient};
use starknet_ff::FieldElement;
use starknet_rpc_test::constants::{CAIRO_1_ACCOUNT_CONTRACT, SIGNER_PRIVATE};
use starknet_rpc_test::fixtures::ThreadSafeMadaraClient;
use starknet_rpc_test::utils::{build_single_owner_account, AccountActions};
use starknet_rpc_test::Transaction;
use zaun_sandbox::deploy::deploy_starknet_eth_bridge_behind_unsafe_proxy;
use zaun_sandbox::EthereumSandbox;

use crate::utils::madara_contract_call;

pub struct StarknetLegacyEthBridge {
    client: StarknetEthBridgeContractClient,
}

impl StarknetLegacyEthBridge {
    pub fn address(&self) -> Address {
        self.client.address()
    }

    pub fn client(&self) -> Arc<LocalWalletSignerMiddleware> {
        self.client.client()
    }

    pub async fn deploy(client: Arc<LocalWalletSignerMiddleware>) -> Self {
        let client = deploy_starknet_eth_bridge_behind_unsafe_proxy(client.clone())
            .await
            .expect("Failed to deploy starknet contract");

        Self { client }
    }

    pub async fn deploy_l2_contracts(madara: &ThreadSafeMadaraClient) -> FieldElement {
        let rpc = madara.get_starknet_client().await;
        let account = build_single_owner_account(&rpc, SIGNER_PRIVATE, CAIRO_1_ACCOUNT_CONTRACT, false);

        let (declare_tx, class_hash) =
            account.declare_legacy_contract("../starknet-e2e-test/contracts/legacy_token_bridge.json");

        let mut madara_write_lock = madara.write().await;

        madara_write_lock
            .create_block_with_txs(vec![Transaction::LegacyDeclaration(declare_tx)])
            .await
            .expect("Unable to declare legacy token bridge on l2");

        let contract_factory = ContractFactory::new(class_hash, account.clone());
        let deploy_tx = &contract_factory.deploy(vec![], FieldElement::ZERO, true);

        madara_write_lock
            .create_block_with_txs(vec![Transaction::Execution(Execution::from(deploy_tx))])
            .await
            .expect("Unable to deploy legacy token bridge on l2");
        deploy_tx.deployed_address()
    }

    /// Initialize Starknet core contract with the specified data.
    ///
    /// Also register Anvil default account as an operator.
    pub async fn initialize(&self, messaging_contract: Address) {
        let empty_bytes = [0u8; 32];

        let messaging_bytes = messaging_contract.as_bytes();

        let mut padded_messaging_bytes = Vec::with_capacity(32);
        padded_messaging_bytes.extend(vec![0u8; 32 - messaging_bytes.len()]);
        padded_messaging_bytes.extend_from_slice(&messaging_bytes);

        let mut calldata = Vec::new();
        calldata.extend(empty_bytes);
        calldata.extend(empty_bytes);
        calldata.extend(padded_messaging_bytes);

        self.client.initialize(Bytes::from(calldata)).await.expect("Failed to initialize Eth bridge");
    }

    /// Sets up the Eth bridge with the specified data
    pub async fn setup_l1_bridge(&self, max_total_balance: &str, max_deposit: &str, l2_bridge: FieldElement) {
        self.client.set_max_total_balance(U256::from_dec_str(max_total_balance).unwrap()).await.unwrap();
        self.client.set_max_deposit(U256::from_dec_str(max_deposit).unwrap()).await.unwrap();
        self.client.set_l2_token_bridge(U256::from(Felt252Wrapper(l2_bridge))).await.unwrap();
    }

    pub async fn setup_l2_bridge(
        &self,
        madara: &ThreadSafeMadaraClient,
        l2_bridge_address: FieldElement,
        erc20_address: FieldElement,
    ) {
        madara_contract_call(
            madara,
            l2_bridge_address,
            "initialize",
            vec![
                FieldElement::from_dec_str("1").unwrap(),
                FieldElement::from_hex_be(CAIRO_1_ACCOUNT_CONTRACT).unwrap(),
            ],
        )
        .await;

        madara_contract_call(madara, l2_bridge_address, "set_l2_token", vec![erc20_address]).await;

        madara_contract_call(
            madara,
            l2_bridge_address,
            "set_l1_bridge",
            vec![FieldElement::from_byte_slice_be(self.client.address().as_bytes()).unwrap()],
        )
        .await;
    }

    pub async fn set_max_total_balance(&self, amount: U256) {
        self.client.set_max_total_balance(amount).await.expect("Failed to set max total balance value in Eth bridge");
    }

    pub async fn set_max_deposit(&self, amount: U256) {
        self.client.set_max_deposit(amount).await.expect("Failed to set max deposit value in Eth bridge");
    }

    pub async fn set_l2_token_bridge(&self, l2_bridge: U256) {
        self.client.set_l2_token_bridge(l2_bridge).await.expect("Failed to set l2 bridge in Eth bridge");
    }

    pub async fn deposit(&self, amount: U256, l2_address: U256, fee: U256) {
        self.client.deposit(amount, l2_address, fee).await.expect("Failed to deposit in Eth bridge");
    }

    pub async fn withdraw(&self, amount: U256, l1_recipient: Address) {
        self.client.withdraw(amount, l1_recipient).await.expect("Failed to withdraw from Eth bridge");
    }

    pub async fn eth_balance(&self, l1_recipient: Address) -> U256 {
        // todo: move this to zaun
        let provider = Provider::<Http>::try_from("http://localhost:8545")
            .expect("Failed to connect to Anvil")
            .interval(Duration::from_millis(10));

        let balance = provider.get_balance(l1_recipient, None).await.unwrap();
        balance
    }
}
