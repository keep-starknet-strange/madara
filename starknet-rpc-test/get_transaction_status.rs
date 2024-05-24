use std::panic;

use assert_matches::assert_matches;
use rstest::rstest;
use starknet_core::types::{StarknetError, TransactionExecutionStatus, TransactionStatus};
use starknet_ff::FieldElement;
use starknet_providers::{Provider, ProviderError};
use starknet_rpc_test::constants::{ARGENT_CONTRACT_ADDRESS, SIGNER_PRIVATE};
use starknet_rpc_test::fixtures::{madara, ThreadSafeMadaraClient};
use starknet_rpc_test::utils::{assert_poll, build_single_owner_account, AccountActions};
use starknet_rpc_test::{Transaction, TransactionResult};

#[rstest]
#[tokio::test]
async fn work_with_valid_transaction_hash(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let mut madara_write_lock = madara.write().await;
    let account = build_single_owner_account(&rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);

    let mut txs = madara_write_lock
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

    // There is a delay between the transaction being available at the client
    // and the sealing of the block, hence sleeping for 1000ms and repeat 20 times
    assert_poll(
        || async {
            let result = rpc.get_transaction_status(rpc_response.transaction_hash).await;
            match result {
                Ok(TransactionStatus::AcceptedOnL2(TransactionExecutionStatus::Succeeded)) => true,
                _ => false,
            }
        },
        1000,
        20,
    )
    .await;

    Ok(())
}

#[rstest]
#[tokio::test]
async fn fail_with_invalid_transaction_hash(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    assert_matches!(
        rpc.get_transaction_status(FieldElement::from_hex_be("0x123").unwrap()).await,
        Err(ProviderError::StarknetError(StarknetError::TransactionHashNotFound))
    );

    Ok(())
}
