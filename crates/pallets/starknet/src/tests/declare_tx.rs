use assert_matches::assert_matches;
use frame_support::{assert_err, assert_ok};
use mp_felt::Felt252Wrapper;
use mp_transactions::compute_hash::ComputeTransactionHash;
use mp_transactions::{DeclareTransactionV1, DeclareTransactionV2};
use sp_runtime::traits::ValidateUnsigned;
use sp_runtime::transaction_validity::{
    InvalidTransaction, TransactionSource, TransactionValidityError, ValidTransaction,
};
use starknet_api::api_core::{ClassHash, Nonce};
use starknet_api::hash::StarkFelt;
use starknet_crypto::FieldElement;

use super::mock::default_mock::*;
use super::mock::*;
use super::utils::{get_contract_class, sign_message_hash};
use crate::tests::{get_declare_dummy, set_nonce};
use crate::{Config, Error};

#[test]
fn given_contract_declare_tx_works_once_not_twice() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();
        let account_addr = get_account_address(None, AccountType::V0(AccountTypeV0Inner::NoValidate));

        let erc20_class = get_contract_class("ERC20.json", 0);
        let erc20_class_hash =
            Felt252Wrapper::from_hex_be("0x057eca87f4b19852cfd4551cf4706ababc6251a8781733a0a11cf8e94211da95").unwrap();

        let transaction = DeclareTransactionV1 {
            sender_address: account_addr.into(),
            class_hash: erc20_class_hash,
            nonce: Felt252Wrapper::ZERO,
            max_fee: u128::MAX,
            signature: vec![],
            offset_version: false,
        };

        assert_ok!(Starknet::declare(none_origin.clone(), transaction.clone().into(), erc20_class.clone()));
        // TODO: Uncomment once we have ABI support
        // assert_eq!(Starknet::contract_class_by_class_hash(erc20_class_hash), erc20_class);
        assert_err!(
            Starknet::declare(none_origin, transaction.into(), erc20_class),
            Error::<MockRuntime>::ClassHashAlreadyDeclared
        );
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

        let transaction = DeclareTransactionV1 {
            sender_address: contract_address,
            class_hash: erc20_class_hash,
            nonce: Felt252Wrapper::ZERO,
            max_fee: u128::MAX,
            signature: vec![],
            offset_version: false,
        };

        assert_err!(
            Starknet::declare(none_origin, transaction.into(), erc20_class),
            Error::<MockRuntime>::AccountNotDeployed
        );
    })
}

#[test]
fn given_contract_declare_on_openzeppelin_account_then_it_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();

        let chain_id = Starknet::chain_id();
        let transaction =
            get_declare_dummy(chain_id, Felt252Wrapper::ZERO, AccountType::V0(AccountTypeV0Inner::Openzeppelin));
        let erc20_class = get_contract_class("ERC20.json", 0);
        let erc20_class_hash = *transaction.class_hash();

        assert_ok!(Starknet::validate_unsigned(
            TransactionSource::InBlock,
            &crate::Call::declare { transaction: transaction.clone(), contract_class: erc20_class.clone() },
        ));

        assert_ok!(Starknet::declare(none_origin, transaction, erc20_class.clone()));
        assert_eq!(Starknet::contract_class_by_class_hash(ClassHash::from(erc20_class_hash)).unwrap(), erc20_class);
    });
}

#[test]
fn given_contract_declare_on_openzeppelin_account_with_incorrect_signature_then_it_fails() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();

        let account_addr = get_account_address(None, AccountType::V0(AccountTypeV0Inner::Openzeppelin));

        let erc20_class = get_contract_class("ERC20.json", 0);
        let erc20_class_hash =
            Felt252Wrapper::from_hex_be("0x057eca87f4b19852cfd4551cf4706ababc6251a8781733a0a11cf8e94211da95").unwrap();

        let transaction = DeclareTransactionV1 {
            max_fee: u128::MAX,
            signature: vec![Felt252Wrapper::ZERO, Felt252Wrapper::ONE],
            nonce: Felt252Wrapper::ZERO,
            class_hash: erc20_class_hash,
            sender_address: account_addr.into(),
            offset_version: false,
        };

        assert_matches!(
            Starknet::validate_unsigned(
                TransactionSource::InBlock,
                &crate::Call::declare { transaction: transaction.clone().into(), contract_class: erc20_class.clone() },
            ),
            Err(TransactionValidityError::Invalid(_))
        );

        assert_err!(
            Starknet::declare(none_origin, transaction.into(), erc20_class),
            Error::<MockRuntime>::TransactionExecutionFailed
        );
    });
}

