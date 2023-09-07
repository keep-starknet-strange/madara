extern crate starknet_rpc_test;

use assert_matches::assert_matches;
use rstest::rstest;
use starknet_accounts::Account;
use starknet_core::types::{
    DeclareTransactionReceipt, Event, ExecutionResult, MaybePendingTransactionReceipt, TransactionFinalityStatus,
    TransactionReceipt,
};
use starknet_core::utils::get_selector_from_name;
use starknet_ff::FieldElement;
use starknet_providers::jsonrpc::{HttpTransport, HttpTransportError, JsonRpcClientError};
use starknet_providers::{JsonRpcClient, Provider, ProviderError};
use starknet_rpc_test::constants::{
    ARGENT_CONTRACT_ADDRESS, CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH, FEE_TOKEN_ADDRESS, MAX_FEE_OVERRIDE,
    SEQUENCER_ADDRESS, SIGNER_PRIVATE,
};
use starknet_rpc_test::fixtures::madara;
use starknet_rpc_test::utils::{
    assert_eq_event, assert_eq_msg_to_l1, build_deploy_account_tx, build_oz_account_factory, create_account,
    AccountActions,
};
use starknet_rpc_test::{MadaraClient, Transaction, TransactionResult};

type TransactionReceiptResult =
    Result<MaybePendingTransactionReceipt, ProviderError<JsonRpcClientError<HttpTransportError>>>;

async fn get_transaction_receipt(
    rpc: &JsonRpcClient<HttpTransport>,
    transaction_hash: FieldElement,
) -> TransactionReceiptResult {
    // there is a delay between the transaction being available at the client
    // and the sealing of the block, hence sleeping for 100ms
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    rpc.get_transaction_receipt(transaction_hash).await
}

