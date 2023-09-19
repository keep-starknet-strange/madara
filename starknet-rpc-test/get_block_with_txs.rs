#![feature(assert_matches)]

extern crate starknet_rpc_test;

use std::assert_matches::assert_matches;

use anyhow::anyhow;
use rstest::rstest;
use starknet_core::types::{
    BlockId, BlockStatus, BlockTag, BlockWithTxs, DeclareTransaction, DeclareTransactionV2, DeployAccountTransaction,
    InvokeTransaction, InvokeTransactionV1, MaybePendingBlockWithTxs, StarknetError,
    Transaction as StarknetTransaction,
};
use starknet_core::utils::get_selector_from_name;
use starknet_ff::FieldElement;
use starknet_providers::{MaybeUnknownErrorCode, Provider, ProviderError, StarknetErrorWithMessage};
use starknet_rpc_test::constants::{
    ARGENT_CONTRACT_ADDRESS, CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH, FEE_TOKEN_ADDRESS, MAX_FEE_OVERRIDE, SIGNER_PRIVATE,
};
use starknet_rpc_test::fixtures::madara;
use starknet_rpc_test::utils::{
    assert_equal_blocks_with_txs, build_deploy_account_tx, build_oz_account_factory, create_account, AccountActions,
};
use starknet_rpc_test::{MadaraClient, Transaction};