#[test]
fn given_contract_declare_on_braavos_account_then_it_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();

        let chain_id = Starknet::chain_id();
        let transaction =
            get_declare_dummy(chain_id, Felt252Wrapper::ZERO, AccountType::V0(AccountTypeV0Inner::Braavos));
        let erc20_class_hash = *transaction.class_hash();
        let erc20_class = get_contract_class("ERC20.json", 0);

        let validate_result = Starknet::validate_unsigned(
            TransactionSource::InBlock,
            &crate::Call::declare { transaction: transaction.clone(), contract_class: erc20_class.clone() },
        );
        assert_ok!(validate_result);

        assert_ok!(Starknet::declare(none_origin, transaction, erc20_class.clone()));
        assert_eq!(Starknet::contract_class_by_class_hash(ClassHash::from(erc20_class_hash)).unwrap(), erc20_class);
    });
}

#[test]
fn given_contract_declare_on_braavos_account_with_incorrect_signature_then_it_fails() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();

        let account_addr = get_account_address(None, AccountType::V0(AccountTypeV0Inner::Braavos));

        let erc20_class = get_contract_class("ERC20.json", 0);
        let erc20_class_hash =
            Felt252Wrapper::from_hex_be("0x057eca87f4b19852cfd4551cf4706ababc6251a8781733a0a11cf8e94211da95").unwrap();

        let transaction = DeclareTransactionV1 {
            max_fee: u128::MAX,
            signature: vec![Felt252Wrapper::ZERO, Felt252Wrapper::ONE],
            nonce: Felt252Wrapper::ZERO,
            class_hash: erc20_class_hash,
            sender_address: account_addr.into(),
            offset_version: false,
        };

        assert_matches!(
            Starknet::validate_unsigned(
                TransactionSource::InBlock,
                &crate::Call::declare { transaction: transaction.clone().into(), contract_class: erc20_class.clone() },
            ),
            Err(TransactionValidityError::Invalid(_))
        );

        assert_err!(
            Starknet::declare(none_origin, transaction.into(), erc20_class),
            Error::<MockRuntime>::TransactionExecutionFailed
        );
    });
}

#[test]
fn given_contract_declare_on_argent_account_then_it_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();

        let chain_id = Starknet::chain_id();
        let transaction =
            get_declare_dummy(chain_id, Felt252Wrapper::ZERO, AccountType::V0(AccountTypeV0Inner::Argent));
        let erc20_class_hash = *transaction.class_hash();
        let erc20_class = get_contract_class("ERC20.json", 0);

        let validate_result = Starknet::validate_unsigned(
            TransactionSource::InBlock,
            &crate::Call::declare { transaction: transaction.clone(), contract_class: erc20_class.clone() },
        );
        assert_ok!(validate_result);

        assert_ok!(Starknet::declare(none_origin, transaction, erc20_class.clone()));
        assert_eq!(Starknet::contract_class_by_class_hash(ClassHash::from(erc20_class_hash)).unwrap(), erc20_class);
    });
}

#[test]
fn given_contract_declare_on_argent_account_with_incorrect_signature_then_it_fails() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();

        let account_addr = get_account_address(None, AccountType::V0(AccountTypeV0Inner::Argent));

        let erc20_class = get_contract_class("ERC20.json", 0);
        let erc20_class_hash =
            Felt252Wrapper::from_hex_be("0x057eca87f4b19852cfd4551cf4706ababc6251a8781733a0a11cf8e94211da95").unwrap();

        let transaction = DeclareTransactionV1 {
            max_fee: u128::MAX,
            signature: vec![Felt252Wrapper::ZERO, Felt252Wrapper::ONE],
            nonce: Felt252Wrapper::ZERO,
            class_hash: erc20_class_hash,
            sender_address: account_addr.into(),
            offset_version: false,
        };

        assert_matches!(
            Starknet::validate_unsigned(
                TransactionSource::InBlock,
                &crate::Call::declare { transaction: transaction.clone().into(), contract_class: erc20_class.clone() },
            ),
            Err(TransactionValidityError::Invalid(_))
        );

        assert_err!(
            Starknet::declare(none_origin, transaction.into(), erc20_class),
            Error::<MockRuntime>::TransactionExecutionFailed
        );
    });
}

