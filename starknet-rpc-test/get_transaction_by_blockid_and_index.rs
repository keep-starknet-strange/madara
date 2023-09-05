extern crate starknet_rpc_test;

use assert_matches::assert_matches;
use rstest::rstest;
use starknet_accounts::SingleOwnerAccount;
use starknet_core::chain_id;
use starknet_core::types::{
    BlockId, BlockTag, InvokeTransaction, InvokeTransactionV1, MaybePendingBlockWithTxs, StarknetError, Transaction,
};
use starknet_ff::FieldElement;
use starknet_providers::ProviderError::StarknetError as StarknetProviderError;
use starknet_providers::{MaybeUnknownErrorCode, Provider, StarknetErrorWithMessage};
use starknet_rpc_test::constants::{ARGENT_CONTRACT_ADDRESS, MINT_AMOUNT, SIGNER_PRIVATE, TEST_CONTRACT_CLASS_HASH};
use starknet_rpc_test::fixtures::madara;
use starknet_rpc_test::utils::AccountActions;
use starknet_rpc_test::{MadaraClient, Transaction as TransactionEnum};
use starknet_signers::{LocalWallet, SigningKey};

#[rstest]
#[tokio::test]
async fn fail_non_existing_block(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    assert_matches!(
        rpc.get_transaction_by_block_id_and_index(BlockId::Number(1), 0).await,
        Err(StarknetProviderError(StarknetErrorWithMessage {
            code: MaybeUnknownErrorCode::Known(StarknetError::BlockNotFound),
            ..
        }))
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn fail_out_of_block_index(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    assert_matches!(
        rpc.get_transaction_by_block_id_and_index(BlockId::Tag(BlockTag::Latest), 0).await,
        Err(StarknetProviderError(StarknetErrorWithMessage {
            code: MaybeUnknownErrorCode::Known(StarknetError::InvalidTransactionIndex),
            ..
        }))
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn work_ok_by_compare_with_get_block_with_tx(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
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

    let execution_2 = account
        .transfer_tokens(
            FieldElement::from_hex_be(TEST_CONTRACT_CLASS_HASH).expect("Invalid Contract Address"),
            FieldElement::from_hex_be(MINT_AMOUNT).expect("Invalid Mint Amount"),
            None,
        )
        .nonce(FieldElement::ONE)
        .max_fee(FieldElement::from_hex_be("0xDEADB").unwrap());

    madara
        .create_block_with_txs(vec![TransactionEnum::Execution(execution_1), TransactionEnum::Execution(execution_2)])
        .await?;

    let tx_1 = rpc.get_transaction_by_block_id_and_index(BlockId::Tag(BlockTag::Latest), 0).await?;
    let tx_2 = rpc.get_transaction_by_block_id_and_index(BlockId::Tag(BlockTag::Latest), 1).await?;

    let tx_1_hash = assert_matches!(tx_1, Transaction::Invoke(InvokeTransaction::V1(InvokeTransactionV1 {
        nonce,
        sender_address,
        transaction_hash,
        ..
     })) if nonce == FieldElement::ZERO 
            && sender_address == argent_account_address 
            => transaction_hash);

    let tx_2_hash = assert_matches!(tx_2, Transaction::Invoke(InvokeTransaction::V1(InvokeTransactionV1 {
        nonce,
        sender_address,
        max_fee,
        transaction_hash,
        ..
        })) if nonce == FieldElement::ONE 
            && sender_address == argent_account_address
            && max_fee == FieldElement::from_hex_be("0xDEADB").unwrap() 
            => transaction_hash);

    let block_with_txs = rpc.get_block_with_txs(BlockId::Tag(BlockTag::Latest)).await?;

    assert_matches!(get_transaction_from_block_with_txs(&block_with_txs, 0), Transaction::Invoke(InvokeTransaction::V1(InvokeTransactionV1 {
        nonce,
        sender_address,
        transaction_hash,
        ..
        })) if nonce == &FieldElement::ZERO 
            && sender_address == &argent_account_address
            && transaction_hash == &tx_1_hash);

    assert_matches!(get_transaction_from_block_with_txs(&block_with_txs, 1), Transaction::Invoke(InvokeTransaction::V1(InvokeTransactionV1 {
        nonce,
        sender_address,
        max_fee,
        transaction_hash,
        ..
        })) if nonce == &FieldElement::ONE
            && sender_address == &argent_account_address
            && max_fee == &FieldElement::from_hex_be("0xDEADB").unwrap()
            && transaction_hash == &tx_2_hash);

    Ok(())
}

fn get_transaction_from_block_with_txs(block_with_txs: &MaybePendingBlockWithTxs, index: usize) -> &Transaction {
    match block_with_txs {
        MaybePendingBlockWithTxs::Block(b) => &b.transactions[index],
        MaybePendingBlockWithTxs::PendingBlock(pb) => &pb.transactions[index],
    }
}
