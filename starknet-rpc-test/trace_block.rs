extern crate starknet_rpc_test;

use assert_matches::assert_matches;
use rstest::rstest;
use starknet_core::types::{
    BlockId, CallType, EntryPointType, ExecuteInvocation, FunctionInvocation, InvokeTransactionTrace, StarknetError,
    TransactionTrace, TransactionTraceWithHash,
};
use starknet_ff::FieldElement;
use starknet_providers::ProviderError::StarknetError as StarknetProviderError;
use starknet_providers::{MaybeUnknownErrorCode, Provider, StarknetErrorWithMessage};
use starknet_rpc_test::constants::{
    ARGENT_ACCOUNT_CLASS_HASH_CAIRO_0, ARGENT_CONTRACT_ADDRESS, FEE_TOKEN_ADDRESS, SIGNER_PRIVATE,
};
use starknet_rpc_test::fixtures::{madara, ThreadSafeMadaraClient};
use starknet_rpc_test::utils::{build_single_owner_account, AccountActions};
use starknet_rpc_test::Transaction;

#[rstest]
#[tokio::test]
async fn fail_non_existing_block(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    assert_matches!(
        rpc.trace_block_transactions(BlockId::Hash(FieldElement::ZERO)).await,
        Err(StarknetProviderError(StarknetErrorWithMessage { code: MaybeUnknownErrorCode::Known(code), .. })) if code == StarknetError::BlockNotFound
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn work_for_one_invoke_tx(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    // Create a block with one tx
    // Copy-pasted from add_invoke_transaction::work_with_storage_change
    let funding_account = build_single_owner_account(&rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
    let recipient_account = FieldElement::from_hex_be("0x123").unwrap();

    let block_number = {
        let mut madara_write_lock = madara.write().await;

        // Simulation at block 0 will fail for a reason I don't understand
        // I think simulation may happen on the state before the genesis logic is executed
        // meaning accounts don't have any funds yet
        madara_write_lock.create_empty_block().await.unwrap();
        madara_write_lock
            .create_block_with_txs(vec![Transaction::Execution(funding_account.transfer_tokens(
                recipient_account,
                FieldElement::ONE,
                None,
            ))])
            .await?;

        rpc.block_number().await?
    };

    // included in block
    let included_txs = rpc.get_block_transaction_count(BlockId::Number(block_number)).await?;
    assert_eq!(included_txs, 1);

    let traces = rpc.trace_block_transactions(BlockId::Number(block_number)).await?;

    // starkli selector __execute__
    let execute_selector =
        FieldElement::from_hex_be("0x15d40a3d6ca2ac30f4031e42be28da9b056fef9bb7357ac5e85627ee876e5ad").unwrap();

    // starkli selector transfer
    let transfer_selector =
        FieldElement::from_hex_be("0x0083afd3f4caedc6eebf44246fe54e38c95e3179a5ec9ea81740eca5b482d12e").unwrap();
    // This is legacy starknet `__execute__` calls encoding
    let expected_calldata = vec![
        FieldElement::ONE,                                     // number of calls
        FieldElement::from_hex_be(FEE_TOKEN_ADDRESS).unwrap(), // contract to call
        transfer_selector,                                     // selector
        FieldElement::ZERO,                                    // data offset
        FieldElement::from(3u8),                               // data len
        FieldElement::from(3u8),                               // Calldata len
        recipient_account,                                     // Recipient address
        FieldElement::ONE,                                     // Amount low
        FieldElement::ZERO,                                    // Amount high
    ];

    assert_eq!(traces.len(), 1);
    assert_matches!(
        &traces[0],
        TransactionTraceWithHash {
            transaction_hash: _,
            trace_root: TransactionTrace::Invoke(InvokeTransactionTrace {
                execute_invocation: ExecuteInvocation::Success(FunctionInvocation { contract_address, class_hash, entry_point_type, call_type, entry_point_selector, calldata, .. }),
                ..
            })
        } if *contract_address == FieldElement::from_hex_be(ARGENT_CONTRACT_ADDRESS).unwrap()
            && *class_hash == FieldElement::from_hex_be(ARGENT_ACCOUNT_CLASS_HASH_CAIRO_0).unwrap()
            && *entry_point_type == EntryPointType::External
            && *call_type == CallType::Call
            && *entry_point_selector == execute_selector
            && *calldata == expected_calldata
    );

    Ok(())
}
