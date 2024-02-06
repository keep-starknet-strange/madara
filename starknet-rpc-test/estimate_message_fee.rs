extern crate starknet_rpc_test;

use assert_matches::assert_matches;
use rstest::rstest;
use starknet_accounts::{Call, ConnectedAccount};
use starknet_api::api_core::{ContractAddress, PatriciaKey};
use starknet_api::hash::StarkFelt;
use starknet_core::types::{BlockId, BlockTag, BroadcastedTransaction, EthAddress, MsgFromL1, StarknetError};
use starknet_core::utils::get_selector_from_name;
use starknet_ff::FieldElement;
use starknet_providers::sequencer::models::ContractAddresses;
use starknet_providers::ProviderError::StarknetError as StarknetProviderError;
use starknet_providers::{MaybeUnknownErrorCode, Provider, StarknetErrorWithMessage};
use starknet_rpc_test::constants::{
    ARGENT_CONTRACT_ADDRESS, L1_CONTRACT_ADDRESS, SIGNER_PRIVATE, TEST_CONTRACT_ADDRESS, UDC_ADDRESS,
};
use starknet_rpc_test::fixtures::{madara, ThreadSafeMadaraClient};
use starknet_rpc_test::utils::{assert_eq_msg_to_l1, build_single_owner_account, AccountActions};
use starknet_rpc_test::{Transaction, TransactionResult};

#[rstest]
#[tokio::test]
async fn fail_non_existing_block(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let message: MsgFromL1 = MsgFromL1 {
        from_address: EthAddress::from_hex(L1_CONTRACT_ADDRESS).unwrap(),
        to_address: FieldElement::from_hex_be(TEST_CONTRACT_ADDRESS).unwrap(),
        entry_point_selector: get_selector_from_name("sqrt").unwrap(),
        payload: vec![FieldElement::from_hex_be("0x0").unwrap()],
    };

    assert_matches!(
        rpc.estimate_message_fee(message, BlockId::Hash(FieldElement::ZERO)).await,
        Err(StarknetProviderError(StarknetErrorWithMessage { code: MaybeUnknownErrorCode::Known(code), .. })) if code == StarknetError::BlockNotFound
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn fail_if_message_fail(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let message: MsgFromL1 = MsgFromL1 {
        from_address: EthAddress::from_hex(L1_CONTRACT_ADDRESS).unwrap(),
        to_address: FieldElement::from_hex_be(TEST_CONTRACT_ADDRESS).unwrap(),
        entry_point_selector: get_selector_from_name("sqrt").unwrap(),
        payload: vec![FieldElement::from_hex_be("0x0").unwrap()],
    };

    assert_matches!(
        rpc.estimate_message_fee(message, BlockId::Tag(BlockTag::Latest)).await,
        Err(StarknetProviderError(StarknetErrorWithMessage { code: MaybeUnknownErrorCode::Known(code), .. })) if code == StarknetError::ContractError
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn works_ok(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let account = build_single_owner_account(&rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
    let (declare_tx, _) = account.declare_legacy_contract("../cairo-contracts/build/test.json");

    let tx = {
        let mut madara_write_lock = madara.write().await;
        madara_write_lock.create_block_with_txs(vec![Transaction::LegacyDeclaration(declare_tx)]).await?
    };

    let message: MsgFromL1 = MsgFromL1 {
        from_address: EthAddress::from_hex(L1_CONTRACT_ADDRESS).unwrap(),
        to_address: FieldElement::from_hex_be(TEST_CONTRACT_ADDRESS).unwrap(),
        entry_point_selector: get_selector_from_name("sqrt").unwrap(),
        payload: vec![FieldElement::from_hex_be("0x1").unwrap()],
    };

    println!("tx {:?}", tx);

    // let invoke_transaction = BroadcastedTransaction::Invoke(tx);

    let estimates_fee = rpc.estimate_message_fee(message, BlockId::Tag(BlockTag::Latest)).await?;
    println!("estimates_fee: {:?}", estimates_fee);

    Ok(())
    // let message = MsgFromL1 {
    //     from_address: EthAddress::from_hex(L1_CONTRACT_ADDRESS).unwrap(),
    //     to_address: 3u64.into(),
    //     entry_point_selector: 2u64.into(),
    //     payload: vec![1u64.into()],
    // };

    // let estimates_fee = rpc.estimate_message_fee(message, BlockId::Tag(BlockTag::Latest)).await?;

    // assert_eq!(estimates_fee.gas_consumed, 0);
    // assert_eq!(estimates_fee.overall_fee, 0);
}
