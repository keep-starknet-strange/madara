extern crate starknet_e2e_test;
use std::str::FromStr;
use std::time::Duration;

use ethers::abi::Address;
use ethers::types::{H160, U256};
use madara_runtime::opaque::Block;
use madara_test_runner::node::MadaraTempDir;
use madara_test_runner::{MadaraArgs, MadaraRunner, Settlement};
use mc_settlement::ethereum::StarknetContractClient;
use mc_settlement::{SettlementProvider, StarknetState};
use rstest::rstest;
use starknet_core::types::{BlockId, BlockTag, FunctionCall};
use starknet_core::utils::get_selector_from_name;
use starknet_e2e_test::eth_bridge::StarknetLegacyEthBridge;
use starknet_e2e_test::starknet_sovereign::StarknetSovereign;
use starknet_e2e_test::token_bridge::StarknetTokenBridge;
use starknet_e2e_test::utils::{catch_and_execute_l1_messages, deploy_eth_token_on_l2, invoke_contract};
use starknet_e2e_test::BridgeDeployable;
use starknet_ff::FieldElement;
use starknet_providers::Provider;
use starknet_test_utils::constants::{ANVIL_DEFAULT_PUBLIC_ADDRESS, CAIRO_1_ACCOUNT_CONTRACT, SIGNER_PUBLIC};
use starknet_test_utils::fixtures::madara_from;
use starknet_test_utils::utils::read_erc20_balance;
use tokio::time::sleep;

const L1_RECIPIENT: &str = "0x59FA6981892396D67BBd31e80E9d91506213335F";

#[rstest]
#[tokio::test]
async fn madara_advances_ethereum_settlement_contract_state_in_sovereign_mode() -> Result<(), anyhow::Error> {
    // Troubleshooting:
    // RUST_LOG=mc_settlement=trace MADARA_LOG=1 cargo test --package starknet-e2e-test
    // works_with_initialized_contract -- --nocapture

    // Run or attach to Anvil sandbox, deploy & initialize core contract
    let starknet_sovereign = StarknetSovereign::deploy().await;
    starknet_sovereign.initialize_for_goerli(0u64.into(), 0u64.into()).await;

    // Create tmp Madara path and write settlement config
    let madara_temp_dir = MadaraTempDir::default();
    let settlement_conf = starknet_sovereign.create_settlement_conf(madara_temp_dir.data_path()).await;

    // Launch new Madara instance and connect to it
    let mut madara = MadaraRunner::new(MadaraArgs {
        settlement: Some(Settlement::Ethereum),
        settlement_conf: Some(settlement_conf),
        base_path: Some(madara_temp_dir.base_path()),
    })
    .await;

    madara.create_n_blocks(3).await?;
    sleep(Duration::from_millis(300)).await;

    let client = StarknetContractClient::new(starknet_sovereign.address(), starknet_sovereign.client());
    let state = SettlementProvider::<Block>::get_state(&client).await?;

    assert_eq!(state, StarknetState { block_number: 3u64.into(), state_root: 0u64.into() });

    Ok(())
}

