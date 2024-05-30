use std::sync::Arc;

use blockifier::abi::abi_utils::get_storage_var_address;
use blockifier::execution::contract_class::ClassInfo;
use blockifier::transaction::transactions::{DeclareTransaction, InvokeTransaction};
use frame_support::{assert_err, assert_ok};
use mp_felt::Felt252Wrapper;
use mp_transactions::compute_hash::ComputeTransactionHash;
use pretty_assertions::assert_eq;
use sp_runtime::traits::ValidateUnsigned;
use sp_runtime::transaction_validity::{
    InvalidTransaction, TransactionSource, TransactionValidityError, ValidTransaction,
};
use starknet_api::core::{ClassHash, CompiledClassHash, ContractAddress, Nonce, PatriciaKey};
use starknet_api::hash::StarkFelt;
use starknet_api::state::StorageKey;
use starknet_api::transaction::{
    Calldata, ContractAddressSalt, Event as StarknetEvent, EventContent, EventData, EventKey, Fee, TransactionHash,
    TransactionSignature,
};
use starknet_core::utils::{get_selector_from_name, get_udc_deployed_address, UdcUniqueSettings, UdcUniqueness};
use starknet_crypto::FieldElement;

use super::constants::{
    BLOCKIFIER_ACCOUNT_ADDRESS, ETH_FEE_TOKEN_ADDRESS, MULTIPLE_EVENT_EMITTING_CONTRACT_ADDRESS,
    STRK_FEE_TOKEN_ADDRESS, TEST_CONTRACT_ADDRESS, TRANSFER_SELECTOR_NAME,
};
use super::mock::default_mock::*;
use super::mock::*;
use super::utils::{
    get_balance_contract_call, get_contract_class, set_account_erc20_balance_to_zero, sign_message_hash,
};
use crate::tests::constants::{UDC_ADDRESS, UDC_SELECTOR};
use crate::tests::{
    get_invoke_argent_dummy, get_invoke_braavos_dummy, get_invoke_dummy, get_invoke_emit_event_dummy,
    get_invoke_nonce_dummy, get_invoke_openzeppelin_dummy, get_invoke_v3_dummy, get_storage_read_write_dummy,
    set_infinite_tokens, set_nonce,
};
use crate::{Call, Error, StorageView};

const NONCE_ZERO: Nonce = Nonce(StarkFelt::ZERO);

#[test]
fn given_hardcoded_contract_run_invoke_tx_fails_sender_not_deployed() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();

        // Wrong address (not deployed)
        let contract_address = ContractAddress(PatriciaKey(
            StarkFelt::try_from("0x03e437FB56Bb213f5708Fcd6966502070e276c093ec271aA33433b89E21fd31f").unwrap(),
        ));

        let mut transaction = get_invoke_dummy(Starknet::chain_id(), NONCE_ZERO);
        if let starknet_api::transaction::InvokeTransaction::V1(tx) = &mut transaction.tx {
            tx.sender_address = contract_address;
        };

        assert_err!(Starknet::invoke(none_origin, transaction), Error::<MockRuntime>::AccountNotDeployed);
    })
}

#[test]
fn given_hardcoded_contract_run_invoke_tx_then_it_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let transaction = get_invoke_dummy(Starknet::chain_id(), NONCE_ZERO);

        assert_ok!(Starknet::invoke(RuntimeOrigin::none(), transaction));

        let pending_txs = Starknet::pending();
        pretty_assertions::assert_eq!(pending_txs.len(), 1);
        let pending_hashes = Starknet::pending_hashes();
        pretty_assertions::assert_eq!(pending_hashes.len(), 1);
        let tx_hash = TransactionHash(
            StarkFelt::try_from("0x02dfd0ded452658d67535279591c1ed9898431e1eafad7896239f0bfa68493d6").unwrap(),
        );

        assert_eq!(pending_hashes[0], tx_hash);
        let events: Vec<StarknetEvent> = Starknet::tx_events(tx_hash);
        println!("evens: {events:?}");

        assert!(events.into_iter().any(|e| e
            == StarknetEvent {
                from_address: Starknet::fee_token_addresses().eth_fee_token_address,
                content: EventContent {
                    keys: vec![EventKey(
                        Felt252Wrapper::from(get_selector_from_name(TRANSFER_SELECTOR_NAME).unwrap()).into(),
                    )],
                    data: EventData(vec![
                        StarkFelt::try_from(BLOCKIFIER_ACCOUNT_ADDRESS).unwrap(),
                        StarkFelt::try_from("0xdead").unwrap(),
                        StarkFelt::try_from("0x2f8").unwrap(),
                        StarkFelt::from(0u128),
                    ]),
                },
            },));
    });
}

