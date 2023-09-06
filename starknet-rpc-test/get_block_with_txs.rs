#![feature(assert_matches)]

extern crate starknet_rpc_test;

use std::assert_matches::assert_matches;

use anyhow::anyhow;
use rstest::rstest;
use starknet_accounts::{AccountFactory, OpenZeppelinAccountFactory};
use starknet_core::chain_id;
use starknet_core::types::{
    BlockId, BlockStatus, BlockTag, BlockWithTxs, DeclareTransaction, DeclareTransactionV1, DeclareTransactionV2,
    DeployAccountTransaction, InvokeTransaction, InvokeTransactionV1, MaybePendingBlockWithTxs, StarknetError,
    Transaction as StarknetTransaction,
};
use starknet_core::utils::get_selector_from_name;
use starknet_ff::FieldElement;
use starknet_providers::{MaybeUnknownErrorCode, Provider, ProviderError, StarknetErrorWithMessage};
use starknet_rpc_test::constants::{
    ARGENT_CONTRACT_ADDRESS, CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH, FEE_TOKEN_ADDRESS, MAX_FEE_OVERRIDE, SIGNER_PRIVATE,
};
use starknet_rpc_test::fixtures::madara;
use starknet_rpc_test::utils::{assert_equal_blocks_with_txs, create_account, AccountActions};
use starknet_rpc_test::{MadaraClient, Transaction};
use starknet_signers::{LocalWallet, SigningKey};

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
            block_hash: FieldElement::from_hex_be("0x078900eec31cb819620f277029089b8bf158cfb8b63e0332f03f57e0d48ce0c6")
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
                    "0x00581e60706c38d474ef27099f5e3f9506c63211340f7ca1849abe382c33123f",
                )
                .unwrap(),
                max_fee: FieldElement::from_hex_be(MAX_FEE_OVERRIDE).unwrap(),
                signature: vec![
                    FieldElement::from_hex_be("0x053804f9408e2487cc3f8c9ab5fdce261ed9bc43c95630be6ed9f276803ecb90")
                        .unwrap(),
                    FieldElement::from_hex_be("0x02823c06c85eaef396f64ac459cc063f026be9dd0b38edd5942566ecc8e3ab63")
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

    let signer = LocalWallet::from(SigningKey::from_secret_scalar(FieldElement::from_hex_be("0x123").unwrap()));

    let class_hash = FieldElement::from_hex_be(CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH).unwrap();
    let contract_address_salt = FieldElement::ONE;

    let oz_factory = OpenZeppelinAccountFactory::new(class_hash, chain_id::TESTNET, signer, rpc).await.unwrap();

    let max_fee = FieldElement::from_hex_be(MAX_FEE_OVERRIDE).unwrap();
    let account_deploy_txn = oz_factory.deploy(contract_address_salt).max_fee(max_fee);

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
            block_hash: FieldElement::from_hex_be("0x05fb29856b6e0afe6a887453a3f68a9fdb8c0889db90aedfa76fb10d910cd1b2")
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
                        "0x0770319fa9fda65e97216fac7cde986406874518deb2337e7f60ea91daa49611",
                    )
                    .unwrap(),
                    max_fee: FieldElement::from_hex_be(MAX_FEE_OVERRIDE).unwrap(),
                    signature: vec![
                        FieldElement::from_hex_be("0x031adb83ec6f5b559f1195f3f4d2460976ee5e1a0b1cc28acee3ae18f4bca245")
                            .unwrap(),
                        FieldElement::from_hex_be("0x011fa58c396b737a68d9daf22a2dc6492ef0fa30fe2365a433176f28628b84d9")
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
                        "0x03569747fea4ad0c6e2d16ac69d353057f2d001229db8968533286c684e1a84a",
                    )
                    .unwrap(),
                    max_fee,
                    signature: vec![
                        FieldElement::from_hex_be("0x05600ddda0366a47b8e060602133980bccf435f58fd15b0cce43e62b204a1b07")
                            .unwrap(),
                        FieldElement::from_hex_be("0x01b145ab62e5c88d126396ef337dbd97d48c91374cc8a06eb5458479ccc86a6a")
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
    let (mut declare_tx_legacy, class_hash_legacy) = account.declare_legacy_contract("./contracts/ERC20.json");

    // manually setting fee else estimate_fee will be called and it will fail
    // as the nonce has not been updated yet
    let max_fee = FieldElement::from_hex_be(MAX_FEE_OVERRIDE).unwrap();

    // manually incrementing nonce else as fetching nonce will get 0 as
    // the both txns are in the same block
    let nonce = FieldElement::ONE;

    declare_tx_legacy = declare_tx_legacy.nonce(nonce).max_fee(max_fee);
    madara
        .create_block_with_txs(vec![
            Transaction::Declaration(declare_tx),
            Transaction::LegacyDeclaration(declare_tx_legacy),
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
            block_hash: FieldElement::from_hex_be("0x001624a3818c71653b975c9c8c89ea670d3ab7f863e9c02b697c7fe4f55470ad")
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
                StarknetTransaction::Declare(DeclareTransaction::V2(DeclareTransactionV2 {
                    transaction_hash: FieldElement::from_hex_be(
                        "0x01fc4c0d8f82edfd74ef83c5db42203fe4a70243a76e88e0a4a6ade9753d8ec9",
                    )
                    .unwrap(),
                    max_fee,
                    signature: vec![
                        FieldElement::from_hex_be("0x06c874338b748868b555ad7f9bf1e362d3d23b6e900ef0065a76eebbe1294438")
                            .unwrap(),
                        FieldElement::from_hex_be("0x025860a19f5bb89068408f12356bb5ff908fd4cb73238feac632bfa880367c5a")
                            .unwrap(),
                    ],
                    nonce: FieldElement::ZERO,
                    sender_address: FieldElement::TWO,
                    class_hash,
                    compiled_class_hash,
                })),
                StarknetTransaction::Declare(DeclareTransaction::V1(DeclareTransactionV1 {
                    transaction_hash: FieldElement::from_hex_be(
                        "0x02c99c0d47dc755f95ea3c0bb92e77dee17bcda42ed2ebba438c8762a1d897c5",
                    )
                    .unwrap(),
                    max_fee,
                    signature: vec![
                        FieldElement::from_hex_be("0x013059485d6e89d6fd69874d742bb2902da853b6cdb48cf73869c7c1be734002")
                            .unwrap(),
                        FieldElement::from_hex_be("0x03e7cb10ebc5d567139ea93e2050824a53fb30f9e718ea06705c7cb7523c9d95")
                            .unwrap(),
                    ],
                    nonce,
                    sender_address: FieldElement::TWO,
                    class_hash: class_hash_legacy,
                })),
            ],
        },
    );

    Ok(())
}
