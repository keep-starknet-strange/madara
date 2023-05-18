use core::str::FromStr;

use blockifier::abi::abi_utils::get_storage_var_address;
use frame_support::{assert_err, assert_ok, bounded_vec};
use hex::FromHex;
use mp_starknet::crypto::commitment;
use mp_starknet::crypto::hash::pedersen::PedersenHasher;
use mp_starknet::starknet_serde::transaction_from_json;
use mp_starknet::transaction::types::{
    EventWrapper, InvokeTransaction, Transaction, TransactionReceiptWrapper, TxType,
};
use sp_core::{H256, U256};
use sp_runtime::traits::ValidateUnsigned;
use sp_runtime::transaction_validity::{TransactionSource, TransactionValidityError};
use starknet_core::utils::get_selector_from_name;

use super::mock::*;
use super::utils::sign_message_hash;
use crate::message::Message;
use crate::{Error, Event, StorageView};

#[test]
fn given_hardcoded_contract_run_invoke_tx_fails_sender_not_deployed() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();

        // Wrong address (not deployed)
        let contract_address_str = "03e437FB56Bb213f5708Fcd6966502070e276c093ec271aA33433b89E21fd31f";
        let contract_address_bytes = <[u8; 32]>::from_hex(contract_address_str).unwrap();

        let transaction = InvokeTransaction {
            version: 1_u8,
            sender_address: contract_address_bytes,
            calldata: bounded_vec!(),
            nonce: U256::zero(),
            max_fee: U256::from(u128::MAX),
            signature: bounded_vec!(),
        };

        assert_err!(Starknet::invoke(none_origin, transaction), Error::<MockRuntime>::AccountNotDeployed);
    })
}

#[test]
fn given_hardcoded_contract_run_invoke_tx_fails_invalid_tx_version() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();

        let sender_add = get_account_address(AccountType::NoValidate);
        let transaction = InvokeTransaction { version: 3, sender_address: sender_add, ..InvokeTransaction::default() };

        assert_err!(Starknet::invoke(none_origin, transaction), Error::<MockRuntime>::TransactionExecutionFailed);
    });
}

#[test]
fn given_hardcoded_contract_run_invoke_tx_then_it_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();

        let json_content: &str = include_str!("../../../../../resources/transactions/invoke.json");
        let transaction =
            transaction_from_json(json_content, &[]).expect("Failed to create Transaction from JSON").into();

        let tx = Message {
            topics: vec![
                "0xdb80dd488acf86d17c747445b0eabb5d57c541d3bd7b6b87af987858e5066b2b".to_owned(),
                "0x0000000000000000000000000000000000000000000000000000000000000001".to_owned(),
                "0x0000000000000000000000000000000000000000000000000000000000000001".to_owned(),
                "0x01310e2c127c3b511c5ac0fd7949d544bb4d75b8bc83aaeb357e712ecf582771".to_owned(),
            ],
            data: "0x0000000000000000000000000000000000000000000000000000000000000001".to_owned(),
        }
        .try_into_transaction()
        .unwrap();

        assert_ok!(Starknet::invoke(none_origin.clone(), transaction));
        assert_ok!(Starknet::consume_l1_message(none_origin, tx));

        let pending = Starknet::pending();
        pretty_assertions::assert_eq!(pending.len(), 2);

        let receipt = &pending.get(0).unwrap().1;
        let expected_receipt = TransactionReceiptWrapper {
            transaction_hash: H256::from_str("0x01b8ffedfb222c609b81f301df55c640225abaa6a0715437c89f8edc21bbe5e8")
                .unwrap(),
            actual_fee: U256::from(52980),
            tx_type: TxType::Invoke,
            block_number: 2_u64,
            block_hash: U256::from_str("0xb4eba7e7c15c481312451a04105527a26ff1ca7ff12db3f0822634421871ecea").unwrap(),
            events: bounded_vec![EventWrapper {
                keys: bounded_vec!(
                    H256::from_str("0x0099cd8bde557814842a3121e8ddfd433a539b8c9f14bf31ebf108d12e6196e9").unwrap(),
                ),
                data: bounded_vec![
                    H256::from_str("0x02356b628d108863baf8644c945d97bad70190af5957031f4852d00d0f690a77").unwrap(),
                    H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000002").unwrap(),
                    H256::from_str("0x000000000000000000000000000000000000000000000000000000000000cef4").unwrap(),
                    H256::zero(),
                ],
                from_address: Starknet::fee_token_address(),
            },],
        };

        pretty_assertions::assert_eq!(*receipt, expected_receipt);
    });
}