#[test]
fn given_hardcoded_contract_run_invoke_tx_v0_then_it_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        // Declare the transaction as mutable
        let mut transaction = get_invoke_dummy(Starknet::chain_id(), NONCE_ZERO);
        if let starknet_api::transaction::InvokeTransaction::V0(tx) = &mut transaction.tx {
            tx.contract_address = ContractAddress(PatriciaKey(
                StarkFelt::try_from("0x03e437FB56Bb213f5708Fcd6966502070e276c093ec271aA33433b89E21fd31f").unwrap(),
            ));
            tx.calldata = Calldata(Arc::new(vec![])); // Empty calldata for simplicity
            tx.max_fee = Fee(0); // Adjusted field name
        };

        assert_ok!(Starknet::invoke(RuntimeOrigin::none(), transaction));

        let pending_txs = Starknet::pending();
        pretty_assertions::assert_eq!(pending_txs.len(), 1);
        let pending_hashes = Starknet::pending_hashes();
        pretty_assertions::assert_eq!(pending_hashes.len(), 1);
        let tx_hash = TransactionHash(
            StarkFelt::try_from("0x02dfd0ded452658d67535279591c1ed9898431e1eafad7896239f0bfa68493d6").unwrap(),
        );

        assert_eq!(pending_hashes[0], tx_hash);
        let events: Vec<StarknetEvent> = Starknet::tx_events(tx_hash);
        println!("events: {events:?}");

        assert!(events.into_iter().any(|e| e
            == StarknetEvent {
                from_address: Starknet::fee_token_addresses().eth_fee_token_address,
                content: EventContent {
                    keys: vec![EventKey(
                        Felt252Wrapper::from(get_selector_from_name(TRANSFER_SELECTOR_NAME).unwrap()).into(),
                    )],
                    data: EventData(vec![
                        StarkFelt::try_from(BLOCKIFIER_ACCOUNT_ADDRESS).unwrap(),
                        StarkFelt::try_from("0xdead").unwrap(),
                        StarkFelt::try_from("0x2f8").unwrap(),
                        StarkFelt::from(0u128),
                    ]),
                },
            },));
    });
}


#[test]
fn given_hardcoded_contract_run_invoke_tx_then_event_is_emitted() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();

        let transaction = get_invoke_emit_event_dummy(Starknet::chain_id());
        let tx_hash = transaction.tx_hash;

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
            from_address: Starknet::fee_token_addresses().eth_fee_token_address,
            content: EventContent {
                keys: vec![EventKey(
                    StarkFelt::try_from(Felt252Wrapper::from(get_selector_from_name(TRANSFER_SELECTOR_NAME).unwrap())).unwrap(),
                )],
                data: EventData(vec![
                    StarkFelt::try_from("0x01a3339ec92ac1061e3e0f8e704106286c642eaf302e94a582e5f95ef5e6b4d0").unwrap(), // From
                    StarkFelt::try_from("0xdead").unwrap(), // To
                    StarkFelt::try_from("0x2f8").unwrap(),  // Amount low
                    StarkFelt::from(0u128),                 // Amount high
                ]),
            },
        };
        let events: Vec<StarknetEvent> = Starknet::tx_events(tx_hash);

        // Actual event.
        pretty_assertions::assert_eq!(
            emitted_event.clone(),
            events[events.len() - 2]
        );
        // Fee transfer event.
        pretty_assertions::assert_eq!(
            expected_fee_transfer_event.clone(),
            events.last().unwrap().clone()
        );
    });
}