#[rstest]
#[tokio::test]
async fn deposit_and_withdraw_from_eth_bridge() -> Result<(), anyhow::Error> {
    let starknet_sovereign = StarknetSovereign::deploy().await;
    starknet_sovereign.initialize_for_goerli(0u64.into(), 0u64.into()).await;

    // Create tmp Madara path and write settlement config
    let madara_temp_dir = MadaraTempDir::default();
    let settlement_conf = starknet_sovereign.create_settlement_conf(madara_temp_dir.data_path()).await;

    // Launch new Madara instance and connect to it
    let madara_runner = MadaraRunner::new(MadaraArgs {
        settlement: Some(Settlement::Ethereum),
        settlement_conf: Some(settlement_conf),
        base_path: Some(madara_temp_dir.base_path()),
    })
    .await;

    let madara = madara_from(madara_runner.url());

    let eth_bridge = StarknetLegacyEthBridge::deploy(starknet_sovereign.client().clone()).await;
    let l2_bridge_address = StarknetLegacyEthBridge::deploy_l2_contracts(&madara).await;
    let l2_eth_address = deploy_eth_token_on_l2(&madara, l2_bridge_address).await;

    eth_bridge.initialize(starknet_sovereign.address()).await;
    eth_bridge.setup_l2_bridge(&madara, l2_bridge_address, l2_eth_address).await;
    eth_bridge.setup_l1_bridge("10000000000000000", "10000000000000000", l2_bridge_address).await;

    let rpc = madara.get_starknet_client().await;
    let balance_before =
        read_erc20_balance(&rpc, l2_eth_address, FieldElement::from_hex_be(SIGNER_PUBLIC).unwrap()).await;

    eth_bridge.deposit(10.into(), U256::from_str(SIGNER_PUBLIC).unwrap(), 1000.into()).await;
    catch_and_execute_l1_messages(&madara).await;

    let balance_after =
        read_erc20_balance(&rpc, l2_eth_address, FieldElement::from_hex_be(SIGNER_PUBLIC).unwrap()).await;

    // balance_before + deposited_amount = balance_after
    assert_eq!(balance_before[0] + FieldElement::from_dec_str("10").unwrap(), balance_after[0]);

    let l1_recipient = FieldElement::from_hex_be(L1_RECIPIENT).unwrap();
    invoke_contract(
        &madara,
        l2_bridge_address,
        "initiate_withdraw",
        vec![l1_recipient, FieldElement::from_dec_str("5").unwrap(), FieldElement::ZERO],
    )
    .await;

    catch_and_execute_l1_messages(&madara).await;

    // Wait for worker to catch L2 messages and send to L1 (state update post block finalization)
    let mut madara_write_lock = madara.write().await;
    madara_write_lock.create_n_blocks(2).await.expect("Unable to create empty blocks in madara");
    sleep(Duration::from_millis(12000)).await;

    let l1_recipient: Address = Address::from_str(L1_RECIPIENT).unwrap();
    let balance_before = eth_bridge.eth_balance(l1_recipient).await;
    eth_bridge.withdraw(5.into(), l1_recipient).await;
    let balance_after = eth_bridge.eth_balance(l1_recipient).await;

    // balance_before + withdrawn_amount = balance_after
    assert_eq!(balance_before + U256::from_dec_str("5").unwrap(), balance_after);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn deposit_and_withdraw_from_erc20_bridge() -> Result<(), anyhow::Error> {
    let starknet_sovereign = StarknetSovereign::deploy().await;
    starknet_sovereign.initialize_for_goerli(0u64.into(), 0u64.into()).await;

    // Create tmp Madara path and write settlement config
    let madara_temp_dir = MadaraTempDir::default();
    let settlement_conf = starknet_sovereign.create_settlement_conf(madara_temp_dir.data_path()).await;

    // Launch new Madara instance and connect to it
    let madara_runner = MadaraRunner::new(MadaraArgs {
        settlement: Some(Settlement::Ethereum),
        settlement_conf: Some(settlement_conf),
        base_path: Some(madara_temp_dir.base_path()),
    })
    .await;

    let madara = madara_from(madara_runner.url());

    let token_bridge = StarknetTokenBridge::deploy(starknet_sovereign.client().clone()).await;
    let l2_bridge_address = StarknetTokenBridge::deploy_l2_contracts(&madara).await;

    token_bridge.initialize(starknet_sovereign.address()).await;
    token_bridge.setup_l2_bridge(&madara, l2_bridge_address).await;
    token_bridge
        .setup_l1_bridge(
            H160::from_str(ANVIL_DEFAULT_PUBLIC_ADDRESS).unwrap(),
            l2_bridge_address,
            U256::from_dec_str("100000000000000").unwrap(),
        )
        .await;

    catch_and_execute_l1_messages(&madara).await;

    sleep(Duration::from_millis(20000)).await;

    let rpc = madara.get_starknet_client().await;

    // Wait for l1_message execution to complete (nonce mismatch)
    sleep(Duration::from_millis(20000)).await;

    let l2_token_address = rpc
        .call(
            FunctionCall {
                contract_address: l2_bridge_address,
                entry_point_selector: get_selector_from_name("get_l2_token").unwrap(),
                calldata: vec![FieldElement::from_byte_slice_be(token_bridge.dai_address().as_bytes()).unwrap()],
            },
            BlockId::Tag(BlockTag::Latest),
        )
        .await
        .unwrap()[0];

    println!("L2 token address [erc20] : {:?}, L2 Bridge address : {:?}", l2_token_address, l2_bridge_address);

    token_bridge.approve(token_bridge.bridge_address(), 100000000.into()).await;
    catch_and_execute_l1_messages(&madara).await;

    // waiting for erc20 token to be deployed on l2. It may take some time
    sleep(Duration::from_millis(30000)).await;

    let balance_before =
        read_erc20_balance(&rpc, l2_token_address, FieldElement::from_hex_be(CAIRO_1_ACCOUNT_CONTRACT).unwrap()).await;

    println!(">>>> balance before : {:?}", balance_before);

    token_bridge
        .deposit(
            token_bridge.dai_address(),
            10.into(),
            U256::from_str(CAIRO_1_ACCOUNT_CONTRACT).unwrap(),
            U256::from_dec_str("100000000000000").unwrap(),
        )
        .await;
    catch_and_execute_l1_messages(&madara).await;

    sleep(Duration::from_millis(20000)).await;

    let balance_after =
        read_erc20_balance(&rpc, l2_token_address, FieldElement::from_hex_be(CAIRO_1_ACCOUNT_CONTRACT).unwrap()).await;

    println!(">>>> balance after : {:?}", balance_after);

    // balance_before + deposited_amount = balance_after
    assert_eq!(balance_before[0] + FieldElement::from_dec_str("10").unwrap(), balance_after[0]);

    let l1_recipient = FieldElement::from_hex_be(L1_RECIPIENT).unwrap();
    invoke_contract(
        &madara,
        l2_bridge_address,
        "initiate_token_withdraw",
        vec![
            FieldElement::from_byte_slice_be(token_bridge.dai_address().as_bytes()).unwrap(),
            l1_recipient,
            FieldElement::from_dec_str("5").unwrap(),
            FieldElement::ZERO,
        ],
    )
    .await;

    catch_and_execute_l1_messages(&madara).await;

    // Wait for worker to catch L2 messages and send to L1 (state update post block finalization)
    let mut madara_write_lock = madara.write().await;
    madara_write_lock.create_n_blocks(2).await.expect("Unable to create empty blocks in madara");
    sleep(Duration::from_millis(12000)).await;

    let l1_recipient: Address = Address::from_str(L1_RECIPIENT).unwrap();
    let balance_before = token_bridge.token_balance(l1_recipient).await;
    token_bridge.withdraw(token_bridge.dai_address(), 5.into(), l1_recipient).await;
    let balance_after = token_bridge.token_balance(l1_recipient).await;

    // balance_before + withdrawn_amount = balance_after
    assert_eq!(balance_before + U256::from_dec_str("5").unwrap(), balance_after);

    Ok(())
}