#[test]
fn given_hardcoded_contract_run_invoke_tx_then_event_is_emitted() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();

        let json_content: &str = include_str!("../../../../../resources/transactions/invoke_emit_event.json");
        let transaction = transaction_from_json(json_content, &[]).expect("Failed to create Transaction from JSON").into();


        assert_ok!(Starknet::invoke(none_origin, transaction));

        let emitted_event = EventWrapper {
            keys: bounded_vec![
                H256::from_str("0x02d4fbe4956fedf49b5892807e00e7e9eea4680becba55f9187684a69e9424fa").unwrap()
            ],
            data: bounded_vec!(
                H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap()
            ),
            from_address: H256::from_str("0x024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7")
                .unwrap()
                .to_fixed_bytes(),
        };
        let expected_fee_transfer_event = EventWrapper {
                keys: bounded_vec![
                    H256::from_str("0x0099cd8bde557814842a3121e8ddfd433a539b8c9f14bf31ebf108d12e6196e9").unwrap()
                ],
                data: bounded_vec!(
                    H256::from_str("0x01a3339ec92ac1061e3e0f8e704106286c642eaf302e94a582e5f95ef5e6b4d0").unwrap(), // From
                    H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000002").unwrap(), // To
                    H256::from_str("0x000000000000000000000000000000000000000000000000000000000000d0f2").unwrap(), // Amount low
                    H256::zero(), // Amount high
                ),
                from_address: Starknet::fee_token_address(),
            };
        let events = System::events();
        // Actual event.
        pretty_assertions::assert_eq!(
            Event::StarknetEvent(emitted_event.clone()),
            events[events.len() - 2].event.clone().try_into().unwrap()
        );
        // Fee transfer event.
        pretty_assertions::assert_eq!(
            Event::StarknetEvent(expected_fee_transfer_event.clone())
            ,events.last().unwrap().event.clone().try_into().unwrap(),
        );

        let pending = Starknet::pending();
        let events = Starknet::pending_events();
        let transactions: Vec<Transaction> = pending.clone().into_iter().map(|(transaction, _)| transaction).collect();
        let (_transaction_commitment, event_commitment) =
            commitment::calculate_commitments::<PedersenHasher>(&transactions, &events);

        assert_eq!(
            event_commitment,
            H256::from_str("0x00ebe70524f4d05a64dc130466d97d0852733d731033a59005c980530b09dd3d").unwrap()
        );
        assert_eq!(events.len(), 2);
        assert_eq!(pending.len(), 1);

        let expected_receipt = TransactionReceiptWrapper {
            transaction_hash: H256::from_str("0x0554f9443c06ce406badc7159f2c0da29eac095f8571fe1a6ce44a2076829a52").unwrap(),
            actual_fee: U256::from(53490),
            tx_type: TxType::Invoke,
            block_number: 2_u64,
            block_hash: U256::from_str("0xb4eba7e7c15c481312451a04105527a26ff1ca7ff12db3f0822634421871ecea").unwrap(),
            events: bounded_vec!(emitted_event, expected_fee_transfer_event),
        };
        let receipt = &pending.get(0).unwrap().1;
        pretty_assertions::assert_eq!(*receipt, expected_receipt);
    });
}

#[test]
fn given_hardcoded_contract_run_storage_read_and_write_it_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();

        let json_content: &str = include_str!("../../../../../resources/transactions/storage_read_write.json");
        let transaction =
            transaction_from_json(json_content, include_bytes!("../../../../../resources/account/simple/account.json"))
                .expect("Failed to create Transaction from JSON")
                .into();

        let target_contract_address =
            U256::from_str("024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap();
        let storage_var_selector = U256::from(25);

        let mut contract_address_bytes = [0_u8; 32];
        target_contract_address.to_big_endian(&mut contract_address_bytes);
        let mut storage_var_selector_bytes = [0_u8; 32];
        storage_var_selector.to_big_endian(&mut storage_var_selector_bytes);

        assert_ok!(Starknet::invoke(none_origin, transaction));
        assert_eq!(
            Starknet::storage((contract_address_bytes, H256::from_slice(&storage_var_selector_bytes))),
            U256::one()
        );
    });
}

#[test]
fn test_verify_nonce() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let json_content: &str = include_str!("../../../../../resources/transactions/invoke.json");
        let tx = transaction_from_json(json_content, &[]).expect("Failed to create Transaction from JSON").into();

        // Test for a valid nonce (0)
        assert_ok!(Starknet::invoke(RuntimeOrigin::none(), tx));

        // Test for an invalid nonce (actual: 0, expected: 1)
        let json_content_2: &str = include_str!("../../../../../resources/transactions/invoke.json");
        let tx_2 = transaction_from_json(json_content_2, &[]).expect("Failed to create Transaction from JSON").into();

        assert_err!(Starknet::invoke(RuntimeOrigin::none(), tx_2), Error::<MockRuntime>::TransactionExecutionFailed);
    });
}

#[test]
fn given_hardcoded_contract_run_invoke_on_openzeppelin_account_then_it_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);
        let none_origin = RuntimeOrigin::none();

        let json_content: &str = include_str!("../../../../../resources/transactions/invoke_openzeppelin.json");
        let mut transaction = transaction_from_json(json_content, &[]).expect("Failed to create Transaction from JSON");
        transaction.signature = sign_message_hash(transaction.hash);

        assert_ok!(Starknet::invoke(none_origin, transaction.into()));
    });
}