#[test]
fn given_hardcoded_contract_run_invoke_tx_then_multiple_events_is_emitted() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let chain_id = Starknet::chain_id();
        let emit_contract_address =
            ContractAddress(PatriciaKey(StarkFelt::try_from(MULTIPLE_EVENT_EMITTING_CONTRACT_ADDRESS).unwrap()));

        let sender_address = get_account_address(None, AccountType::V0(AccountTypeV0Inner::NoValidate));

        let emit_internal_selector = Felt252Wrapper::from(get_selector_from_name("emit_internal").unwrap()).into();
        let emit_external_selector = Felt252Wrapper::from(get_selector_from_name("emit_external").unwrap()).into();

        let expected_emitted_internal_event_hash = get_selector_from_name("internal").unwrap();
        let expected_emitted_external_event_hash = get_selector_from_name("external").unwrap();

        let emit_internal_event_tx = starknet_api::transaction::InvokeTransactionV1 {
            sender_address,
            calldata: Calldata(Arc::new(vec![
                emit_contract_address.0.0, // Token address
                emit_internal_selector,
                StarkFelt::ZERO, // Calldata len
            ])),
            nonce: Nonce(StarkFelt::ZERO),
            max_fee: Fee(u128::MAX),
            signature: TransactionSignature::default(),
        };

        let none_origin = RuntimeOrigin::none();

        let tx_hash = emit_internal_event_tx.compute_hash(chain_id, false);
        let transaction = InvokeTransaction { tx: emit_internal_event_tx.into(), tx_hash, only_query: false };

        assert_ok!(Starknet::invoke(none_origin.clone(), transaction));

        let events: Vec<StarknetEvent> = Starknet::tx_events(tx_hash);
        let first_event = events.first();
        assert_eq!(
            first_event.and_then(|e| e.content.keys.get(0).cloned()).unwrap(),
            EventKey(Felt252Wrapper::from(expected_emitted_internal_event_hash).into())
        );

        let do_two_event_tx = starknet_api::transaction::InvokeTransactionV1 {
            sender_address,
            calldata: Calldata(Arc::new(vec![
                emit_contract_address.0.0, // Token address
                emit_external_selector,
                StarkFelt::ZERO, // Calldata len
            ])),
            nonce: Nonce(StarkFelt::ONE),
            max_fee: Fee(u128::MAX),
            signature: TransactionSignature::default(),
        };
        let tx_hash = do_two_event_tx.compute_hash(chain_id, false);
        let transaction = InvokeTransaction { tx: do_two_event_tx.into(), tx_hash, only_query: false };

        assert_ok!(Starknet::invoke(none_origin, transaction));

        let events = Starknet::tx_events(tx_hash);
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
        let transaction = get_storage_read_write_dummy(Starknet::chain_id());

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

        let tx = get_invoke_dummy(Starknet::chain_id(), NONCE_ZERO);

        // Test for a valid nonce (0)
        assert_ok!(Starknet::invoke(RuntimeOrigin::none(), tx));

        // Test for an invalid nonce (actual: 0, expected: 1)
        let tx_2 = get_invoke_dummy(Starknet::chain_id(), NONCE_ZERO);

        assert_err!(Starknet::invoke(RuntimeOrigin::none(), tx_2), Error::<MockRuntime>::TransactionExecutionFailed);
    });
}

#[test]
fn given_hardcoded_contract_run_invoke_on_openzeppelin_account_then_it_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();

        let transaction = get_invoke_openzeppelin_dummy(Starknet::chain_id());

        assert_ok!(Starknet::invoke(none_origin, transaction));
    });
}

#[test]
fn given_hardcoded_contract_run_invoke_on_openzeppelin_account_with_incorrect_signature_then_it_fails() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();

        let mut transaction = get_invoke_openzeppelin_dummy(Starknet::chain_id());
        // by default we get valid signature so set it to something invalid
        if let starknet_api::transaction::InvokeTransaction::V1(tx) = &mut transaction.tx {
            tx.signature = TransactionSignature(vec![StarkFelt::ONE, StarkFelt::ONE]);
        };

        let validate_result = Starknet::validate_unsigned(
            TransactionSource::InBlock,
            &crate::Call::invoke { transaction: transaction.clone() },
        );
        assert!(matches!(validate_result.unwrap_err(), TransactionValidityError::Invalid(_)));

        assert_err!(Starknet::invoke(none_origin, transaction), Error::<MockRuntime>::TransactionExecutionFailed);
    });
}

#[test]
fn given_hardcoded_contract_run_invoke_on_argent_account_then_it_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();

        let chain_id = Starknet::chain_id();
        let mut transaction = get_invoke_argent_dummy(chain_id);
        if let starknet_api::transaction::InvokeTransaction::V1(tx) = &mut transaction.tx {
            tx.signature = sign_message_hash(transaction.tx_hash);
        };

        assert_ok!(Starknet::invoke(none_origin, transaction));
    });
}

