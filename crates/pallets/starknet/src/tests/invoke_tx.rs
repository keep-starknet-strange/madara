use blockifier::abi::abi_utils::get_storage_var_address;
use frame_support::{assert_err, assert_ok};
use mp_felt::Felt252Wrapper;
use mp_transactions::compute_hash::ComputeTransactionHash;
use mp_transactions::{InvokeTransaction, InvokeTransactionV1};
use pretty_assertions::assert_eq;
use sp_runtime::traits::ValidateUnsigned;
use sp_runtime::transaction_validity::{
    InvalidTransaction, TransactionSource, TransactionValidityError, ValidTransaction,
};
use starknet_api::api_core::{ContractAddress, Nonce, PatriciaKey};
use starknet_api::hash::StarkFelt;
use starknet_api::state::StorageKey;
use starknet_api::transaction::{Event as StarknetEvent, EventContent, EventData, EventKey, TransactionHash};
use starknet_core::utils::get_selector_from_name;
use starknet_crypto::FieldElement;

use super::constants::{BLOCKIFIER_ACCOUNT_ADDRESS, MULTIPLE_EVENT_EMITTING_CONTRACT_ADDRESS, TEST_CONTRACT_ADDRESS};
use super::mock::default_mock::*;
use super::mock::*;
use super::utils::sign_message_hash;
use crate::tests::{
    get_invoke_argent_dummy, get_invoke_braavos_dummy, get_invoke_dummy, get_invoke_emit_event_dummy,
    get_invoke_nonce_dummy, get_invoke_openzeppelin_dummy, get_storage_read_write_dummy, set_nonce,
};
use crate::{Call, Config, Error, Event, StorageView};

#[test]
fn given_hardcoded_contract_run_invoke_tx_fails_sender_not_deployed() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();

        // Wrong address (not deployed)
        let contract_address =
            Felt252Wrapper::from_hex_be("0x03e437FB56Bb213f5708Fcd6966502070e276c093ec271aA33433b89E21fd31f").unwrap();

        let transaction = InvokeTransactionV1 {
            sender_address: contract_address,
            calldata: vec![],
            nonce: Felt252Wrapper::ZERO,
            max_fee: u128::MAX,
            signature: vec![],
            offset_version: false,
        };

        assert_err!(Starknet::invoke(none_origin, transaction.into()), Error::<MockRuntime>::AccountNotDeployed);
    })
}

#[test]
fn given_hardcoded_contract_run_invoke_tx_then_it_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();

        let transaction: InvokeTransaction = get_invoke_dummy(Felt252Wrapper::ZERO).into();

        assert_ok!(Starknet::invoke(none_origin.clone(), transaction));

        let pending_txs = Starknet::pending();
        pretty_assertions::assert_eq!(pending_txs.len(), 1);
        let pending_hashes = Starknet::pending_hashes();
        pretty_assertions::assert_eq!(pending_hashes.len(), 1);

        assert_eq!(
            pending_hashes[0],
            TransactionHash(
                StarkFelt::try_from("0x02dfd0ded452658d67535279591c1ed9898431e1eafad7896239f0bfa68493d6").unwrap()
            )
        );
        assert!(System::events().into_iter().map(|event_record| event_record.event).any(|e| match e {
            RuntimeEvent::Starknet(Event::StarknetEvent(e)) => {
                e == StarknetEvent {
                    from_address: Starknet::fee_token_address(),
                    content: EventContent {
                        keys: vec![EventKey(
                            Felt252Wrapper::from(get_selector_from_name(mp_fee::TRANSFER_SELECTOR_NAME).unwrap())
                                .into(),
                        )],
                        data: EventData(vec![
                            StarkFelt::try_from(BLOCKIFIER_ACCOUNT_ADDRESS).unwrap(),
                            StarkFelt::try_from("0x000000000000000000000000000000000000000000000000000000000000dead")
                                .unwrap(),
                            StarkFelt::try_from("0x00000000000000000000000000000000000000000000000000000000000001a4")
                                .unwrap(),
                            StarkFelt::from(0u128),
                        ]),
                    },
                }
            }
            _ => false,
        }));
    });
}

