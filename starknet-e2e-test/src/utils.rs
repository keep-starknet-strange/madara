use std::str::FromStr;
use std::time::Duration;

use ethers::addressbook::Address;
use starknet_accounts::{Account, Execution};
use starknet_contract::ContractFactory;
use starknet_ff::FieldElement;
use starknet_test_utils::constants::{CAIRO_1_ACCOUNT_CONTRACT, SIGNER_PRIVATE};
use starknet_test_utils::fixtures::ThreadSafeMadaraClient;
use starknet_test_utils::utils::{build_single_owner_account, AccountActions};
use starknet_test_utils::Transaction;
use tokio::time::sleep;

const ERC20_SIERRA_PATH: &str = "../starknet-e2e-test/contracts/erc20.sierra.json";
const ERC20_CASM_PATH: &str = "../starknet-e2e-test/contracts/erc20.casm.json";

pub async fn deploy_eth_token_on_l2(madara: &ThreadSafeMadaraClient, minter: FieldElement) -> FieldElement {
    let rpc = madara.get_starknet_client().await;
    let account = build_single_owner_account(&rpc, SIGNER_PRIVATE, CAIRO_1_ACCOUNT_CONTRACT, false);

    let (declare_tx, class_hash, _) = account.declare_contract(ERC20_SIERRA_PATH, ERC20_CASM_PATH, None);

    let mut madara_write_lock = madara.write().await;

    madara_write_lock
        .create_block_with_txs(vec![Transaction::Declaration(declare_tx)])
        .await
        .expect("Unable to declare ERC20 token on L2");

    let contract_factory = ContractFactory::new(class_hash, account.clone());

    let deploy_tx = &contract_factory.deploy(
        vec![
            FieldElement::from_byte_slice_be("ether".as_bytes()).unwrap(), // Name
            FieldElement::from_byte_slice_be("ETH".as_bytes()).unwrap(),   // Symbol
            FieldElement::from_str("18").unwrap(),                         // Decimals
            FieldElement::from_str("10000000000000000000").unwrap(),       // Initial supply low
            FieldElement::from_str("0").unwrap(),                          // Initial supply high
            account.address(),                                             // recipient
            minter,                                                        // permitted_minter
            account.address(),                                             // provisional_governance_admin
            FieldElement::from_str("0").unwrap(),                          // upgrade_delay
        ],
        FieldElement::ZERO,
        true,
    );

    madara_write_lock
        .create_block_with_txs(vec![Transaction::Execution(Execution::from(deploy_tx))])
        .await
        .expect("Unable to deploy ERC20 token on madara");
    deploy_tx.deployed_address()
}

pub async fn invoke_contract(
    madara: &ThreadSafeMadaraClient,
    contract: FieldElement,
    method: &str,
    calldata: Vec<FieldElement>,
) {
    let rpc = madara.get_starknet_client().await;
    let account = build_single_owner_account(&rpc, SIGNER_PRIVATE, CAIRO_1_ACCOUNT_CONTRACT, false);
    let mut madara_write_lock = madara.write().await;

    let call = account.invoke_contract(contract, method, calldata, None);

    madara_write_lock
        .create_block_with_txs(vec![Transaction::Execution(call)])
        .await
        .expect("Failed to make a contract call to madara");
}

pub async fn catch_and_execute_l1_messages(madara: &ThreadSafeMadaraClient) {
    // Wait for worker to catch L1 messages
    sleep(Duration::from_millis(12000)).await;
    let mut madara_write_lock = madara.write().await;
    madara_write_lock.create_block_with_pending_txs().await.expect("Failed to execute L1 Messages");
}

pub fn pad_bytes(address: Address) -> Vec<u8> {
    let address_bytes = address.as_bytes();
    let mut padded_address_bytes = Vec::with_capacity(32);
    padded_address_bytes.extend(vec![0u8; 32 - address_bytes.len()]);
    padded_address_bytes.extend_from_slice(address_bytes);
    padded_address_bytes
}