#[test]
fn given_hardcoded_contract_run_invoke_on_argent_account_with_incorrect_signature_then_it_fails() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();

        let mut transaction = get_invoke_argent_dummy(Starknet::chain_id());
        if let starknet_api::transaction::InvokeTransaction::V1(tx) = &mut transaction.tx {
            tx.signature = TransactionSignature(vec![StarkFelt::ONE, StarkFelt::ONE]);
        };

        let validate_result = Starknet::validate_unsigned(
            TransactionSource::InBlock,
            &crate::Call::invoke { transaction: transaction.clone() },
        );
        assert!(matches!(validate_result.unwrap_err(), TransactionValidityError::Invalid(_)));

        assert_err!(Starknet::invoke(none_origin, transaction), Error::<MockRuntime>::TransactionExecutionFailed);
    });
}

#[test]
fn given_hardcoded_contract_run_invoke_on_braavos_account_then_it_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();

        let mut transaction = get_invoke_braavos_dummy(Starknet::chain_id());
        if let starknet_api::transaction::InvokeTransaction::V1(tx) = &mut transaction.tx {
            tx.signature = sign_message_hash(transaction.tx_hash);
        };

        assert_ok!(Starknet::invoke(none_origin, transaction));
    });
}

#[test]
fn given_hardcoded_contract_run_invoke_on_braavos_account_with_incorrect_signature_then_it_fails() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();

        let mut transaction = get_invoke_braavos_dummy(Starknet::chain_id());
        if let starknet_api::transaction::InvokeTransaction::V1(tx) = &mut transaction.tx {
            tx.signature = TransactionSignature(vec![StarkFelt::ONE, StarkFelt::ONE]);
        };

        let validate_result = Starknet::validate_unsigned(
            TransactionSource::InBlock,
            &crate::Call::invoke { transaction: transaction.clone() },
        );
        assert!(matches!(validate_result.unwrap_err(), TransactionValidityError::Invalid(_)));

        assert_err!(Starknet::invoke(none_origin, transaction), Error::<MockRuntime>::TransactionExecutionFailed);
    });
}

#[test]
fn given_hardcoded_contract_run_invoke_with_inner_call_in_validate_then_it_fails() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();

        let sender_address = get_account_address(None, AccountType::V0(AccountTypeV0Inner::InnerCall));
        let mut transaction = get_invoke_dummy(Starknet::chain_id(), NONCE_ZERO);
        if let starknet_api::transaction::InvokeTransaction::V1(tx) = &mut transaction.tx {
            tx.signature = TransactionSignature(vec![StarkFelt::ONE, StarkFelt::ONE]);
            tx.sender_address = sender_address;
        };

        let storage_key = get_storage_var_address("destination", &[]);
        let destination = StarkFelt::try_from(TEST_CONTRACT_ADDRESS).unwrap();
        StorageView::<MockRuntime>::insert((sender_address, storage_key), destination);

        let storage_key = get_storage_var_address("function_selector", &[]);
        let selector = get_selector_from_name("without_arg").unwrap();
        StorageView::<MockRuntime>::insert(
            (sender_address, storage_key),
            StarkFelt::from(Felt252Wrapper::from(selector)),
        );

        assert_err!(Starknet::invoke(none_origin, transaction), Error::<MockRuntime>::TransactionExecutionFailed);
    });
}

#[test]
fn given_account_not_deployed_invoke_tx_validate_works_for_nonce_one() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        // Wrong address (not deployed)
        let contract_address = ContractAddress(PatriciaKey(StarkFelt::try_from("0x13123131").unwrap()));

        let transaction = starknet_api::transaction::InvokeTransactionV1 {
            sender_address: contract_address,
            calldata: Calldata::default(),
            nonce: Nonce(StarkFelt::ONE),
            max_fee: Fee(u128::MAX),
            signature: TransactionSignature::default(),
        };

        set_infinite_tokens::<MockRuntime>(&contract_address);
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
        let contract_address = ContractAddress(PatriciaKey(StarkFelt::try_from("0x13123131").unwrap()));

        let transaction = starknet_api::transaction::InvokeTransactionV1 {
            sender_address: contract_address,
            calldata: Calldata::default(),
            nonce: Nonce(StarkFelt::TWO),
            max_fee: Fee(u128::MAX),
            signature: TransactionSignature::default(),
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

        let transaction = get_invoke_dummy(Starknet::chain_id(), NONCE_ZERO);

        let validate_result =
            Starknet::validate_unsigned(TransactionSource::InBlock, &crate::Call::invoke { transaction });

        assert!(validate_result.unwrap().longevity == TransactionLongevity::get());
    });
}

