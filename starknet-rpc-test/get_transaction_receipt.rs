extern crate starknet_rpc_test;

use assert_matches::assert_matches;
use rstest::rstest;
use starknet_core::types::{
    DeclareTransactionReceipt, Event, ExecutionResult, MaybePendingTransactionReceipt, TransactionFinalityStatus,
    TransactionReceipt,
};
use starknet_core::utils::get_selector_from_name;
use starknet_ff::FieldElement;
use starknet_providers::jsonrpc::{HttpTransport, HttpTransportError, JsonRpcClientError};
use starknet_providers::{JsonRpcClient, Provider, ProviderError};
use starknet_rpc_test::constants::{
    ARGENT_CONTRACT_ADDRESS, CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH, FEE_TOKEN_ADDRESS, SEQUENCER_ADDRESS, SIGNER_PRIVATE,
};
use starknet_rpc_test::fixtures::{madara, ThreadSafeMadaraClient};
use starknet_rpc_test::utils::{
    assert_eq_event, assert_eq_msg_to_l1, assert_poll, build_deploy_account_tx, build_oz_account_factory,
    build_single_owner_account, AccountActions,
};
use starknet_rpc_test::{Transaction, TransactionResult};

type TransactionReceiptResult =
    Result<MaybePendingTransactionReceipt, ProviderError<JsonRpcClientError<HttpTransportError>>>;

async fn get_transaction_receipt(
    rpc: &JsonRpcClient<HttpTransport>,
    transaction_hash: FieldElement,
) -> TransactionReceiptResult {
    // there is a delay between the transaction being available at the client
    // and the sealing of the block, hence sleeping for 100ms
    assert_poll(|| async { rpc.get_transaction_receipt(transaction_hash).await.is_ok() }, 100, 20).await;

    rpc.get_transaction_receipt(transaction_hash).await
}

