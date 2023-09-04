use frame_support::{assert_err, assert_ok, bounded_vec};
use mp_starknet::crypto::commitment::calculate_declare_tx_hash;
use mp_starknet::execution::types::Felt252Wrapper;
use mp_starknet::transaction::types::DeclareTransaction;
use sp_runtime::traits::ValidateUnsigned;
use sp_runtime::transaction_validity::{TransactionSource, TransactionValidityError, ValidTransaction};
use starknet_crypto::FieldElement;

use super::mock::default_mock::*;
use super::mock::*;
use super::utils::{get_contract_class, sign_message_hash};
use crate::tests::get_declare_dummy;
use crate::Error;

#[test]
fn given_contract_declare_tx_works_once_not_twice() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();
        let account_addr = get_account_address(AccountType::V0(AccountTypeV0Inner::NoValidate));

        let erc20_class = get_contract_class("ERC20.json", 0);
        let erc20_class_hash =
            Felt252Wrapper::from_hex_be("0x057eca87f4b19852cfd4551cf4706ababc6251a8781733a0a11cf8e94211da95").unwrap();

        let transaction = DeclareTransaction {
            sender_address: account_addr,
            version: 1,
            class_hash: erc20_class_hash,
            compiled_class_hash: None,
            contract_class: erc20_class,
            nonce: Felt252Wrapper::ZERO,
            max_fee: Felt252Wrapper::from(u64::MAX),
            signature: bounded_vec!(),
            is_query: false,
        };

        assert_ok!(Starknet::declare(none_origin.clone(), transaction.clone()));
        // TODO: Uncomment once we have ABI support
        // assert_eq!(Starknet::contract_class_by_class_hash(erc20_class_hash), erc20_class);
        assert_err!(Starknet::declare(none_origin, transaction), Error::<MockRuntime>::ClassHashAlreadyDeclared);
    });
}

#[test]
fn given_contract_declare_tx_fails_sender_not_deployed() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();

        // Wrong address (not deployed)
        let contract_address =
            Felt252Wrapper::from_hex_be("0x03e437FB56Bb213f5708Fcd6966502070e276c093ec271aA33433b89E21fd31f").unwrap();

        let erc20_class = get_contract_class("ERC20.json", 0);
        let erc20_class_hash =
            Felt252Wrapper::from_hex_be("0x057eca87f4b19852cfd4551cf4706ababc6251a8781733a0a11cf8e94211da95").unwrap();

        let transaction = DeclareTransaction {
            sender_address: contract_address,
            contract_class: erc20_class,
            version: 1,
            class_hash: erc20_class_hash,
            compiled_class_hash: None,
            nonce: Felt252Wrapper::ZERO,
            max_fee: Felt252Wrapper::from(u64::MAX),
            signature: bounded_vec!(),
            is_query: false,
        };

        assert_err!(Starknet::declare(none_origin, transaction), Error::<MockRuntime>::AccountNotDeployed);
    })
}

#[test]
fn given_contract_declare_tx_fails_wrong_tx_version() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();
        let account_addr = get_account_address(AccountType::V0(AccountTypeV0Inner::Argent));

        let erc20_class = get_contract_class("ERC20.json", 0);
        // TODO: Delete when the class hash can be derived from ContractClass
        let erc20_class_hash =
            Felt252Wrapper::from_hex_be("0x057eca87f4b19852cfd4551cf4706ababc6251a8781733a0a11cf8e94211da95").unwrap();

        let wrong_tx_version = 50_u8;

        let transaction = DeclareTransaction {
            sender_address: account_addr,
            contract_class: erc20_class,
            version: wrong_tx_version,
            class_hash: erc20_class_hash,
            compiled_class_hash: None,
            nonce: Felt252Wrapper::ZERO,
            max_fee: Felt252Wrapper::from(u64::MAX),
            signature: bounded_vec!(),
            is_query: false,
        };

        assert_err!(Starknet::declare(none_origin, transaction), Error::<MockRuntime>::TransactionExecutionFailed);
    })
}

#[test]
fn given_contract_declare_on_openzeppelin_account_then_it_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();

        let mut transaction = get_declare_dummy(AccountType::V0(AccountTypeV0Inner::Openzeppelin));
        let erc20_class_hash = transaction.class_hash;

        let chain_id = Starknet::chain_id();
        let transaction_hash = calculate_declare_tx_hash(transaction.clone(), chain_id);
        transaction.signature = sign_message_hash(transaction_hash);

        let validate_result = Starknet::validate_unsigned(
            TransactionSource::InBlock,
            &crate::Call::declare { transaction: transaction.clone() },
        );
        assert_ok!(validate_result);

        assert_ok!(Starknet::declare(none_origin, transaction));
        assert_eq!(
            Starknet::contract_class_by_class_hash(erc20_class_hash).unwrap(),
            get_contract_class("ERC20.json", 0)
        );
    });
}

#[test]
fn given_contract_declare_on_openzeppelin_account_with_incorrect_signature_then_it_fails() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();

        let mut transaction = get_declare_dummy(AccountType::V0(AccountTypeV0Inner::Openzeppelin));
        transaction.signature = bounded_vec!(Felt252Wrapper::ZERO, Felt252Wrapper::ONE);

        let validate_result = Starknet::validate_unsigned(
            TransactionSource::InBlock,
            &crate::Call::declare { transaction: transaction.clone() },
        );
        assert!(matches!(validate_result.unwrap_err(), TransactionValidityError::Invalid(_)));

        assert_err!(Starknet::declare(none_origin, transaction), Error::<MockRuntime>::TransactionExecutionFailed);
    });
}

