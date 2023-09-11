extern crate starknet_rpc_test;

use assert_matches::assert_matches;
use rstest::rstest;
use starknet_core::types::StarknetError;
use starknet_ff::FieldElement;
use starknet_providers::{MaybeUnknownErrorCode, Provider, ProviderError, StarknetErrorWithMessage};
use starknet_rpc_test::constants::{ARGENT_CONTRACT_ADDRESS, SIGNER_PRIVATE};
use starknet_rpc_test::fixtures::madara;
use starknet_rpc_test::utils::{assert_poll, create_account, AccountActions};
use starknet_rpc_test::{MadaraClient, Transaction, TransactionResult};

#[rstest]
#[tokio::test]
async fn work_valid_transaction_hash(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    let account = create_account(rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
    let mut txs = madara
        .create_block_with_txs(vec![Transaction::Execution(account.transfer_tokens(
            FieldElement::from_hex_be("0x123").unwrap(),
            FieldElement::ONE,
            None,
        ))])
        .await?;

    assert_eq!(txs.len(), 1);

    let rpc_response = match txs.remove(0).unwrap() {
        TransactionResult::Execution(rpc_response) => rpc_response,
        _ => panic!("expected execution result"),
    };

    // 1. There is a delay between the transaction being available at the client
    // and the sealing of the block, hence sleeping for 100ms
    // 2. Not validating the fields inside the transaction as
    // that is covered in get_block_with_txs
    assert_poll(|| async { rpc.get_transaction_by_hash(rpc_response.transaction_hash).await.is_ok() }, 100, 20).await;

    Ok(())
}

#[rstest]
#[tokio::test]
async fn fail_invalid_transaction_hash(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    assert_matches!(
        rpc.get_transaction_by_hash(FieldElement::from_hex_be("0x123").unwrap()).await,
        Err(ProviderError::StarknetError(StarknetErrorWithMessage {
            code: MaybeUnknownErrorCode::Known(StarknetError::TransactionHashNotFound),
            message: _
        }))
    );

    Ok(())
}