#[rstest]
#[tokio::test]
async fn work_with_invoke_transaction(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let recepient = FieldElement::from_hex_be("0x123").unwrap();
    let transfer_amount = FieldElement::ONE;

    let mut txs = {
        let mut madara_write_lock = madara.write().await;
        let account = build_single_owner_account(&rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
        madara_write_lock
            .create_block_with_txs(vec![Transaction::Execution(account.transfer_tokens(
                recepient,
                transfer_amount,
                None,
            ))])
            .await?
    };

    assert_eq!(txs.len(), 1);
    let rpc_response = match txs.remove(0).unwrap() {
        TransactionResult::Execution(rpc_response) => rpc_response,
        _ => panic!("expected execution result"),
    };

    let invoke_tx_receipt = get_transaction_receipt(&rpc, rpc_response.transaction_hash).await;
    let fee_token_address = FieldElement::from_hex_be(FEE_TOKEN_ADDRESS).unwrap();
    let expected_fee = FieldElement::from_hex_be("0xf154").unwrap();

    match invoke_tx_receipt {
        Ok(MaybePendingTransactionReceipt::Receipt(TransactionReceipt::Invoke(receipt))) => {
            assert_eq!(receipt.transaction_hash, rpc_response.transaction_hash);
            // assert_eq!(receipt.actual_fee, expected_fee); TODO: Fix in RPC
            assert_eq!(receipt.finality_status, TransactionFinalityStatus::AcceptedOnL2);
            assert_eq_msg_to_l1(receipt.messages_sent, vec![]);
            assert_eq_event(
                receipt.events,
                vec![
                    Event {
                        from_address: fee_token_address,
                        keys: vec![get_selector_from_name("Transfer").unwrap()],
                        data: vec![
                            FieldElement::from_hex_be(ARGENT_CONTRACT_ADDRESS).unwrap(), // from
                            recepient,                                                   // to
                            transfer_amount,                                             // value low
                            FieldElement::ZERO,                                          // value high
                        ],
                    },
                    Event {
                        from_address: FieldElement::from_hex_be(ARGENT_CONTRACT_ADDRESS).unwrap(), // from
                        keys: vec![get_selector_from_name("transaction_executed").unwrap()],
                        data: vec![
                            rpc_response.transaction_hash, // txn hash
                            FieldElement::TWO,             // response_len
                            FieldElement::ONE,
                            FieldElement::ONE,
                        ],
                    },
                    Event {
                        from_address: fee_token_address,
                        keys: vec![get_selector_from_name("Transfer").unwrap()],
                        data: vec![
                            FieldElement::from_hex_be(ARGENT_CONTRACT_ADDRESS).unwrap(), // from
                            FieldElement::from_hex_be(SEQUENCER_ADDRESS).unwrap(),       // to (sequencer address)
                            expected_fee,                                                // value low
                            FieldElement::ZERO,                                          // value high
                        ],
                    },
                ],
            );
            assert_matches!(receipt.execution_result, ExecutionResult::Succeeded);
        }
        _ => panic!("expected invoke transaction receipt"),
    };

    Ok(())
}

#[rstest]
#[tokio::test]
#[ignore = "class already declared"]
async fn work_with_declare_transaction(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let mut txs = {
        let mut madara_write_lock = madara.write().await;
        let account = build_single_owner_account(&rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
        let (declare_tx, _, _) =
            account.declare_contract("./contracts/Counter.sierra.json", "./contracts/Counter.casm.json");

        madara_write_lock.create_block_with_txs(vec![Transaction::Declaration(declare_tx)]).await?
    };

    let rpc_response_declare = match txs.remove(0).unwrap() {
        TransactionResult::Declaration(rpc_response) => rpc_response,
        _ => panic!("expected execution result"),
    };

    // not validating the fields inside the transaction as
    // that is covered in get_block_with_txs
    let declare_tx_receipt = get_transaction_receipt(&rpc, rpc_response_declare.transaction_hash).await;

    let assert_declare_tx_receipt = |d1: TransactionReceiptResult, d2: DeclareTransactionReceipt| {
        let d1 = match d1 {
            Ok(MaybePendingTransactionReceipt::Receipt(TransactionReceipt::Declare(d1))) => d1,
            _ => panic!("expected declare transaction receipt"),
        };
        assert_eq!(d1.transaction_hash, d2.transaction_hash);
        // assert_eq!(d1.actual_fee, d2.actual_fee); TODO: Fix in rpc
        assert_eq!(d1.finality_status, d2.finality_status);
        assert_eq!(d1.block_hash, d2.block_hash);
        assert_eq!(d1.block_number, d2.block_number);
        assert_eq_msg_to_l1(d1.messages_sent, d2.messages_sent);
        assert_eq_event(d1.events, d2.events);
        // assert_matches does not accept d2.execution_result on the RHS
        assert_matches!(d1.execution_result, ExecutionResult::Succeeded);
        assert_matches!(d2.execution_result, ExecutionResult::Succeeded);
    };

    let fee_token_address = FieldElement::from_hex_be(FEE_TOKEN_ADDRESS).unwrap();
    let expected_fee =
        FieldElement::from_hex_be("0x000000000000000000000000000000000000000000000000000000000000d3ae").unwrap();

    assert_declare_tx_receipt(
        declare_tx_receipt,
        DeclareTransactionReceipt {
            transaction_hash: FieldElement::from_hex_be(
                "0x05e0f64e8140019f2657f244dd9fd136d18acc6f52d8a0b85d3f84a110d4c708",
            )
            .unwrap(),
            actual_fee: expected_fee,
            finality_status: TransactionFinalityStatus::AcceptedOnL2,
            block_hash: FieldElement::from_hex_be("0x031622c96d67dabe52c0317752d6e6be69a4288e6dcec09a6f8324bee49d4ce5")
                .unwrap(),
            block_number: 1,
            messages_sent: vec![],
            events: vec![Event {
                from_address: fee_token_address,
                keys: vec![get_selector_from_name("Transfer").unwrap()],
                data: vec![
                    FieldElement::from_hex_be(ARGENT_CONTRACT_ADDRESS).unwrap(), // to (sequencer address)
                    FieldElement::from_hex_be(SEQUENCER_ADDRESS).unwrap(),       // to (sequencer address)
                    expected_fee,                                                // value low
                    FieldElement::ZERO,                                          // value high
                ],
            }],
            execution_result: ExecutionResult::Succeeded,
        },
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn work_with_deploy_account_transaction(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let (mut txs, account_address) = {
        let mut madara_write_lock = madara.write().await;
        let oz_factory = build_oz_account_factory(
            &rpc,
            "0x456",
            FieldElement::from_hex_be(CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH).unwrap(),
        )
        .await;
        let account_deploy_txn = build_deploy_account_tx(&oz_factory, FieldElement::ONE);
        let account_address = account_deploy_txn.address();

        // add funds to deploy account
        let funding_account = build_single_owner_account(&rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
        madara_write_lock
            .create_block_with_txs(vec![Transaction::Execution(funding_account.transfer_tokens(
                account_address,
                FieldElement::from_hex_be("0x100000").unwrap(),
                None,
            ))])
            .await?;

        let txs =
            madara_write_lock.create_block_with_txs(vec![Transaction::AccountDeployment(account_deploy_txn)]).await?;

        (txs, account_address)
    };

    assert_eq!(txs.len(), 1);
    let rpc_response = match txs.remove(0).unwrap() {
        TransactionResult::AccountDeployment(rpc_response) => rpc_response,
        _ => panic!("expected execution result"),
    };

    let account_deployment_tx_receipt = get_transaction_receipt(&rpc, rpc_response.transaction_hash).await;
    let fee_token_address = FieldElement::from_hex_be(FEE_TOKEN_ADDRESS).unwrap();
    let expected_fee = FieldElement::from_hex_be("0x790e").unwrap();

    match account_deployment_tx_receipt {
        Ok(MaybePendingTransactionReceipt::Receipt(TransactionReceipt::DeployAccount(receipt))) => {
            assert_eq!(receipt.transaction_hash, rpc_response.transaction_hash);
            // assert_eq!(receipt.actual_fee, expected_fee); TODO: fix in code
            assert_eq!(receipt.finality_status, TransactionFinalityStatus::AcceptedOnL2);
            assert_eq_msg_to_l1(receipt.messages_sent, vec![]);
            assert_eq_event(
                receipt.events,
                vec![Event {
                    from_address: fee_token_address,
                    keys: vec![get_selector_from_name("Transfer").unwrap()],
                    data: vec![
                        account_address,
                        FieldElement::from_hex_be(SEQUENCER_ADDRESS).unwrap(), // to
                        expected_fee,                                          // value low
                        FieldElement::ZERO,                                    // value high
                    ],
                }],
            );
            assert_matches!(receipt.execution_result, ExecutionResult::Succeeded);
            assert_eq!(receipt.contract_address, FieldElement::ZERO);
        }
        _ => panic!("expected deploy account transaction receipt"),
    };

    Ok(())
}

#[rstest]
#[tokio::test]
async fn fail_invalid_transaction_hash(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    assert!(rpc.get_transaction_receipt(FieldElement::ZERO).await.is_err());

    Ok(())
}