#[test]
fn test_verify_require_tag() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let transaction = get_invoke_nonce_dummy(Starknet::chain_id());

        let validate_result = Starknet::validate_unsigned(
            TransactionSource::InBlock,
            &crate::Call::invoke { transaction: transaction.clone() },
        );

        let valid_transaction_expected = ValidTransaction::with_tag_prefix("starknet")
            .priority(u64::MAX)
            .and_provides((transaction.tx.sender_address(), transaction.tx.nonce()))
            .longevity(TransactionLongevity::get())
            .propagate(true)
            .and_requires((
                transaction.tx.sender_address(),
                Nonce::from(Felt252Wrapper::from(Felt252Wrapper::from(transaction.tx.nonce()).0 - FieldElement::ONE)),
            ))
            .build();

        assert_eq!(validate_result.unwrap(), valid_transaction_expected.unwrap())
    });
}

#[test]
fn test_verify_nonce_in_unsigned_tx() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let transaction = get_invoke_dummy(Starknet::chain_id(), NONCE_ZERO);

        let tx_sender = transaction.tx.sender_address();
        let tx_source = TransactionSource::InBlock;
        let call = Call::invoke { transaction };

        assert!(Starknet::validate_unsigned(tx_source, &call).is_ok());

        set_nonce::<MockRuntime>(&tx_sender, &Nonce(StarkFelt::from(1u64)));

        assert_eq!(
            Starknet::validate_unsigned(tx_source, &call),
            Err(TransactionValidityError::Invalid(InvalidTransaction::BadProof))
        );
    });
}

#[test]
#[ignore]
fn storage_changes_should_revert_on_transaction_revert() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let chain_id = Starknet::chain_id();
        let none_origin = RuntimeOrigin::none();

        let account_addr = get_account_address(None, AccountType::V1(AccountTypeV1Inner::NoValidate));

        let transaction_revert_class = get_contract_class("TransactionRevert.casm.json", 1);
        let transaction_revert_class_hash = ClassHash(
            StarkFelt::try_from("0x7d2bcb1df4970245665a19b23a4d3877eb86a661e8d98b89afc4531134b99f6").unwrap(),
        );
        let transaction_revert_compiled_class_hash = CompiledClassHash(
            StarkFelt::try_from("0x1c02b663e928ed213d3a0fa206efb59182fa2ba41f5c204daa56c4a434b53e5").unwrap(),
        );

        let mut declare_tx = starknet_api::transaction::DeclareTransactionV2 {
            sender_address: account_addr,
            class_hash: transaction_revert_class_hash,
            compiled_class_hash: transaction_revert_compiled_class_hash,
            nonce: Nonce::default(),
            max_fee: Fee(u128::MAX),
            signature: TransactionSignature(vec![]),
        };

        let tx_hash = declare_tx.compute_hash(chain_id, false);
        declare_tx.signature = sign_message_hash(tx_hash);

        let transaction = DeclareTransaction::new(
            starknet_api::transaction::DeclareTransaction::V2(declare_tx),
            tx_hash,
            ClassInfo::new(&transaction_revert_class, 1, 1).unwrap(),
        )
        .unwrap();

        assert_ok!(Starknet::declare(none_origin, transaction));
        assert_eq!(
            Starknet::contract_class_by_class_hash(transaction_revert_class_hash.0).unwrap(),
            transaction_revert_class
        );

        let salt = ContractAddressSalt(StarkFelt::ZERO);

        let mut invoke_tx = starknet_api::transaction::InvokeTransactionV1 {
            sender_address: account_addr,
            signature: TransactionSignature(vec![]),
            nonce: Nonce(StarkFelt::ONE),
            calldata: Calldata(Arc::new(vec![
                StarkFelt::ONE,
                StarkFelt::try_from(UDC_ADDRESS).unwrap(),  // udc address
                StarkFelt::try_from(UDC_SELECTOR).unwrap(), // deployContract selector
                StarkFelt::try_from("0x4").unwrap(),        // calldata len
                transaction_revert_class_hash.0,            // contract class hash
                salt.0,                                     // salt
                StarkFelt::ONE,                             // unique
                StarkFelt::ZERO,                            // constructor calldata len
            ])),
            max_fee: Fee(u128::MAX),
        };

        let tx_hash = invoke_tx.compute_hash(chain_id, false);
        invoke_tx.signature = sign_message_hash(tx_hash);
        let transaction = InvokeTransaction {
            tx: starknet_api::transaction::InvokeTransaction::V1(invoke_tx),
            tx_hash,
            only_query: false,
        };
        let contract_address: FieldElement = get_udc_deployed_address(
            Felt252Wrapper::from(salt).into(),
            Felt252Wrapper::from(transaction_revert_class_hash).into(),
            &UdcUniqueness::Unique(UdcUniqueSettings {
                deployer_address: Felt252Wrapper::from(account_addr).into(),
                udc_contract_address: FieldElement::from_hex_be(UDC_ADDRESS).unwrap(),
            }),
            &[],
        );
        let contract_address: ContractAddress = Felt252Wrapper::from(contract_address).into();

        // deploy contract
        assert_ok!(Starknet::invoke(RuntimeOrigin::none(), transaction));

        let increase_balance_function_selector = get_selector_from_name("increase_balance").unwrap();

        // create increase balance transaction
        let increase_balance_tx = starknet_api::transaction::InvokeTransactionV1 {
            sender_address: account_addr,
            signature: TransactionSignature(vec![]),
            nonce: Nonce(StarkFelt::TWO),
            max_fee: Fee(u128::MAX),
            calldata: Calldata(Arc::new(vec![
                StarkFelt::ONE,
                contract_address.0.0,
                Felt252Wrapper::from(increase_balance_function_selector).into(),
                StarkFelt::try_from("0x1").unwrap(),
                StarkFelt::try_from("0xa").unwrap(),
            ])),
        };

        // the transaction reverts and returns Ok
        assert_ok!(Starknet::invoke(RuntimeOrigin::none(), increase_balance_tx.clone().into()));

        // the storage value should be 0 after the transaction reverts
        let get_balance_function_selector = get_selector_from_name("get_balance").unwrap();

        let get_balance_function_selector_entrypoint = Felt252Wrapper::from(get_balance_function_selector).into();

        let default_calldata = Calldata(Default::default());

        let balance_value =
            Starknet::call_contract(contract_address, get_balance_function_selector_entrypoint, default_calldata)
                .unwrap();
        assert_eq!(balance_value, vec![Felt252Wrapper::ZERO])
    })
}