#[test]
fn given_hardcoded_contract_run_invoke_tx_then_event_is_emitted() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();

        let transaction: InvokeTransaction = get_invoke_emit_event_dummy().into();

        assert_ok!(Starknet::invoke(none_origin, transaction));

        let emitted_event = StarknetEvent {
            from_address: ContractAddress(PatriciaKey(StarkFelt::try_from(TEST_CONTRACT_ADDRESS).unwrap())),
            content: EventContent {
                keys: vec![EventKey(
                    StarkFelt::try_from("0x02d4fbe4956fedf49b5892807e00e7e9eea4680becba55f9187684a69e9424fa").unwrap(),
                )],
                data: EventData(vec![
                    StarkFelt::from(1u128), // Amount high
                ]),
            },
        };
        let expected_fee_transfer_event = StarknetEvent {
            from_address: Starknet::fee_token_address(),
            content: EventContent {
                keys: vec![EventKey(
                    StarkFelt::try_from(Felt252Wrapper::from(get_selector_from_name(mp_fee::TRANSFER_SELECTOR_NAME).unwrap())).unwrap(),
                )],
                data: EventData(vec![
                    StarkFelt::try_from("0x01a3339ec92ac1061e3e0f8e704106286c642eaf302e94a582e5f95ef5e6b4d0").unwrap(), // From
                    StarkFelt::try_from("0xdead").unwrap(), // To
                    StarkFelt::try_from("0x1a4").unwrap(),  // Amount low
                    StarkFelt::from(0u128),                 // Amount high
                ]),
            },
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
    });
}

#[test]
fn given_hardcoded_contract_run_invoke_tx_then_multiple_events_is_emitted() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let emit_contract_address = Felt252Wrapper::from_hex_be(MULTIPLE_EVENT_EMITTING_CONTRACT_ADDRESS).unwrap();

        let sender_account = get_account_address(None, AccountType::V0(AccountTypeV0Inner::NoValidate));

        let emit_internal_selector = Felt252Wrapper::from(get_selector_from_name("emit_internal").unwrap());
        let emit_external_selector = Felt252Wrapper::from(get_selector_from_name("emit_external").unwrap());

        let expected_emitted_internal_event_hash = get_selector_from_name("internal").unwrap();
        let expected_emitted_external_event_hash = get_selector_from_name("external").unwrap();

        let emit_internal_event_transaction = InvokeTransactionV1 {
            sender_address: sender_account.into(),
            calldata: vec![
                emit_contract_address, // Token address
                emit_internal_selector,
                Felt252Wrapper::ZERO, // Calldata len
            ],
            nonce: Felt252Wrapper::ZERO,
            max_fee: u128::MAX,
            signature: vec![],
            offset_version: false,
        };

        let none_origin = RuntimeOrigin::none();

        assert_ok!(Starknet::invoke(none_origin, emit_internal_event_transaction.into()));

        let mut events = System::events().into_iter().filter_map(|event_record| match event_record.event {
            RuntimeEvent::Starknet(Event::StarknetEvent(e)) => Some(e),
            _ => None,
        });
        let first_event = events.next();
        assert_eq!(
            first_event.and_then(|e| e.content.keys.get(0).cloned()).unwrap(),
            EventKey(Felt252Wrapper::from(expected_emitted_internal_event_hash).into())
        );

        let do_two_event_transaction = InvokeTransactionV1 {
            sender_address: sender_account.into(),
            calldata: vec![
                emit_contract_address, // Token address
                emit_external_selector,
                Felt252Wrapper::ZERO, // Calldata len
            ],
            nonce: Felt252Wrapper::ONE,
            max_fee: u128::MAX,
            signature: vec![],
            offset_version: false,
        };

        let none_origin = RuntimeOrigin::none();

        assert_ok!(Starknet::invoke(none_origin, do_two_event_transaction.clone().into()));

        let chain_id = Starknet::chain_id();
        let do_two_hash: TransactionHash =
            do_two_event_transaction.compute_hash::<<MockRuntime as Config>::SystemHash>(chain_id, false).into();
        let events = Starknet::tx_events(do_two_hash);
        assert_eq!(
            events[0].content.keys[0],
            EventKey(Felt252Wrapper::from(expected_emitted_external_event_hash).into())
        );
    });
}