#[test]
fn given_contract_declare_on_cairo_1_no_validate_account_then_it_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();

        let account_addr = get_account_address(None, AccountType::V1(AccountTypeV1Inner::NoValidate));

        let hello_starknet_class = get_contract_class("HelloStarknet.casm.json", 1);
        let hello_starknet_class_hash =
            Felt252Wrapper::from_hex_be("0x05518b17fb5c84683ba37eba8a682b7a6f330911c2216c52c6badff69cc2ec13").unwrap();
        let hello_starknet_compiled_class_hash =
            Felt252Wrapper::from_hex_be("0x00df4d3042eec107abe704619f13d92bbe01a58029311b7a1886b23dcbb4ea87").unwrap();

        let mut transaction = DeclareTransactionV2 {
            sender_address: account_addr.into(),
            class_hash: hello_starknet_class_hash,
            compiled_class_hash: hello_starknet_compiled_class_hash,
            nonce: Felt252Wrapper::ZERO,
            max_fee: u128::MAX,
            signature: vec![],
            offset_version: false,
        };

        let chain_id = Starknet::chain_id();
        let transaction_hash = transaction.compute_hash::<<MockRuntime as Config>::SystemHash>(chain_id, false);
        transaction.signature = sign_message_hash(transaction_hash);

        assert_ok!(Starknet::validate_unsigned(
            TransactionSource::InBlock,
            &crate::Call::declare {
                transaction: transaction.clone().into(),
                contract_class: hello_starknet_class.clone()
            },
        ));

        assert_ok!(Starknet::declare(none_origin, transaction.into(), hello_starknet_class.clone()));
        assert_eq!(
            Starknet::contract_class_by_class_hash(ClassHash::from(hello_starknet_class_hash)).unwrap(),
            hello_starknet_class
        );
    });
}

#[test]
fn test_verify_tx_longevity() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let chain_id = Starknet::chain_id();
        let transaction =
            get_declare_dummy(chain_id, Felt252Wrapper::ZERO, AccountType::V0(AccountTypeV0Inner::NoValidate));
        let erc20_class = get_contract_class("ERC20.json", 0);

        let validate_result = Starknet::validate_unsigned(
            TransactionSource::InBlock,
            &crate::Call::declare { transaction, contract_class: erc20_class },
        )
        .unwrap();

        assert_eq!(validate_result.longevity, TransactionLongevity::get());
    });
}

#[test]
fn test_verify_require_tag() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let chain_id = Starknet::chain_id();
        let transaction =
            get_declare_dummy(chain_id, Felt252Wrapper::ONE, AccountType::V0(AccountTypeV0Inner::NoValidate));
        let erc20_class = get_contract_class("ERC20.json", 0);

        let validate_result = Starknet::validate_unsigned(
            TransactionSource::InBlock,
            &crate::Call::declare { transaction: transaction.clone(), contract_class: erc20_class },
        )
        .unwrap();

        let valid_transaction_expected = ValidTransaction::with_tag_prefix("starknet")
            .priority(u64::MAX)
            .and_provides((*transaction.sender_address(), *transaction.nonce()))
            .longevity(TransactionLongevity::get())
            .propagate(true)
            .and_requires((*transaction.sender_address(), Felt252Wrapper(transaction.nonce().0 - FieldElement::ONE)))
            .build()
            .unwrap();

        assert_eq!(validate_result, valid_transaction_expected)
    });
}

#[test]
fn test_verify_nonce_in_unsigned_tx() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let transaction = get_declare_dummy(
            Starknet::chain_id(),
            Felt252Wrapper::ONE,
            AccountType::V0(AccountTypeV0Inner::NoValidate),
        );
        let erc20_class = get_contract_class("ERC20.json", 0);

        let tx_sender = (*transaction.sender_address()).into();
        let tx_source = TransactionSource::InBlock;
        let call = crate::Call::declare { transaction, contract_class: erc20_class };

        assert!(Starknet::validate_unsigned(tx_source, &call).is_ok());

        set_nonce::<MockRuntime>(&tx_sender, &Nonce(StarkFelt::from(2u64)));

        assert_eq!(
            Starknet::validate_unsigned(tx_source, &call),
            Err(TransactionValidityError::Invalid(InvalidTransaction::Stale))
        );
    });
}