#[test]
fn given_hardcoded_contract_run_invoke_v1_without_strk_with_eth_fee_token_it_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();

        // Account that gonna make the transaction
        let sender_address = get_account_address(None, AccountType::V0(AccountTypeV0Inner::NoValidate));

        // Ethereum fee token contract address
        let eth_fee_contract_address =
            ContractAddress(PatriciaKey(StarkFelt::try_from(ETH_FEE_TOKEN_ADDRESS).unwrap()));

        // Starknet fee token contract address
        let strk_fee_contract_address =
            ContractAddress(PatriciaKey(StarkFelt::try_from(STRK_FEE_TOKEN_ADDRESS).unwrap()));

        let mut transaction = get_invoke_dummy(Starknet::chain_id(), NONCE_ZERO);

        if let starknet_api::transaction::InvokeTransaction::V1(tx) = &mut transaction.tx {
            tx.sender_address = sender_address;
        };

        set_account_erc20_balance_to_zero(sender_address, strk_fee_contract_address);

        let eth_initial_balance_vec = get_balance_contract_call(sender_address, eth_fee_contract_address);

        // Ensure that eth fee token balance is not empty
        assert_eq!(
            eth_initial_balance_vec,
            vec![
                Felt252Wrapper::from_hex_be("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap(),
                Felt252Wrapper::from_hex_be("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap()
            ]
        );

        assert_ok!(Starknet::invoke(none_origin, transaction));
        let eth_final_balance_vec = get_balance_contract_call(sender_address, eth_fee_contract_address);

        // Ensure ETH is consumed and STRK balance still the same
        assert!(eth_final_balance_vec[1] == eth_initial_balance_vec[1]);
        assert!(eth_final_balance_vec[0] < eth_initial_balance_vec[0]);
    });
}

