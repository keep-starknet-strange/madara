extern crate starknet_rpc_test;

use assert_matches::assert_matches;
use starknet_accounts::SingleOwnerAccount;
use starknet_core::chain_id;
use starknet_core::types::{
    BlockId, BlockTag, DeclareTransaction, InvokeTransaction, MaybePendingBlockWithTxs, StarknetError, Transaction,
};
use starknet_ff::FieldElement;
use starknet_providers::ProviderError::StarknetError as StarknetProviderError;
use starknet_providers::{MaybeUnknownErrorCode, Provider, StarknetErrorWithMessage};
use starknet_rpc_test::constants::{ARGENT_CONTRACT_ADDRESS, MINT_AMOUNT, SIGNER_PRIVATE, TEST_CONTRACT_CLASS_HASH};
use starknet_rpc_test::utils::AccountActions;
use starknet_rpc_test::{ExecutionStrategy, MadaraClient};
use starknet_signers::{LocalWallet, SigningKey};

#[tokio::test]
async fn fail_non_existing_block() -> Result<(), anyhow::Error> {
    let madara = MadaraClient::new(ExecutionStrategy::Native).await;
    let rpc = madara.get_starknet_client();

    assert_matches!(
        rpc
        .get_transaction_by_block_id_and_index(
            BlockId::Number(1),
            0
        )
        .await,
        Err(StarknetProviderError(StarknetErrorWithMessage { code:
MaybeUnknownErrorCode::Known(code), .. })) if code == StarknetError::BlockNotFound     );

    Ok(())
}

#[tokio::test]
async fn fail_out_of_block_index() -> Result<(), anyhow::Error> {
    let madara = MadaraClient::new(ExecutionStrategy::Native).await;
    let rpc = madara.get_starknet_client();

    assert_matches!(
            rpc
            .get_transaction_by_block_id_and_index(
                BlockId::Tag(BlockTag::Latest),
                0
            )
            .await,
            Err(StarknetProviderError(StarknetErrorWithMessage { code:
    MaybeUnknownErrorCode::Known(code), .. })) if code == StarknetError::InvalidTransactionIndex
        );

    Ok(())
}

#[tokio::test]
async fn work_ok_by_compare_with_get_block_with_tx() -> Result<(), anyhow::Error> {
    let madara = MadaraClient::new(ExecutionStrategy::Native).await;
    let rpc = madara.get_starknet_client();

    let signer = LocalWallet::from(SigningKey::from_secret_scalar(FieldElement::from_hex_be(SIGNER_PRIVATE).unwrap()));
    let argent_account_address = FieldElement::from_hex_be(ARGENT_CONTRACT_ADDRESS).expect("Invalid Contract Address");
    let account = SingleOwnerAccount::new(rpc, signer, argent_account_address, chain_id::TESTNET);

    madara.create_empty_block().await?;

    let execution_1 = account.transfer_tokens(
        argent_account_address,
        FieldElement::from_hex_be(MINT_AMOUNT).expect("Invalid Mint Amount"),
        None,
    );

    let execution_2 = account.transfer_tokens(
        FieldElement::from_hex_be(TEST_CONTRACT_CLASS_HASH).expect("Invalid Contract Address"),
        FieldElement::from_hex_be(MINT_AMOUNT).expect("Invalid Mint Amount"),
        None,
    );
    let execution_2 = execution_2.nonce(FieldElement::ONE);
    let execution_2 = execution_2.max_fee(FieldElement::from_hex_be("0xDEADB").expect("Invalid Fee Amount"));

    madara.create_block_with_txs(vec![execution_1, execution_2]).await?;

    let tx_1 = rpc.get_transaction_by_block_id_and_index(BlockId::Tag(BlockTag::Latest), 0).await?;
    let tx_2 = rpc.get_transaction_by_block_id_and_index(BlockId::Tag(BlockTag::Latest), 1).await?;

    let block_with_txs = rpc.get_block_with_txs(BlockId::Tag(BlockTag::Latest)).await?;

    assert_eq!(get_transaction_hash(&tx_1), get_transaction_hash_from_block_with_txs(&block_with_txs, 0));
    assert_eq!(get_transaction_hash(&tx_2), get_transaction_hash_from_block_with_txs(&block_with_txs, 1));

    Ok(())
}

fn get_transaction_hash(tx: &Transaction) -> Option<&FieldElement> {
    match tx {
        Transaction::Invoke(InvokeTransaction::V0(v0_tx)) => Some(&v0_tx.transaction_hash),
        Transaction::L1Handler(l1_handler_tx) => Some(&l1_handler_tx.transaction_hash),
        Transaction::Declare(DeclareTransaction::V0(v0_tx)) => Some(&v0_tx.transaction_hash),
        Transaction::Declare(DeclareTransaction::V1(v1_tx)) => Some(&v1_tx.transaction_hash),
        Transaction::Declare(DeclareTransaction::V2(v2_tx)) => Some(&v2_tx.transaction_hash),
        Transaction::Invoke(InvokeTransaction::V1(v1_tx)) => Some(&v1_tx.transaction_hash),
        Transaction::Deploy(deploy_tx) => Some(&deploy_tx.transaction_hash),
        Transaction::DeployAccount(deploy_account_tx) => Some(&deploy_account_tx.transaction_hash),
    }
}

fn get_transaction_hash_from_block_with_txs(
    block_with_txs: &MaybePendingBlockWithTxs,
    index: usize,
) -> Option<&FieldElement> {
    match block_with_txs {
        MaybePendingBlockWithTxs::Block(b) => get_transaction_hash(&b.transactions[index]),
        MaybePendingBlockWithTxs::PendingBlock(pb) => get_transaction_hash(&pb.transactions[index]),
    }
}