#[rstest]
#[tokio::test]
async fn work_with_invoke_transaction(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    let account = create_account(rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
    let recepient = FieldElement::from_hex_be("0x123").unwrap();
    let transfer_amount = FieldElement::ONE;
    let txs = madara
        .create_block_with_txs(vec![Transaction::Execution(account.transfer_tokens(recepient, transfer_amount, None))])
        .await;

    let rpc_response = match txs.unwrap().remove(0).unwrap() {
        TransactionResult::Execution(rpc_response) => rpc_response,
        _ => panic!("expected execution result"),
    };

    let invoke_tx_receipt = get_transaction_receipt(rpc, rpc_response.transaction_hash).await;
    let fee_token_address = FieldElement::from_hex_be(FEE_TOKEN_ADDRESS).unwrap();
    let expected_fee = FieldElement::from_hex_be("0x1ffe0").unwrap();

    match invoke_tx_receipt {
        Ok(MaybePendingTransactionReceipt::Receipt(TransactionReceipt::Invoke(receipt))) => {
            assert_eq!(receipt.transaction_hash, rpc_response.transaction_hash);
            assert_eq!(receipt.actual_fee, expected_fee);
            assert_eq!(receipt.finality_status, TransactionFinalityStatus::AcceptedOnL2);
            assert_eq!(
                receipt.block_hash,
                FieldElement::from_hex_be("0x06da61828bd573bb29d57a8dbc410684db35a934f90400d559812230c481849e")
                    .unwrap()
            );
            assert_eq!(receipt.block_number, 1);
            assert_eq_msg_to_l1(receipt.messages_sent, vec![]);
            assert_eq_event(
                receipt.events,
                vec![
                    Event {
                        from_address: fee_token_address,
                        keys: vec![get_selector_from_name("Transfer").unwrap()],
                        data: vec![
                            account.address(),  // from
                            recepient,          // to
                            transfer_amount,    // value low
                            FieldElement::ZERO, // value high
                        ],
                    },
                    Event {
                        from_address: account.address(),
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
                            account.address(),                                     // from
                            FieldElement::from_hex_be(SEQUENCER_ADDRESS).unwrap(), // to (sequencer address)
                            expected_fee,                                          // value low
                            FieldElement::ZERO,                                    // value high
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
async fn work_with_declare_transaction(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    let account = create_account(rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
    let (declare_tx, _, _) =
        account.declare_contract("./contracts/Counter.sierra.json", "./contracts/Counter.casm.json");
    let (mut declare_tx_legacy, _) = account.declare_legacy_contract("./contracts/ERC20.json");

    // manually setting fee else estimate_fee will be called and it will fail
    // as the nonce has not been updated yet
    let max_fee = FieldElement::from_hex_be(MAX_FEE_OVERRIDE).unwrap();

    // manually incrementing nonce else as fetching nonce will get 0 as
    // the both txns are in the same block
    let nonce = FieldElement::ONE;

    declare_tx_legacy = declare_tx_legacy.nonce(nonce).max_fee(max_fee);

    let txs = madara
        .create_block_with_txs(vec![
            Transaction::Declaration(declare_tx),
            Transaction::LegacyDeclaration(declare_tx_legacy),
        ])
        .await;

    assert!(txs.is_ok());

    let mut txs = txs.unwrap();
    assert_eq!(txs.len(), 2);

    let rpc_response_declare = match txs.remove(0).unwrap() {
        TransactionResult::Declaration(rpc_response) => rpc_response,
        _ => panic!("expected execution result"),
    };

    let rpc_response_declare_legacy = match txs.remove(0).unwrap() {
        TransactionResult::Declaration(rpc_response) => rpc_response,
        _ => panic!("expected execution result"),
    };

    // not validating the fields inside the transaction as
    // that is covered in get_block_with_txs
    let declare_tx_receipt = get_transaction_receipt(rpc, rpc_response_declare.transaction_hash).await;
    let declare_legacy_tx_receipt = get_transaction_receipt(rpc, rpc_response_declare_legacy.transaction_hash).await;

    let assert_declare_tx_receipt = |d1: TransactionReceiptResult, d2: DeclareTransactionReceipt| {
        let d1 = match d1 {
            Ok(MaybePendingTransactionReceipt::Receipt(TransactionReceipt::Declare(d1))) => d1,
            _ => panic!("expected declare transaction receipt"),
        };
        assert_eq!(d1.transaction_hash, d2.transaction_hash);
        assert_eq!(d1.actual_fee, d2.actual_fee);
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
                "0x01fc4c0d8f82edfd74ef83c5db42203fe4a70243a76e88e0a4a6ade9753d8ec9",
            )
            .unwrap(),
            actual_fee: expected_fee,
            finality_status: TransactionFinalityStatus::AcceptedOnL2,
            block_hash: FieldElement::from_hex_be("0x001624a3818c71653b975c9c8c89ea670d3ab7f863e9c02b697c7fe4f55470ad")
                .unwrap(),
            block_number: 1,
            messages_sent: vec![],
            events: vec![Event {
                from_address: fee_token_address,
                keys: vec![get_selector_from_name("Transfer").unwrap()],
                data: vec![
                    account.address(),                                     // from
                    FieldElement::from_hex_be(SEQUENCER_ADDRESS).unwrap(), // to (sequencer address)
                    expected_fee,                                          // value low
                    FieldElement::ZERO,                                    // value high
                ],
            }],
            execution_result: ExecutionResult::Succeeded,
        },
    );

    let expected_fee =
        FieldElement::from_hex_be("0x000000000000000000000000000000000000000000000000000000000000a3de").unwrap();
    assert_declare_tx_receipt(
        declare_legacy_tx_receipt,
        DeclareTransactionReceipt {
            transaction_hash: FieldElement::from_hex_be(
                "0x02c99c0d47dc755f95ea3c0bb92e77dee17bcda42ed2ebba438c8762a1d897c5",
            )
            .unwrap(),
            actual_fee: expected_fee,
            finality_status: TransactionFinalityStatus::AcceptedOnL2,
            block_hash: FieldElement::from_hex_be("0x001624a3818c71653b975c9c8c89ea670d3ab7f863e9c02b697c7fe4f55470ad")
                .unwrap(),
            block_number: 1,
            messages_sent: vec![],
            events: vec![Event {
                from_address: fee_token_address,
                keys: vec![get_selector_from_name("Transfer").unwrap()],
                data: vec![
                    account.address(),                                     // from
                    FieldElement::from_hex_be(SEQUENCER_ADDRESS).unwrap(), // to (sequencer address)
                    expected_fee,                                          // value low
                    FieldElement::ZERO,                                    // value high
                ],
            }],
            execution_result: ExecutionResult::Succeeded,
        },
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn work_with_deploy_account_transaction(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    let oz_factory =
        build_oz_account_factory(rpc, "0x123", FieldElement::from_hex_be(CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH).unwrap())
            .await;
    let account_deploy_txn = build_deploy_account_tx(&oz_factory, FieldElement::ONE);
    let account_address = account_deploy_txn.address();

    // add funds to deploy account
    let funding_account = create_account(rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
    assert!(
        madara
            .create_block_with_txs(vec![Transaction::Execution(funding_account.transfer_tokens(
                account_address,
                FieldElement::from_hex_be("0xFFFFFFFFFF").unwrap(),
                None,
            ))])
            .await
            .is_ok()
    );

    let txs = madara.create_block_with_txs(vec![Transaction::AccountDeployment(account_deploy_txn)]).await;

    let rpc_response = match txs.unwrap().remove(0).unwrap() {
        TransactionResult::AccountDeployment(rpc_response) => rpc_response,
        _ => panic!("expected execution result"),
    };

    let account_deployment_tx_receipt = get_transaction_receipt(rpc, rpc_response.transaction_hash).await;
    let fee_token_address = FieldElement::from_hex_be(FEE_TOKEN_ADDRESS).unwrap();
    let expected_fee = FieldElement::from_hex_be("0x13d6c").unwrap();

    match account_deployment_tx_receipt {
        Ok(MaybePendingTransactionReceipt::Receipt(TransactionReceipt::DeployAccount(receipt))) => {
            assert_eq!(receipt.transaction_hash, rpc_response.transaction_hash);
            assert_eq!(receipt.actual_fee, expected_fee);
            assert_eq!(receipt.finality_status, TransactionFinalityStatus::AcceptedOnL2);
            assert_eq!(
                receipt.block_hash,
                FieldElement::from_hex_be("0x07b5bff8207d465c2b92a89a9a65de363ff46089348e389ab5007aff914276c6")
                    .unwrap()
            );
            assert_eq!(receipt.block_number, 2);
            assert_eq_msg_to_l1(receipt.messages_sent, vec![]);
            assert_eq_event(
                receipt.events,
                vec![Event {
                    from_address: fee_token_address,
                    keys: vec![get_selector_from_name("Transfer").unwrap()],
                    data: vec![
                        account_address,                                       // from
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
async fn fail_invalid_transaction_hash(#[future] madara: MadaraClient) -> Result<(), anyhow::Error> {
    let madara = madara.await;
    let rpc = madara.get_starknet_client();

    assert!(rpc.get_transaction_receipt(FieldElement::ZERO).await.is_err());

    Ok(())
}