#[rstest]
#[tokio::test]
async fn fail_non_existing_block(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    madara.create_empty_block().await?;

    assert_matches!(
        rpc.get_block_with_txs(BlockId::Hash(FieldElement::ZERO)).await.err(),
        Some(ProviderError::StarknetError(StarknetErrorWithMessage {
            message: _,
            code: MaybeUnknownErrorCode::Known(StarknetError::BlockNotFound)
        }))
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn works_with_invoke_txn(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    let account = create_account(rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
    let recipient = FieldElement::from_hex_be("0x1234").unwrap();
    madara
        .create_block_with_txs(vec![Transaction::Execution(account.transfer_tokens(
            recipient,
            FieldElement::ONE,
            None,
        ))])
        .await?;

    let block = match rpc.get_block_with_txs(BlockId::Tag(BlockTag::Latest)).await.unwrap() {
        MaybePendingBlockWithTxs::Block(block) => block,
        MaybePendingBlockWithTxs::PendingBlock(_) => return Err(anyhow!("Expected block, got pending block")),
    };

    assert_equal_blocks_with_txs(
        block.clone(),
        BlockWithTxs {
            status: BlockStatus::AcceptedOnL2,
            block_hash: FieldElement::from_hex_be("0x015e8bc7066c6d98d71c52bd52bb8eb0d1747eaa189c7f90a2a31045edccf2a8")
                .unwrap(),
            parent_hash: FieldElement::from_hex_be(
                "0x031ebd02657f940683ae7bddf19716932c56d463fc16662d14031f8635df52ad",
            )
            .unwrap(),
            block_number: 1,
            new_root: FieldElement::ZERO,
            sequencer_address: FieldElement::from_hex_be(
                "0x000000000000000000000000000000000000000000000000000000000000dead",
            )
            .unwrap(),
            timestamp: block.timestamp,
            transactions: vec![StarknetTransaction::Invoke(InvokeTransaction::V1(InvokeTransactionV1 {
                transaction_hash: FieldElement::from_hex_be(
                    "0x069d9d0ac1f5a4ad8d8e9a3954da53b5dc8ed239c02ad04492b9e15c50fe6d11",
                )
                .unwrap(),
                max_fee: FieldElement::from_hex_be(MAX_FEE_OVERRIDE).unwrap(),
                signature: vec![
                    FieldElement::from_hex_be("0x0611fcebbeffcbe80056f163dba051de342fbf139ece6071663a6f5d1100f4db")
                        .unwrap(),
                    FieldElement::from_hex_be("0x02c52a90217e781fd959fe961076d580c07b1bfb8e120576a55f2cb04c699a67")
                        .unwrap(),
                ],
                nonce: FieldElement::ZERO,
                sender_address: FieldElement::TWO,
                calldata: vec![
                    FieldElement::ONE,
                    FieldElement::from_hex_be(FEE_TOKEN_ADDRESS).unwrap(),
                    get_selector_from_name("transfer").unwrap(),
                    FieldElement::ZERO,
                    FieldElement::THREE,
                    FieldElement::THREE,
                    recipient,
                    FieldElement::ONE,
                    FieldElement::ZERO,
                ],
            }))],
        },
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn works_with_deploy_account_txn(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    let class_hash = FieldElement::from_hex_be(CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH).unwrap();
    let contract_address_salt = FieldElement::ONE;
    let max_fee = FieldElement::from_hex_be(MAX_FEE_OVERRIDE).unwrap();

    let oz_factory = build_oz_account_factory(rpc, "0x123", class_hash).await;
    let account_deploy_txn = build_deploy_account_tx(&oz_factory, FieldElement::ONE);

    let funding_account = create_account(rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
    let account_address = account_deploy_txn.address();

    madara
        .create_block_with_txs(vec![
            Transaction::Execution(funding_account.transfer_tokens(account_address, max_fee, None)),
            Transaction::AccountDeployment(account_deploy_txn),
        ])
        .await?;

    let block = match rpc.get_block_with_txs(BlockId::Tag(BlockTag::Latest)).await.unwrap() {
        MaybePendingBlockWithTxs::Block(block) => block,
        MaybePendingBlockWithTxs::PendingBlock(_) => return Err(anyhow!("Expected block, got pending block")),
    };

    assert_equal_blocks_with_txs(
        block.clone(),
        BlockWithTxs {
            status: BlockStatus::AcceptedOnL2,
            block_hash: FieldElement::from_hex_be("0x04d16ce836f8c4f15b30669313fd8b2e3d0118a6e9e5ee8a5de44b954056bdd8")
                .unwrap(),
            parent_hash: FieldElement::from_hex_be(
                "0x031ebd02657f940683ae7bddf19716932c56d463fc16662d14031f8635df52ad",
            )
            .unwrap(),
            block_number: 1,
            new_root: FieldElement::ZERO,
            sequencer_address: FieldElement::from_hex_be(
                "0x000000000000000000000000000000000000000000000000000000000000dead",
            )
            .unwrap(),
            timestamp: block.timestamp,
            transactions: vec![
                StarknetTransaction::Invoke(InvokeTransaction::V1(InvokeTransactionV1 {
                    transaction_hash: FieldElement::from_hex_be(
                        "0x03be8055eece65051368768a6b92ae51e1a228edb04ebbd269e3bab555c4ed0e",
                    )
                    .unwrap(),
                    max_fee: FieldElement::from_hex_be(MAX_FEE_OVERRIDE).unwrap(),
                    signature: vec![
                        FieldElement::from_hex_be("0x0676c246cb9d166ee69e20278767837e543a9982641d05e03ca3ea9bdb7629eb")
                            .unwrap(),
                        FieldElement::from_hex_be("0x066a8ee0282af011008df1a07bd30b20575b2a7b267a2ca5428eba7c8589b0ef")
                            .unwrap(),
                    ],
                    nonce: FieldElement::ZERO,
                    sender_address: FieldElement::TWO,
                    calldata: vec![
                        FieldElement::ONE,
                        FieldElement::from_hex_be(FEE_TOKEN_ADDRESS).unwrap(),
                        get_selector_from_name("transfer").unwrap(),
                        FieldElement::ZERO,
                        FieldElement::THREE,
                        FieldElement::THREE,
                        account_address,
                        max_fee, // transfer uses the same max_fee as the deploy txn internally
                        FieldElement::ZERO,
                    ],
                })),
                StarknetTransaction::DeployAccount(DeployAccountTransaction {
                    transaction_hash: FieldElement::from_hex_be(
                        "0x02105f08ba02511ccef6ff6676a1481645ec33c9e0d9f7d654b0590aa6afb013",
                    )
                    .unwrap(),
                    max_fee,
                    signature: vec![
                        FieldElement::from_hex_be("0x06bea565e0ac2450b1765ce3fec2ffd665f88b7c1c809a5713f795ab9641e133")
                            .unwrap(),
                        FieldElement::from_hex_be("0x00d8227bb300a313abb456689776dec594c2807b57824bf1159933e95946d227")
                            .unwrap(),
                    ],
                    nonce: FieldElement::ZERO,
                    contract_address_salt,
                    constructor_calldata: vec![
                        FieldElement::from_hex_be("0x0566d69d8c99f62bc71118399bab25c1f03719463eab8d6a444cd11ece131616")
                            .unwrap(),
                    ],
                    class_hash,
                }),
            ],
        },
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn works_with_declare_txn(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    let account = create_account(rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
    let (declare_tx, class_hash, compiled_class_hash) =
        account.declare_contract("./contracts/Counter.sierra.json", "./contracts/Counter.casm.json");

    // manually setting fee else estimate_fee will be called and it will fail
    // as the nonce has not been updated yet
    let max_fee = FieldElement::from_hex_be(MAX_FEE_OVERRIDE).unwrap();

    madara.create_block_with_txs(vec![Transaction::Declaration(declare_tx)]).await?;

    let block = match rpc.get_block_with_txs(BlockId::Tag(BlockTag::Latest)).await.unwrap() {
        MaybePendingBlockWithTxs::Block(block) => block,
        MaybePendingBlockWithTxs::PendingBlock(_) => return Err(anyhow!("Expected block, got pending block")),
    };

    assert_equal_blocks_with_txs(
        block.clone(),
        BlockWithTxs {
            status: BlockStatus::AcceptedOnL2,
            block_hash: FieldElement::from_hex_be("0x065e90b2a9571d961a874056372238922aeefc54984d78db15f7146797746a0b")
                .unwrap(),
            parent_hash: FieldElement::from_hex_be(
                "0x031ebd02657f940683ae7bddf19716932c56d463fc16662d14031f8635df52ad",
            )
            .unwrap(),
            block_number: 1,
            new_root: FieldElement::ZERO,
            sequencer_address: FieldElement::from_hex_be(
                "0x000000000000000000000000000000000000000000000000000000000000dead",
            )
            .unwrap(),
            timestamp: block.timestamp,
            transactions: vec![StarknetTransaction::Declare(DeclareTransaction::V2(DeclareTransactionV2 {
                transaction_hash: FieldElement::from_hex_be(
                    "0x05e0f64e8140019f2657f244dd9fd136d18acc6f52d8a0b85d3f84a110d4c708",
                )
                .unwrap(),
                max_fee,
                signature: vec![
                    FieldElement::from_hex_be("0x047a258d089e26d77f4dfcb87ad6e2537ca729c228bc75aeb9d2332cd525a25f")
                        .unwrap(),
                    FieldElement::from_hex_be("0x00b3ce21b372da9e878fd5730297589f22f7ad7a0d45520ef41602f001f90c5b")
                        .unwrap(),
                ],
                nonce: FieldElement::ZERO,
                sender_address: FieldElement::TWO,
                class_hash,
                compiled_class_hash,
            }))],
        },
    );

    Ok(())
}