#[test]
fn given_hardcoded_contract_run_storage_read_and_write_it_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();
        let transaction = get_storage_read_write_dummy();

        let transaction = transaction.into();

        let target_contract_address = ContractAddress(PatriciaKey(
            StarkFelt::try_from("024d1e355f6b9d27a5a420c8f4b50cea9154a8e34ad30fc39d7c98d3c177d0d7").unwrap(),
        ));
        let storage_var_selector = StorageKey(PatriciaKey(StarkFelt::from(25_u128)));

        assert_ok!(Starknet::invoke(none_origin, transaction));
        assert_eq!(Starknet::storage((target_contract_address, storage_var_selector)), StarkFelt::from(1u128));
    });
}

#[test]
fn test_verify_nonce() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let tx = get_invoke_dummy(Felt252Wrapper::ZERO);

        // Test for a valid nonce (0)
        assert_ok!(Starknet::invoke(RuntimeOrigin::none(), tx.into()));

        // Test for an invalid nonce (actual: 0, expected: 1)
        let tx_2 = get_invoke_dummy(Felt252Wrapper::ZERO);

        assert_err!(
            Starknet::invoke(RuntimeOrigin::none(), tx_2.into()),
            Error::<MockRuntime>::TransactionExecutionFailed
        );
    });
}

#[test]
fn given_hardcoded_contract_run_invoke_on_openzeppelin_account_then_it_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();

        let transaction: InvokeTransaction = get_invoke_openzeppelin_dummy().into();

        let validate_result = Starknet::validate_unsigned(
            TransactionSource::InBlock,
            &crate::Call::invoke { transaction: transaction.clone() },
        );
        assert_ok!(validate_result);

        assert_ok!(Starknet::invoke(none_origin, transaction));
    });
}

#[test]
fn given_hardcoded_contract_run_invoke_on_openzeppelin_account_with_incorrect_signature_then_it_fails() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();

        let mut transaction: InvokeTransactionV1 = get_invoke_openzeppelin_dummy();
        // by default we get valid signature so set it to something invalid
        transaction.signature = vec![Felt252Wrapper::ONE, Felt252Wrapper::ONE];

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
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();

        let chain_id = Starknet::chain_id();
        let mut transaction: InvokeTransactionV1 = get_invoke_argent_dummy();
        let tx_hash = transaction.compute_hash::<<MockRuntime as Config>::SystemHash>(chain_id, false);
        transaction.signature = sign_message_hash(tx_hash);

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
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();

        let mut transaction = get_invoke_argent_dummy();
        transaction.signature = vec![Felt252Wrapper::ONE, Felt252Wrapper::ONE];

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
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();

        let chain_id = Starknet::chain_id();
        let mut transaction: InvokeTransactionV1 = get_invoke_braavos_dummy();
        let tx_hash = transaction.compute_hash::<<MockRuntime as Config>::SystemHash>(chain_id, false);
        transaction.signature = sign_message_hash(tx_hash);

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
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();

        let mut transaction = get_invoke_braavos_dummy();
        transaction.signature = vec![Felt252Wrapper::ONE, Felt252Wrapper::ONE];

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
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();

        let sender_address = get_account_address(None, AccountType::V0(AccountTypeV0Inner::InnerCall));
        let mut transaction: InvokeTransactionV1 = get_invoke_dummy(Felt252Wrapper::ZERO);
        transaction.signature = vec![Felt252Wrapper::ONE, Felt252Wrapper::ONE];
        transaction.sender_address = sender_address.into();

        let storage_key = get_storage_var_address("destination", &[]).unwrap();
        let destination = StarkFelt::try_from(TEST_CONTRACT_ADDRESS).unwrap();
        StorageView::<MockRuntime>::insert((sender_address, storage_key), destination);

        let storage_key = get_storage_var_address("function_selector", &[]).unwrap();
        let selector = get_selector_from_name("without_arg").unwrap();
        StorageView::<MockRuntime>::insert(
            (sender_address, storage_key),
            StarkFelt::from(Felt252Wrapper::from(selector)),
        );

        assert_err!(
            Starknet::invoke(none_origin, transaction.into()),
            Error::<MockRuntime>::TransactionExecutionFailed
        );
    });
}