#[test]
fn given_hardcoded_contract_run_invoke_v1_without_eth_with_strk_fee_token_it_fails() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();

        // Account that gonna make the transaction
        let sender_address = get_account_address(None, AccountType::V0(AccountTypeV0Inner::NoValidate));

        // Ethereum fee token contract address
        let eth_fee_contract_address =
            ContractAddress(PatriciaKey(StarkFelt::try_from(ETH_FEE_TOKEN_ADDRESS).unwrap()));

        // Starknet fee token contract address
        let strk_fee_contract_address =
            ContractAddress(PatriciaKey(StarkFelt::try_from(STRK_FEE_TOKEN_ADDRESS).unwrap()));

        let mut transaction = get_invoke_dummy(Starknet::chain_id(), NONCE_ZERO);

        if let starknet_api::transaction::InvokeTransaction::V1(tx) = &mut transaction.tx {
            tx.sender_address = sender_address;
        };

        set_account_erc20_balance_to_zero(sender_address, eth_fee_contract_address);

        // Ensure that strk fee token balance is not empty
        assert_eq!(
            get_balance_contract_call(sender_address, strk_fee_contract_address),
            vec![
                Felt252Wrapper::from_hex_be("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap(),
                Felt252Wrapper::from_hex_be("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap()
            ]
        );

        assert_err!(Starknet::invoke(none_origin, transaction), Error::<MockRuntime>::TransactionExecutionFailed);
    });
}

#[test]
fn given_hardcoded_contract_run_invoke_v3_without_fees_token_it_fails() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();

        // Account that gonna make the transaction
        let sender_address = get_account_address(None, AccountType::V0(AccountTypeV0Inner::NoValidate));

        // Ethereum fee token contract address
        let eth_fee_contract_address =
            ContractAddress(PatriciaKey(StarkFelt::try_from(ETH_FEE_TOKEN_ADDRESS).unwrap()));

        // Starknet fee token contract address
        let strk_fee_contract_address =
            ContractAddress(PatriciaKey(StarkFelt::try_from(STRK_FEE_TOKEN_ADDRESS).unwrap()));

        let mut transaction = get_invoke_v3_dummy(Starknet::chain_id(), NONCE_ZERO);

        if let starknet_api::transaction::InvokeTransaction::V3(tx) = &mut transaction.tx {
            tx.sender_address = sender_address;
        };

        set_account_erc20_balance_to_zero(sender_address, eth_fee_contract_address);
        set_account_erc20_balance_to_zero(sender_address, strk_fee_contract_address);

        assert_err!(Starknet::invoke(none_origin, transaction), Error::<MockRuntime>::TransactionExecutionFailed);
    });
}

#[test]
fn given_hardcoded_contract_run_invoke_v3_without_eth_with_strk_fee_token_it_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();

        // Account that gonna make the transaction
        let sender_address = get_account_address(None, AccountType::V0(AccountTypeV0Inner::NoValidate));

        // Ethereum fee token contract address
        let eth_fee_contract_address =
            ContractAddress(PatriciaKey(StarkFelt::try_from(ETH_FEE_TOKEN_ADDRESS).unwrap()));

        // Starknet fee token contract address
        let strk_fee_contract_address =
            ContractAddress(PatriciaKey(StarkFelt::try_from(STRK_FEE_TOKEN_ADDRESS).unwrap()));

        let mut transaction = get_invoke_v3_dummy(Starknet::chain_id(), NONCE_ZERO);

        if let starknet_api::transaction::InvokeTransaction::V3(tx) = &mut transaction.tx {
            tx.sender_address = sender_address;
        };

        set_account_erc20_balance_to_zero(sender_address, eth_fee_contract_address);

        let initial_balance_vec = get_balance_contract_call(sender_address, strk_fee_contract_address);

        // Ensure that strk fee token balance is not empty
        assert_eq!(
            initial_balance_vec,
            vec![
                Felt252Wrapper::from_hex_be("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap(),
                Felt252Wrapper::from_hex_be("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap()
            ]
        );

        assert_ok!(Starknet::invoke(none_origin, transaction));
        let final_balance_vec = get_balance_contract_call(sender_address, strk_fee_contract_address);
        assert!(final_balance_vec[1] == initial_balance_vec[1]);
        assert!(final_balance_vec[0] < initial_balance_vec[0]);
    });
}

