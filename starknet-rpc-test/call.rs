#![feature(assert_matches)]

mod get_block_hash_and_number;

extern crate starknet_rpc_test;

use std::assert_matches::assert_matches;

use rstest::rstest;
use starknet_accounts::{Account, Execution};
use starknet_contract::ContractFactory;
use starknet_core::types::{BlockId, BlockTag, FunctionCall, StarknetError};
use starknet_core::utils::get_selector_from_name;
use starknet_ff::FieldElement;
use starknet_providers::{MaybeUnknownErrorCode, Provider, ProviderError, StarknetErrorWithMessage};
use starknet_rpc_test::constants::{ARGENT_CONTRACT_ADDRESS, FEE_TOKEN_ADDRESS, SIGNER_PRIVATE};
use starknet_rpc_test::fixtures::{madara, ThreadSafeMadaraClient};
use starknet_rpc_test::utils::{build_single_owner_account, AccountActions};
use starknet_rpc_test::Transaction;

#[rstest]
#[tokio::test]
async fn fail_non_existing_block(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    assert_matches!(
        rpc.call(
            FunctionCall {
                contract_address: FieldElement::from_hex_be(FEE_TOKEN_ADDRESS).unwrap(),
                entry_point_selector: get_selector_from_name("name").unwrap(),
                calldata: vec![]
            },
            BlockId::Hash(FieldElement::ZERO)
        )
        .await
        .err(),
        Some(ProviderError::StarknetError(StarknetErrorWithMessage {
            message: _,
            code: MaybeUnknownErrorCode::Known(StarknetError::BlockNotFound)
        }))
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn fail_non_existing_entrypoint(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    assert_matches!(
        rpc.call(
            FunctionCall {
                contract_address: FieldElement::from_hex_be(FEE_TOKEN_ADDRESS).unwrap(),
                entry_point_selector: FieldElement::from_hex_be("0x0").unwrap(),
                calldata: vec![]
            },
            BlockId::Tag(BlockTag::Latest)
        )
        .await
        .err(),
        Some(ProviderError::StarknetError(StarknetErrorWithMessage {
            message: _,
            code: MaybeUnknownErrorCode::Known(StarknetError::ContractError)
        }))
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn fail_incorrect_calldata(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    assert_matches!(
        rpc.call(
            FunctionCall {
                contract_address: FieldElement::from_hex_be(FEE_TOKEN_ADDRESS).unwrap(),
                entry_point_selector: get_selector_from_name("name").unwrap(),
                calldata: vec![FieldElement::ONE] // name function has no calldata
            },
            BlockId::Tag(BlockTag::Latest)
        )
        .await
        .err(),
        Some(ProviderError::StarknetError(StarknetErrorWithMessage {
            message: _,
            code: MaybeUnknownErrorCode::Known(StarknetError::ContractError)
        }))
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn works_on_correct_call_no_calldata(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    assert_eq!(
        rpc.call(
            FunctionCall {
                contract_address: FieldElement::from_hex_be(FEE_TOKEN_ADDRESS).unwrap(),
                entry_point_selector: get_selector_from_name("name").unwrap(),
                calldata: vec![] // name function has no calldata
            },
            BlockId::Tag(BlockTag::Latest)
        )
        .await
        .unwrap(),
        vec![FieldElement::ZERO]
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn works_on_correct_call_with_calldata(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    assert!(
        rpc.call(
            FunctionCall {
                contract_address: FieldElement::from_hex_be(FEE_TOKEN_ADDRESS).unwrap(),
                entry_point_selector: get_selector_from_name("balanceOf").unwrap(),
                calldata: vec![FieldElement::TWO] // name function has no calldata
            },
            BlockId::Tag(BlockTag::Latest)
        )
        .await
        .unwrap()[0]
            .gt(&FieldElement::ZERO)
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn works_on_mutable_call_without_modifying_storage(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    {
        let mut madara_write_lock = madara.write().await;
        let account = build_single_owner_account(&rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);

        let (declare_tx, class_hash, _) =
            account.declare_contract("./contracts/Counter.sierra.json", "./contracts/Counter.casm.json");
        let contract_factory = ContractFactory::new(class_hash, account.clone());

        // manually setting fee else estimate_fee will be called and it will fail
        // as contract is not declared yet (declared in the same block as deployment)
        let max_fee = FieldElement::from_hex_be("0x1000000000").unwrap();

        // manually incrementing nonce else as both declare and deploy are in the same block
        // so automatic nonce calculation will fail
        let nonce = rpc.get_nonce(BlockId::Tag(BlockTag::Latest), account.address()).await.unwrap() + FieldElement::ONE;

        let deploy_tx =
            Execution::from(&contract_factory.deploy(vec![], FieldElement::ZERO, true).max_fee(max_fee).nonce(nonce));

        println!("before declare");
        // declare and deploy contract
        madara_write_lock.create_block_with_txs(vec![Transaction::Declaration(declare_tx)]).await?;
        println!("before execute");
        madara_write_lock.create_block_with_txs(vec![Transaction::Execution(deploy_tx)]).await?;
        println!("after block");
    }

    // address of deployed contract (will always be the same for 0 salt)
    let contract_address =
        FieldElement::from_hex_be("0x0226d81ce04c3c7081fe05f51b32b75210aad1ea8be8bce566f26d25d5ffb4c3").unwrap();

    let read_balance = || async {
        rpc.call(
            FunctionCall {
                contract_address,
                entry_point_selector: get_selector_from_name("get_balance").unwrap(),
                calldata: vec![],
            },
            BlockId::Tag(BlockTag::Latest),
        )
        .await
        .unwrap()
    };

    let initial_balance = read_balance().await[0];
    // call increase_balance and verify it returns a result
    assert!(
        rpc.call(
            FunctionCall {
                contract_address,
                entry_point_selector: get_selector_from_name("increase_balance").unwrap(),
                calldata: vec![FieldElement::ONE]
            },
            BlockId::Tag(BlockTag::Latest)
        )
        .await
        .is_ok()
    );
    let final_balance = read_balance().await[0];

    // initial and final balance should be same as starknet_call doesn't change storage
    assert_eq!(initial_balance, final_balance);

    Ok(())
}