#[test]
fn given_account_not_deployed_invoke_tx_validate_works_for_nonce_one() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        // Wrong address (not deployed)
        let contract_address = Felt252Wrapper::from_hex_be("0x13123131").unwrap();

        let transaction = InvokeTransactionV1 {
            sender_address: contract_address,
            calldata: vec![],
            nonce: Felt252Wrapper::ONE,
            max_fee: u128::MAX,
            signature: vec![],
            offset_version: false,
        };

        assert_ok!(Starknet::validate_unsigned(
            TransactionSource::InBlock,
            &crate::Call::invoke { transaction: transaction.into() }
        ));
    })
}

#[test]
fn given_account_not_deployed_invoke_tx_fails_for_nonce_not_one() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        // Wrong address (not deployed)
        let contract_address = Felt252Wrapper::from_hex_be("0x13123131").unwrap();

        let transaction = InvokeTransactionV1 {
            sender_address: contract_address,
            calldata: vec![],
            nonce: Felt252Wrapper::TWO,
            max_fee: u128::MAX,
            signature: vec![],
            offset_version: false,
        };

        assert_eq!(
            Starknet::validate_unsigned(
                TransactionSource::InBlock,
                &crate::Call::invoke { transaction: transaction.into() }
            ),
            Err(TransactionValidityError::Invalid(InvalidTransaction::BadProof))
        );
    })
}

#[test]
fn test_verify_tx_longevity() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let transaction = get_invoke_dummy(Felt252Wrapper::ZERO);

        let validate_result = Starknet::validate_unsigned(
            TransactionSource::InBlock,
            &crate::Call::invoke { transaction: transaction.into() },
        );

        assert!(validate_result.unwrap().longevity == TransactionLongevity::get());
    });
}

#[test]
fn test_verify_require_tag() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let transaction = get_invoke_nonce_dummy();

        let validate_result = Starknet::validate_unsigned(
            TransactionSource::InBlock,
            &crate::Call::invoke { transaction: transaction.clone().into() },
        );

        let valid_transaction_expected = ValidTransaction::with_tag_prefix("starknet")
            .priority(u64::MAX)
            .and_provides((transaction.sender_address, transaction.nonce))
            .longevity(TransactionLongevity::get())
            .propagate(true)
            .and_requires((transaction.sender_address, Felt252Wrapper(transaction.nonce.0 - FieldElement::ONE)))
            .build();

        assert_eq!(validate_result.unwrap(), valid_transaction_expected.unwrap())
    });
}

#[test]
fn test_verify_nonce_in_unsigned_tx() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let transaction = get_invoke_dummy(Felt252Wrapper::ZERO);

        let tx_sender = transaction.sender_address.into();
        let tx_source = TransactionSource::InBlock;
        let call = Call::invoke { transaction: transaction.into() };

        assert!(Starknet::validate_unsigned(tx_source, &call).is_ok());

        set_nonce::<MockRuntime>(&tx_sender, &Nonce(StarkFelt::from(1u64)));

        assert_eq!(
            Starknet::validate_unsigned(tx_source, &call),
            Err(TransactionValidityError::Invalid(InvalidTransaction::Stale))
        );
    });
}
