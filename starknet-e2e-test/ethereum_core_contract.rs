extern crate starknet_e2e_test;

use madara_runtime::opaque::Block;
use mc_settlement::ethereum::StarknetContractClient;
use mc_settlement::{SettlementProvider, StarknetSpec, StarknetState};
use mp_messages::{MessageL1ToL2, MessageL2ToL1};
use mp_snos_output::StarknetOsOutput;
use rstest::rstest;
use starknet_api::api_core::{ContractAddress, Nonce, PatriciaKey};
use starknet_api::hash::StarkFelt;
use starknet_e2e_test::starknet_sovereign::StarknetSovereign;

#[rstest]
#[tokio::test]
async fn starknet_core_contract_is_initialized() -> anyhow::Result<()> {
    let starknet_sovereign = StarknetSovereign::deploy().await;
    starknet_sovereign.initialize(1u64.into(), 1u64.into()).await;

    let starknet = StarknetContractClient::new(starknet_sovereign.address(), starknet_sovereign.client());

    let spec = SettlementProvider::<Block>::get_chain_spec(&starknet).await.expect("Failed to get chain spec");
    assert_eq!(spec, StarknetSpec { program_hash: 1u64.into(), config_hash: 1u64.into() });

    let state = SettlementProvider::<Block>::get_state(&starknet).await.expect("Failed to get state");
    assert_eq!(state, StarknetState::default());

    Ok(())
}

#[rstest]
#[tokio::test]
async fn starknet_core_contract_advances_state() -> anyhow::Result<()> {
    let starknet_sovereign = StarknetSovereign::deploy().await;
    starknet_sovereign.initialize(1u64.into(), 1u64.into()).await;

    let starknet = StarknetContractClient::new(starknet_sovereign.address(), starknet_sovereign.client());

    // Now let's transition the state from block 0 to 1 (state root 0 -> 1)
    let program_output = StarknetOsOutput {
        new_state_root: 1u64.into(),
        block_number: 1u64.into(),
        config_hash: 1u64.into(),
        ..Default::default()
    };

    SettlementProvider::<Block>::update_state(&starknet, program_output).await.expect("Failed to update state");

    let state = SettlementProvider::<Block>::get_state(&starknet).await.expect("Failed to get state");
    assert_eq!(state, StarknetState { block_number: 1u64.into(), state_root: 1u64.into() });

    Ok(())
}

#[rstest]
#[tokio::test]
async fn starknet_core_contract_sends_messages_to_l2() -> anyhow::Result<()> {
    // In this test we do not check Starknet messaging logic, but rather that all our encodings are
    // correct

    let starknet_sovereign = StarknetSovereign::deploy().await;
    starknet_sovereign.initialize(1u64.into(), 1u64.into()).await;

    // Converting our EOA address to felt
    let mut from_address = [0u8; 32];
    from_address[12..32].copy_from_slice(starknet_sovereign.client().address().as_bytes());

    let message = MessageL1ToL2 {
        from_address: ContractAddress(PatriciaKey(StarkFelt::new(from_address).unwrap())),
        to_address: 3u64.into(),
        nonce: Nonce(0u64.into()), // Starknet contract maintains global nonce counter
        selector: 2u64.into(),
        payload: vec![1u64.into()],
    };

    // Sending message to L2 (this will update msg hash table)
    starknet_sovereign.send_message_to_l2(&message).await;
    assert!(starknet_sovereign.message_to_l2_exists(&message).await);

    let starknet = StarknetContractClient::new(starknet_sovereign.address(), starknet_sovereign.client());

    let program_output = StarknetOsOutput {
        new_state_root: 1u64.into(),
        block_number: 1u64.into(),
        config_hash: 1u64.into(),
        messages_to_l2: vec![message.clone()],
        ..Default::default()
    };

    // During the state update, the message will be consumed (removed from hash table)
    SettlementProvider::<Block>::update_state(&starknet, program_output).await.expect("Failed to update state");

    // At this point the counter has to be reset
    assert!(!starknet_sovereign.message_to_l2_exists(&message).await);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn starknet_core_contract_consumes_messages_from_l2() -> anyhow::Result<()> {
    // In this test we do not check Starknet messaging logic, but rather that all our encodings are
    // correct

    let starknet_sovereign = StarknetSovereign::deploy().await;
    starknet_sovereign.initialize(1u64.into(), 1u64.into()).await;

    let message = MessageL2ToL1 {
        from_address: 1u64.into(),
        to_address: StarkFelt::from(2u64).try_into().unwrap(),
        payload: vec![3u64.into()],
    };
    let starknet = StarknetContractClient::new(starknet_sovereign.address(), starknet_sovereign.client());

    let program_output = StarknetOsOutput {
        new_state_root: 1u64.into(),
        block_number: 1u64.into(),
        config_hash: 1u64.into(),
        messages_to_l1: vec![message.clone()],
        ..Default::default()
    };

    // During the state update, the message will be consumed (removed from hash table)
    SettlementProvider::<Block>::update_state(&starknet, program_output).await.expect("Failed to update state");

    assert!(starknet_sovereign.message_to_l1_exists(&message).await);

    Ok(())
}