#[test]
fn given_hardcoded_contract_run_invoke_on_openzeppelin_account_with_incorrect_signature_then_it_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();

        let json_content: &str = include_str!("../../../../../resources/transactions/invoke_openzeppelin.json");
        let mut transaction = transaction_from_json(json_content, &[]).expect("Failed to create Transaction from JSON");
        transaction.signature = bounded_vec!(H256::from_low_u64_be(1), H256::from_low_u64_be(1));

        assert_err!(
            Starknet::invoke(none_origin, transaction.into()),
            Error::<MockRuntime>::TransactionExecutionFailed
        );
    });
}

#[test]
fn given_hardcoded_contract_run_invoke_on_argent_account_then_it_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);
        let none_origin = RuntimeOrigin::none();

        let json_content: &str = include_str!("../../../../../resources/transactions/invoke_argent.json");
        let mut transaction = transaction_from_json(json_content, &[]).expect("Failed to create Transaction from JSON");
        transaction.signature = sign_message_hash(transaction.hash);

        assert_ok!(Starknet::invoke(none_origin, transaction.into()));
    });
}

#[test]
fn given_hardcoded_contract_run_invoke_on_argent_account_with_incorrect_signature_then_it_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);
        let none_origin = RuntimeOrigin::none();

        let json_content: &str = include_str!("../../../../../resources/transactions/invoke_argent.json");
        let mut transaction = transaction_from_json(json_content, &[]).expect("Failed to create Transaction from JSON");
        transaction.signature = bounded_vec!(H256::from_low_u64_be(1), H256::from_low_u64_be(1));

        assert_err!(
            Starknet::invoke(none_origin, transaction.into()),
            Error::<MockRuntime>::TransactionExecutionFailed
        );
    });
}

#[test]
fn given_hardcoded_contract_run_invoke_on_braavos_account_then_it_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);
        let none_origin = RuntimeOrigin::none();

        let json_content: &str = include_str!("../../../../../resources/transactions/invoke_braavos.json");
        let mut transaction = transaction_from_json(json_content, &[]).expect("Failed to create Transaction from JSON");
        transaction.signature = sign_message_hash(transaction.hash);

        assert_ok!(Starknet::invoke(none_origin, transaction.into()));
    });
}

#[test]
fn given_hardcoded_contract_run_invoke_on_braavos_account_with_incorrect_signature_then_it_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);
        let none_origin = RuntimeOrigin::none();

        let json_content: &str = include_str!("../../../../../resources/transactions/invoke_braavos.json");
        let mut transaction = transaction_from_json(json_content, &[]).expect("Failed to create Transaction from JSON");
        transaction.signature = bounded_vec!(H256::from_low_u64_be(1), H256::from_low_u64_be(1));

        assert_err!(
            Starknet::invoke(none_origin, transaction.into()),
            Error::<MockRuntime>::TransactionExecutionFailed
        );
    });
}

#[test]
fn given_hardcoded_contract_run_invoke_with_inner_call_in_validate_then_it_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);
        let none_origin = RuntimeOrigin::none();

        let json_content: &str = include_str!("../../../../../resources/transactions/invoke.json");
        let mut transaction = transaction_from_json(json_content, &[]).expect("Failed to create Transaction from JSON");
        transaction.signature = bounded_vec!(H256::from_low_u64_be(1), H256::from_low_u64_be(1));
        transaction.sender_address = get_account_address(AccountType::InnerCall);

        let storage_key = get_storage_var_address("destination", &[]).unwrap();
        let destination =
            <[u8; 32]>::from_hex("024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap(); // Test contract address
        StorageView::<MockRuntime>::insert(
            (transaction.sender_address, H256::from(storage_key.0.0.0)),
            U256::from(destination),
        );

        let storage_key = get_storage_var_address("function_selector", &[]).unwrap();
        let selector = get_selector_from_name("without_arg").unwrap();
        StorageView::<MockRuntime>::insert(
            (transaction.sender_address, H256::from(storage_key.0.0.0)),
            U256::from(selector.to_bytes_be()),
        );

        assert_err!(
            Starknet::invoke(none_origin, transaction.into()),
            Error::<MockRuntime>::TransactionExecutionFailed
        );
    });
}

#[test]
fn given_hardcoded_contract_validate_invoke_on_braavos_account_with_incorrect_signature_should_fail() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let json_content: &str = include_str!("../../../../../resources/transactions/invoke_braavos.json");
        let mut transaction = transaction_from_json(json_content, &[]).expect("Failed to create Transaction from JSON");
        transaction.signature = bounded_vec!(H256::from_low_u64_be(1), H256::from_low_u64_be(1));

        let validate_result = Starknet::validate_unsigned(
            TransactionSource::InBlock,
            &crate::Call::invoke { transaction: transaction.into() },
        );
        assert!(matches!(validate_result.unwrap_err(), TransactionValidityError::Invalid(_)));
    });
}
