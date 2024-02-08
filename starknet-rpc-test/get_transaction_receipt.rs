extern crate starknet_rpc_test;

use std::vec;

use assert_matches::assert_matches;
use rstest::rstest;
use starknet_core::types::{
    Event, ExecutionResult, MaybePendingTransactionReceipt, MsgToL1, TransactionFinalityStatus, TransactionReceipt,
};
use starknet_core::utils::get_selector_from_name;
use starknet_ff::FieldElement;
use starknet_providers::Provider;
use starknet_rpc_test::constants::{
    ARGENT_CONTRACT_ADDRESS, CAIRO_1_ACCOUNT_CONTRACT_CLASS_HASH, FEE_TOKEN_ADDRESS, SEQUENCER_ADDRESS, SIGNER_PRIVATE,
    UDC_ADDRESS,
};
use starknet_rpc_test::fixtures::{madara, ThreadSafeMadaraClient};
use starknet_rpc_test::utils::{
    assert_eq_msg_to_l1, build_deploy_account_tx, build_oz_account_factory, build_single_owner_account,
    get_contract_address_from_deploy_tx, get_transaction_receipt, AccountActions,
};
use starknet_rpc_test::{Transaction, TransactionResult};

#[rstest]
#[tokio::test]
async fn work_with_invoke_transaction(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let recipient = FieldElement::from_hex_be("0x123").unwrap();
    let transfer_amount = FieldElement::ONE;

    let mut txs = {
        let mut madara_write_lock = madara.write().await;
        let account = build_single_owner_account(&rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
        madara_write_lock
            .create_block_with_txs(vec![Transaction::Execution(account.transfer_tokens(
                recipient,
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
            assert_eq!(receipt.actual_fee, expected_fee);
            assert_eq!(receipt.finality_status, TransactionFinalityStatus::AcceptedOnL2);
            assert_eq_msg_to_l1(receipt.messages_sent, vec![]);
            assert_eq!(
                receipt.events,
                vec![
                    Event {
                        from_address: fee_token_address,
                        keys: vec![get_selector_from_name("Transfer").unwrap()],
                        data: vec![
                            FieldElement::from_hex_be(ARGENT_CONTRACT_ADDRESS).unwrap(), // from
                            recipient,                                                   // to
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
async fn work_with_declare_transaction(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let mut txs = {
        let mut madara_write_lock = madara.write().await;
        let account = build_single_owner_account(&rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
        let (declare_tx, _, _) = account.declare_contract(
            "./contracts/counter7/counter7.contract_class.json",
            "./contracts/counter7/counter7.compiled_contract_class.json",
        );

        madara_write_lock.create_block_with_txs(vec![Transaction::Declaration(declare_tx)]).await?
    };

    let rpc_response_declare = match txs.remove(0).unwrap() {
        TransactionResult::Declaration(rpc_response) => rpc_response,
        _ => panic!("expected execution result"),
    };

    let fee_token_address = FieldElement::from_hex_be(FEE_TOKEN_ADDRESS).unwrap();
    let expected_fee =
        FieldElement::from_hex_be("0x00000000000000000000000000000000000000000000000000000000000030fc").unwrap();
    let expected_events = vec![Event {
        from_address: fee_token_address,
        keys: vec![get_selector_from_name("Transfer").unwrap()],
        data: vec![
            FieldElement::from_hex_be(ARGENT_CONTRACT_ADDRESS).unwrap(), // from
            FieldElement::from_hex_be(SEQUENCER_ADDRESS).unwrap(),       // to (sequencer address)
            expected_fee,                                                // value low
            FieldElement::ZERO,                                          // value high
        ],
    }];

    match get_transaction_receipt(&rpc, rpc_response_declare.transaction_hash).await {
        Ok(MaybePendingTransactionReceipt::Receipt(TransactionReceipt::Declare(tx_receipt))) => {
            assert_eq!(tx_receipt.actual_fee, expected_fee);
            assert_eq!(tx_receipt.finality_status, TransactionFinalityStatus::AcceptedOnL2);
            assert_eq_msg_to_l1(tx_receipt.messages_sent, vec![]);
            assert_eq!(tx_receipt.events, expected_events);
            assert_matches!(tx_receipt.execution_result, ExecutionResult::Succeeded);
        }
        _ => panic!("expected declare transaction receipt"),
    };

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
            assert_eq!(receipt.actual_fee, expected_fee);
            assert_eq!(receipt.finality_status, TransactionFinalityStatus::AcceptedOnL2);
            assert_eq_msg_to_l1(receipt.messages_sent, vec![]);
            assert_eq!(
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
            assert_eq!(receipt.contract_address, account_address);
        }
        _ => panic!("expected deploy account transaction receipt"),
    };

    Ok(())
}

#[rstest]
#[tokio::test]
async fn ensure_transfer_fee_event_not_messed_up_with_similar_transfer(
    madara: &ThreadSafeMadaraClient,
) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    let mut madara_write_lock = madara.write().await;
    let transfer_amount = FieldElement::from_hex_be("0x100000").unwrap();
    let funding_account = build_single_owner_account(&rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
    let mut tx = madara_write_lock
        .create_block_with_txs(vec![Transaction::Execution(funding_account.transfer_tokens(
            FieldElement::from_hex_be(SEQUENCER_ADDRESS).unwrap(),
            transfer_amount,
            None,
        ))])
        .await?;
    let rpc_response = match tx.remove(0).unwrap() {
        TransactionResult::Execution(rpc_response) => rpc_response,
        _ => panic!("expected execution result"),
    };
    let tx_receipt = get_transaction_receipt(&rpc, rpc_response.transaction_hash).await;
    let fee_token_address = FieldElement::from_hex_be(FEE_TOKEN_ADDRESS).unwrap();
    let expected_fee = FieldElement::from_hex_be("0xf154").unwrap();

    match tx_receipt {
        Ok(MaybePendingTransactionReceipt::Receipt(TransactionReceipt::Invoke(mut receipt))) => {
            assert_eq!(receipt.transaction_hash, rpc_response.transaction_hash);
            assert_eq!(receipt.actual_fee, expected_fee);
            assert_eq!(receipt.finality_status, TransactionFinalityStatus::AcceptedOnL2);
            assert_eq_msg_to_l1(receipt.messages_sent, vec![]);
            receipt.events.remove(1);
            assert_eq!(
                receipt.events,
                vec![
                    Event {
                        from_address: fee_token_address,
                        keys: vec![get_selector_from_name("Transfer").unwrap()],
                        data: vec![
                            FieldElement::from_hex_be(ARGENT_CONTRACT_ADDRESS).unwrap(), // from
                            FieldElement::from_hex_be(SEQUENCER_ADDRESS).unwrap(),       // to
                            transfer_amount,                                             // value low
                            FieldElement::ZERO,                                          // value high
                        ],
                    },
                    Event {
                        from_address: fee_token_address,
                        keys: vec![get_selector_from_name("Transfer").unwrap()],
                        data: vec![
                            FieldElement::from_hex_be(ARGENT_CONTRACT_ADDRESS).unwrap(), // from
                            FieldElement::from_hex_be(SEQUENCER_ADDRESS).unwrap(),       // to
                            expected_fee,                                                // value low
                            FieldElement::ZERO,                                          // value high
                        ],
                    },
                ],
            );
            assert_matches!(receipt.execution_result, ExecutionResult::Succeeded);
        }
        _ => panic!("expected transfer receipt"),
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

#[rstest]
#[tokio::test]
async fn work_with_messages_to_l1(madara: &ThreadSafeMadaraClient) -> Result<(), anyhow::Error> {
    let rpc = madara.get_starknet_client().await;

    // 1. Declaring class for our L2 > L1 contract

    let account = build_single_owner_account(&rpc, SIGNER_PRIVATE, ARGENT_CONTRACT_ADDRESS, true);
    let (declare_tx, _) = account.declare_legacy_contract("../cairo-contracts/build/send_message.json");

    let txs = {
        let mut madara_write_lock = madara.write().await;
        madara_write_lock.create_block_with_txs(vec![Transaction::LegacyDeclaration(declare_tx)]).await?
    };

    // 2. Determine class hash

    let class_hash = assert_matches!(
        &txs[0],
        Ok(TransactionResult::Declaration(rpc_response)) => rpc_response.class_hash
    );

    // 3. Next, deploying an instance of this class using universal deployer

    let deploy_tx = account.invoke_contract(
        FieldElement::from_hex_be(UDC_ADDRESS).unwrap(),
        "deployContract",
        vec![
            class_hash,
            FieldElement::ZERO, // salt
            FieldElement::ZERO, // unique
            FieldElement::ZERO, // calldata len
        ],
        None,
    );

    let mut txs = {
        let mut madara_write_lock = madara.write().await;
        madara_write_lock.create_block_with_txs(vec![Transaction::Execution(deploy_tx)]).await?
    };

    // 4. Now, we need to get the deployed contract address
    let deploy_tx_result = txs.pop().unwrap();
    let contract_address = get_contract_address_from_deploy_tx(&rpc, deploy_tx_result).await?;

    // 5. Sending message to L1

    let invoke_tx = account.invoke_contract(
        contract_address,
        "send_message_l2_to_l1",
        vec![FieldElement::ZERO, FieldElement::ONE, FieldElement::TWO],
        None,
    );

    let txs = {
        let mut madara_write_lock = madara.write().await;
        madara_write_lock.create_block_with_txs(vec![Transaction::Execution(invoke_tx)]).await?
    };

    // 6. Finally, checking that there is a single MessageToL1 in the receipt

    let invoke_tx_hash = assert_matches!(
        &txs[0],
        Ok(TransactionResult::Execution(rpc_response)) => rpc_response.transaction_hash
    );

    let invoke_tx_receipt = get_transaction_receipt(&rpc, invoke_tx_hash).await?;

    let messages_sent = assert_matches!(
        invoke_tx_receipt,
        MaybePendingTransactionReceipt::Receipt(TransactionReceipt::Invoke(receipt)) => receipt.messages_sent
    );

    assert_eq_msg_to_l1(
        vec![MsgToL1 {
            from_address: contract_address,
            to_address: FieldElement::ZERO,
            payload: vec![FieldElement::TWO],
        }],
        messages_sent,
    );

    Ok(())
}