#[test]
fn given_hardcoded_contract_run_invoke_v3_without_strk_with_eth_fee_token_it_fails() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();

        // Account that gonna make the transaction
        let sender_address = get_account_address(None, AccountType::V0(AccountTypeV0Inner::NoValidate));

        // Ethereum fee token contract address
        let eth_fee_contract_address =
            ContractAddress(PatriciaKey(StarkFelt::try_from(ETH_FEE_TOKEN_ADDRESS).unwrap()));

        // Starknet fee token contract address
        let strk_fee_contract_address =
            ContractAddress(PatriciaKey(StarkFelt::try_from(STRK_FEE_TOKEN_ADDRESS).unwrap()));

        let mut transaction = get_invoke_v3_dummy(Starknet::chain_id(), NONCE_ZERO);

        if let starknet_api::transaction::InvokeTransaction::V3(tx) = &mut transaction.tx {
            tx.sender_address = sender_address;
        };

        set_account_erc20_balance_to_zero(sender_address, strk_fee_contract_address);

        // Ensure that eth fee token balance is not empty
        assert_eq!(
            get_balance_contract_call(sender_address, eth_fee_contract_address),
            vec![
                Felt252Wrapper::from_hex_be("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap(),
                Felt252Wrapper::from_hex_be("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap()
            ]
        );

        assert_err!(Starknet::invoke(none_origin, transaction), Error::<MockRuntime>::TransactionExecutionFailed);
    });
}

#[test]
fn given_hardcoded_contract_run_invoke_v3_with_eth_with_strk_fees_token_it_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();

        // Account that gonna make the transaction
        let sender_address = get_account_address(None, AccountType::V0(AccountTypeV0Inner::NoValidate));

        // Ethereum fee token contract address
        let eth_fee_contract_address =
            ContractAddress(PatriciaKey(StarkFelt::try_from(ETH_FEE_TOKEN_ADDRESS).unwrap()));

        // Starknet fee token contract address
        let strk_fee_contract_address =
            ContractAddress(PatriciaKey(StarkFelt::try_from(STRK_FEE_TOKEN_ADDRESS).unwrap()));

        let mut transaction = get_invoke_v3_dummy(Starknet::chain_id(), NONCE_ZERO);

        if let starknet_api::transaction::InvokeTransaction::V3(tx) = &mut transaction.tx {
            tx.sender_address = sender_address;
        };

        let strk_initial_balance_vec = get_balance_contract_call(sender_address, strk_fee_contract_address);
        let eth_initial_balance_vec = get_balance_contract_call(sender_address, eth_fee_contract_address);

        // Ensure that strk fee token balance is not empty
        assert_eq!(
            strk_initial_balance_vec,
            vec![
                Felt252Wrapper::from_hex_be("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap(),
                Felt252Wrapper::from_hex_be("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap()
            ]
        );

        assert_eq!(
            eth_initial_balance_vec,
            vec![
                Felt252Wrapper::from_hex_be("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap(),
                Felt252Wrapper::from_hex_be("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap()
            ]
        );

        assert_ok!(Starknet::invoke(none_origin, transaction));
        let strk_final_balance_vec = get_balance_contract_call(sender_address, strk_fee_contract_address);
        let eth_final_balance_vec = get_balance_contract_call(sender_address, eth_fee_contract_address);

        // Ensure STRK is consumed and ETH balance still the same
        assert!(strk_final_balance_vec[1] == strk_initial_balance_vec[1]);
        assert!(strk_final_balance_vec[0] < strk_initial_balance_vec[0]);
        assert!(eth_final_balance_vec[1] == eth_initial_balance_vec[1]);
        assert!(eth_final_balance_vec[0] == eth_initial_balance_vec[0]);
    });
}

#[test]
fn given_hardcoded_contract_set_erc20_balance_to_zero() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        // Account that gonna make the transaction
        let account_address = get_account_address(None, AccountType::V0(AccountTypeV0Inner::NoValidate));

        // ERC20 contract address (ETH in this case)
        let erc20_contract_address = ContractAddress(PatriciaKey(StarkFelt::try_from(ETH_FEE_TOKEN_ADDRESS).unwrap()));

        let erc20_initial_balance_vec = get_balance_contract_call(account_address, erc20_contract_address);

        assert_eq!(
            erc20_initial_balance_vec,
            vec![
                Felt252Wrapper::from_hex_be("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap(),
                Felt252Wrapper::from_hex_be("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap()
            ]
        );

        set_account_erc20_balance_to_zero(account_address, erc20_contract_address);
        let erc20_final_balance_vec = get_balance_contract_call(account_address, erc20_contract_address);

        // Ensure ERC20 balance of account is zero
        assert_eq!(
            erc20_final_balance_vec,
            vec![Felt252Wrapper::from_hex_be("0x0").unwrap(), Felt252Wrapper::from_hex_be("0x0").unwrap()]
        );
    });
}