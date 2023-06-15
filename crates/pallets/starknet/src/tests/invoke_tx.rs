use core::str::FromStr;

use blockifier::abi::abi_utils::get_storage_var_address;
use frame_support::{assert_err, assert_ok, bounded_vec};
use mp_starknet::crypto::commitment::{self, calculate_invoke_tx_hash};
use mp_starknet::execution::types::Felt252Wrapper;
use mp_starknet::starknet_serde::transaction_from_json;
use mp_starknet::transaction::types::{
    EventWrapper, InvokeTransaction, Transaction, TransactionReceiptWrapper, TxType,
};
use sp_core::H256;
use sp_runtime::traits::ValidateUnsigned;
use sp_runtime::transaction_validity::{TransactionSource, TransactionValidityError, ValidTransaction};
use starknet_core::utils::get_selector_from_name;
use starknet_crypto::FieldElement;

use super::constants::{BLOCKIFIER_ACCOUNT_ADDRESS, TEST_CONTRACT_ADDRESS};
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
        let contract_address =
            Felt252Wrapper::from_hex_be("0x03e437FB56Bb213f5708Fcd6966502070e276c093ec271aA33433b89E21fd31f").unwrap();

        let transaction = InvokeTransaction {
            version: 1_u8,
            sender_address: contract_address,
            calldata: bounded_vec!(),
            nonce: Felt252Wrapper::ZERO,
            max_fee: Felt252Wrapper::from(u128::MAX),
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
        let transaction: InvokeTransaction =
            transaction_from_json(json_content, &[]).expect("Failed to create Transaction from JSON").into();
        let chain_id = Starknet::chain_id();
        let transaction_hash = calculate_invoke_tx_hash(transaction.clone(), chain_id);

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
            transaction_hash: Felt252Wrapper::from_hex_be(
                "0x01b8ffedfb222c609b81f301df55c640225abaa6a0715437c89f8edc21bbe5e8",
            )
            .unwrap(),
            actual_fee: Felt252Wrapper::from(52980_u128),
            tx_type: TxType::Invoke,
            events: bounded_vec![EventWrapper {
                keys: bounded_vec!(
                    Felt252Wrapper::from_hex_be("0x0099cd8bde557814842a3121e8ddfd433a539b8c9f14bf31ebf108d12e6196e9")
                        .unwrap(),
                ),
                data: bounded_vec![
                    Felt252Wrapper::from_hex_be(BLOCKIFIER_ACCOUNT_ADDRESS).unwrap(),
                    Felt252Wrapper::from_hex_be("0x000000000000000000000000000000000000000000000000000000000000dead")
                        .unwrap(),
                    Felt252Wrapper::from_hex_be("0x000000000000000000000000000000000000000000000000000000000000cef4")
                        .unwrap(),
                    Felt252Wrapper::ZERO,
                ],
                from_address: Starknet::fee_token_address(),
                transaction_hash
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
        let transaction: InvokeTransaction =
            transaction_from_json(json_content, &[]).expect("Failed to create Transaction from JSON").into();
        let chain_id = Starknet::chain_id();
        let transaction_hash = calculate_invoke_tx_hash(transaction.clone(), chain_id);

        assert_ok!(Starknet::invoke(none_origin, transaction));

        let emitted_event = EventWrapper {
            keys: bounded_vec![
                Felt252Wrapper::from_hex_be("0x02d4fbe4956fedf49b5892807e00e7e9eea4680becba55f9187684a69e9424fa")
                    .unwrap()
            ],
            data: bounded_vec!(Felt252Wrapper::from_hex_be("0x1").unwrap()),
            from_address: Felt252Wrapper::from_hex_be(TEST_CONTRACT_ADDRESS).unwrap(),
            transaction_hash,
        };
        let expected_fee_transfer_event = EventWrapper {
            keys: bounded_vec![
                Felt252Wrapper::from_hex_be("0x0099cd8bde557814842a3121e8ddfd433a539b8c9f14bf31ebf108d12e6196e9")
                    .unwrap()
            ],
            data: bounded_vec!(
                Felt252Wrapper::from_hex_be("0x01a3339ec92ac1061e3e0f8e704106286c642eaf302e94a582e5f95ef5e6b4d0")
                    .unwrap(), // From
                Felt252Wrapper::from_hex_be("0xdead").unwrap(), // To
                Felt252Wrapper::from_hex_be("0xd0f2").unwrap(), // Amount low
                Felt252Wrapper::ZERO,                           // Amount high
            ),
            from_address: Starknet::fee_token_address(),
            transaction_hash,
        };
        let events = System::events();
        // Actual event.
        pretty_assertions::assert_eq!(
            Event::StarknetEvent(emitted_event.clone()),
            events[events.len() - 2].event.clone().try_into().unwrap()
        );
        // Fee transfer event.
        pretty_assertions::assert_eq!(
            Event::StarknetEvent(expected_fee_transfer_event.clone()),
            events.last().unwrap().event.clone().try_into().unwrap(),
        );

        let pending = Starknet::pending();
        let events = Starknet::pending_events();
        let transactions: Vec<Transaction> = pending.clone().into_iter().map(|(transaction, _)| transaction).collect();
        let (_transaction_commitment, event_commitment) =
            commitment::calculate_commitments::<<MockRuntime as crate::Config>::SystemHash>(&transactions, &events);

        assert_eq!(
            event_commitment,
            H256::from_str("0x0627b4c3f2b6b1d89df90492d1f98d4479d59e7e074496f893731cb79ea2f6ba").unwrap()
        );
        assert_eq!(events.len(), 2);
        assert_eq!(pending.len(), 1);

        let expected_receipt = TransactionReceiptWrapper {
            transaction_hash: Felt252Wrapper::from_hex_be(
                "0x0554f9443c06ce406badc7159f2c0da29eac095f8571fe1a6ce44a2076829a52",
            )
            .unwrap(),
            actual_fee: Felt252Wrapper::from(53490_u128),
            tx_type: TxType::Invoke,
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
        let transaction = transaction_from_json(
            json_content,
            include_bytes!("../../../../../cairo-contracts/build/NoValidateAccount.json"),
        )
        .expect("Failed to create Transaction from JSON")
        .into();

        let target_contract_address =
            Felt252Wrapper::from_hex_be("024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap();
        let storage_var_selector = Felt252Wrapper::from(25_u128);

        assert_ok!(Starknet::invoke(none_origin, transaction));
        assert_eq!(
            Starknet::storage((
                Into::<Felt252Wrapper>::into(target_contract_address),
                Into::<Felt252Wrapper>::into(storage_var_selector)
            )),
            Felt252Wrapper::ONE
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

        let validate_result = Starknet::validate_unsigned(
            TransactionSource::InBlock,
            &crate::Call::invoke { transaction: transaction.clone().into() },
        );
        assert_ok!(validate_result);

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
        transaction.signature = bounded_vec!(Felt252Wrapper::ONE, Felt252Wrapper::ONE);

        let validate_result = Starknet::validate_unsigned(
            TransactionSource::InBlock,
            &crate::Call::invoke { transaction: transaction.clone().into() },
        );
        assert!(matches!(validate_result.unwrap_err(), TransactionValidityError::Invalid(_)));

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

        let validate_result = Starknet::validate_unsigned(
            TransactionSource::InBlock,
            &crate::Call::invoke { transaction: transaction.clone().into() },
        );
        assert_ok!(validate_result);

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
        transaction.signature = bounded_vec!(Felt252Wrapper::ONE, Felt252Wrapper::ONE);

        let validate_result = Starknet::validate_unsigned(
            TransactionSource::InBlock,
            &crate::Call::invoke { transaction: transaction.clone().into() },
        );
        assert!(matches!(validate_result.unwrap_err(), TransactionValidityError::Invalid(_)));

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

        let validate_result = Starknet::validate_unsigned(
            TransactionSource::InBlock,
            &crate::Call::invoke { transaction: transaction.clone().into() },
        );
        assert_ok!(validate_result);

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
        transaction.signature = bounded_vec!(Felt252Wrapper::ONE, Felt252Wrapper::ONE);

        let validate_result = Starknet::validate_unsigned(
            TransactionSource::InBlock,
            &crate::Call::invoke { transaction: transaction.clone().into() },
        );
        assert!(matches!(validate_result.unwrap_err(), TransactionValidityError::Invalid(_)));

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
        transaction.signature = bounded_vec!(Felt252Wrapper::ONE, Felt252Wrapper::ONE);
        transaction.sender_address = get_account_address(AccountType::InnerCall);

        let storage_key = get_storage_var_address("destination", &[]).unwrap();
        let destination = Felt252Wrapper::from_hex_be(TEST_CONTRACT_ADDRESS).unwrap();
        StorageView::<MockRuntime>::insert(
            (transaction.sender_address, Felt252Wrapper::from(storage_key.0.0)),
            Into::<Felt252Wrapper>::into(destination),
        );

        let storage_key = get_storage_var_address("function_selector", &[]).unwrap();
        let selector = get_selector_from_name("without_arg").unwrap();
        StorageView::<MockRuntime>::insert(
            (transaction.sender_address, Felt252Wrapper::from(storage_key.0.0)),
            Felt252Wrapper::from(selector),
        );

        assert_err!(
            Starknet::invoke(none_origin, transaction.into()),
            Error::<MockRuntime>::TransactionExecutionFailed
        );
    });
}

#[test]
fn test_verify_tx_longevity() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let json_content: &str = include_str!("../../../../../resources/transactions/invoke.json");
        let transaction: InvokeTransaction =
            transaction_from_json(json_content, &[]).expect("Failed to create Transaction from JSON").into();

        let validate_result =
            Starknet::validate_unsigned(TransactionSource::InBlock, &crate::Call::invoke { transaction });

        assert!(validate_result.unwrap().longevity == TransactionLongevity::get());
    });
}