#[test]
fn given_contract_declare_on_braavos_account_then_it_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();

        let mut transaction = get_declare_dummy(AccountType::V0(AccountTypeV0Inner::Braavos));
        let erc20_class_hash = transaction.class_hash;

        let chain_id = Starknet::chain_id();
        let transaction_hash = calculate_declare_tx_hash(transaction.clone(), chain_id);
        transaction.signature = sign_message_hash(transaction_hash);

        let validate_result = Starknet::validate_unsigned(
            TransactionSource::InBlock,
            &crate::Call::declare { transaction: transaction.clone() },
        );
        assert_ok!(validate_result);

        assert_ok!(Starknet::declare(none_origin, transaction));
        assert_eq!(
            Starknet::contract_class_by_class_hash(erc20_class_hash).unwrap(),
            get_contract_class("ERC20.json", 0)
        );
    });
}

#[test]
fn given_contract_declare_on_braavos_account_with_incorrect_signature_then_it_fails() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();

        let mut transaction = get_declare_dummy(AccountType::V0(AccountTypeV0Inner::Braavos));
        transaction.signature = bounded_vec!(Felt252Wrapper::ZERO, Felt252Wrapper::ONE);

        let validate_result = Starknet::validate_unsigned(
            TransactionSource::InBlock,
            &crate::Call::declare { transaction: transaction.clone() },
        );
        assert!(matches!(validate_result.unwrap_err(), TransactionValidityError::Invalid(_)));

        assert_err!(Starknet::declare(none_origin, transaction), Error::<MockRuntime>::TransactionExecutionFailed);
    });
}

#[test]
fn given_contract_declare_on_argent_account_then_it_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();

        let mut transaction = get_declare_dummy(AccountType::V0(AccountTypeV0Inner::Argent));
        let erc20_class_hash = transaction.class_hash;

        let chain_id = Starknet::chain_id();
        let transaction_hash = calculate_declare_tx_hash(transaction.clone(), chain_id);
        transaction.signature = sign_message_hash(transaction_hash);

        let validate_result = Starknet::validate_unsigned(
            TransactionSource::InBlock,
            &crate::Call::declare { transaction: transaction.clone() },
        );
        assert_ok!(validate_result);

        assert_ok!(Starknet::declare(none_origin, transaction));
        assert_eq!(
            Starknet::contract_class_by_class_hash(erc20_class_hash).unwrap(),
            get_contract_class("ERC20.json", 0)
        );
    });
}

#[test]
fn given_contract_declare_on_argent_account_with_incorrect_signature_then_it_fails() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();

        let mut transaction = get_declare_dummy(AccountType::V0(AccountTypeV0Inner::Argent));
        transaction.signature = bounded_vec!(Felt252Wrapper::ZERO, Felt252Wrapper::ONE);

        let validate_result = Starknet::validate_unsigned(
            TransactionSource::InBlock,
            &crate::Call::declare { transaction: transaction.clone() },
        );
        assert!(matches!(validate_result.unwrap_err(), TransactionValidityError::Invalid(_)));

        assert_err!(Starknet::declare(none_origin, transaction), Error::<MockRuntime>::TransactionExecutionFailed);
    });
}

#[test]
fn given_contract_declare_on_cairo_1_no_validate_account_then_it_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();

        let account_addr = get_account_address(AccountType::V1(AccountTypeV1Inner::NoValidate));

        let hello_starknet_class = get_contract_class("HelloStarknet.casm.json", 1);
        let hello_starknet_class_hash =
            Felt252Wrapper::from_hex_be("0x010bd93d6a001480047a4474daf84aaa33be4c5419a6e0e8f0330348cb61faac").unwrap();
        let hello_starknet_compiled_class_hash =
            Felt252Wrapper::from_hex_be("0x00df4d3042eec107abe704619f13d92bbe01a58029311b7a1886b23dcbb4ea87").unwrap();

        let mut transaction = DeclareTransaction {
            sender_address: account_addr,
            contract_class: hello_starknet_class,
            version: 2,
            class_hash: hello_starknet_class_hash,
            compiled_class_hash: Some(hello_starknet_compiled_class_hash),
            nonce: Felt252Wrapper::ZERO,
            max_fee: Felt252Wrapper::from(u64::MAX),
            signature: bounded_vec!(),
            is_query: false,
        };

        let chain_id = Starknet::chain_id();
        let transaction_hash = calculate_declare_tx_hash(transaction.clone(), chain_id);
        transaction.signature = sign_message_hash(transaction_hash);

        let validate_result = Starknet::validate_unsigned(
            TransactionSource::InBlock,
            &crate::Call::declare { transaction: transaction.clone() },
        );
        assert_ok!(validate_result);

        assert_ok!(Starknet::declare(none_origin, transaction));
        assert_eq!(
            Starknet::contract_class_by_class_hash(hello_starknet_class_hash).unwrap(),
            get_contract_class("HelloStarknet.casm.json", 1)
        );
    });
}

#[test]
fn test_verify_tx_longevity() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let transaction = get_declare_dummy(AccountType::V0(AccountTypeV0Inner::NoValidate));

        let validate_result =
            Starknet::validate_unsigned(TransactionSource::InBlock, &crate::Call::declare { transaction });

        assert!(validate_result.unwrap().longevity == TransactionLongevity::get());
    });
}

#[test]
fn test_verify_no_require_tag() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let transaction = get_declare_dummy(AccountType::V0(AccountTypeV0Inner::NoValidate));

        let validate_result = Starknet::validate_unsigned(
            TransactionSource::InBlock,
            &crate::Call::declare { transaction: transaction.clone() },
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
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let mut transaction = get_declare_dummy(AccountType::V0(AccountTypeV0Inner::NoValidate));
        transaction.nonce = Felt252Wrapper::ONE;

        let validate_result = Starknet::validate_unsigned(
            TransactionSource::InBlock,
            &crate::Call::declare { transaction: transaction.clone() },
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