#[test]
fn test_verify_no_require_tag() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let json_content: &str = include_str!("../../../../../resources/transactions/invoke.json");
        let transaction: InvokeTransaction =
            transaction_from_json(json_content, &[]).expect("Failed to create Transaction from JSON").into();

        let validate_result = Starknet::validate_unsigned(
            TransactionSource::InBlock,
            &crate::Call::invoke { transaction: transaction.clone() },
        );

        let valid_transaction_expected = ValidTransaction::with_tag_prefix("starknet")
            .priority(u64::MAX - (TryInto::<u64>::try_into(transaction.nonce)).unwrap())
            .and_provides((transaction.sender_address, transaction.nonce))
            .longevity(TransactionLongevity::get())
            .propagate(true)
            .build();

        assert_eq!(validate_result.unwrap(), valid_transaction_expected.unwrap())
    });
}

#[test]
fn test_verify_require_tag() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let json_content: &str = include_str!("../../../../../resources/transactions/invoke_nonce.json");
        let transaction: InvokeTransaction =
            transaction_from_json(json_content, &[]).expect("Failed to create Transaction from JSON").into();

        let validate_result = Starknet::validate_unsigned(
            TransactionSource::InBlock,
            &crate::Call::invoke { transaction: transaction.clone() },
        );

        let valid_transaction_expected = ValidTransaction::with_tag_prefix("starknet")
            .priority(u64::MAX - (TryInto::<u64>::try_into(transaction.nonce)).unwrap())
            .and_provides((transaction.sender_address, transaction.nonce))
            .longevity(TransactionLongevity::get())
            .propagate(true)
            .and_requires((transaction.sender_address, Felt252Wrapper(transaction.nonce.0 - FieldElement::ONE)))
            .build();

        assert_eq!(validate_result.unwrap(), valid_transaction_expected.unwrap())
    });
}
