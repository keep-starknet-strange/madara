use core::str::FromStr;

use frame_support::{assert_err, assert_ok, bounded_vec};
use hex::FromHex;
use mp_starknet::execution::types::ContractClassWrapper;
use mp_starknet::transaction::types::DeclareTransaction;
use sp_core::{H256, U256};
use sp_runtime::traits::ValidateUnsigned;
use sp_runtime::transaction_validity::{TransactionSource, TransactionValidityError};

use super::mock::*;
use super::utils::{get_contract_class, sign_message_hash};
use crate::tests::constants::TEST_ACCOUNT_SALT;
use crate::Error;

#[test]
fn given_contract_declare_tx_works_once_not_twice() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);
        let none_origin = RuntimeOrigin::none();
        let account_addr = get_account_address(AccountType::NoValidate);

        let erc20_class = ContractClassWrapper::try_from(get_contract_class("erc20/erc20.json")).unwrap();
        let erc20_class_hash =
            <[u8; 32]>::from_hex("057eca87f4b19852cfd4551cf4706ababc6251a8781733a0a11cf8e94211da95").unwrap();

        let transaction = DeclareTransaction {
            sender_address: account_addr,
            version: 1,
            compiled_class_hash: erc20_class_hash,
            contract_class: erc20_class,
            nonce: U256::zero(),
            max_fee: U256::from(u128::MAX),
            signature: bounded_vec!(),
        };

        assert_ok!(Starknet::declare(none_origin.clone(), transaction.clone()));
        // TODO: Uncomment once we have ABI support
        // assert_eq!(Starknet::contract_class_by_class_hash(erc20_class_hash), erc20_class);
        assert_err!(Starknet::declare(none_origin, transaction), Error::<MockRuntime>::ClassHashAlreadyDeclared);
    });
}

#[test]
fn given_contract_declare_tx_fails_sender_not_deployed() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();

        // Wrong address (not deployed)
        let contract_address_str = "03e437FB56Bb213f5708Fcd6966502070e276c093ec271aA33433b89E21fd31f";
        let contract_address_bytes = <[u8; 32]>::from_hex(contract_address_str).unwrap();

        let erc20_class = ContractClassWrapper::try_from(get_contract_class("erc20/erc20.json")).unwrap();
        let erc20_class_hash =
            <[u8; 32]>::from_hex("057eca87f4b19852cfd4551cf4706ababc6251a8781733a0a11cf8e94211da95").unwrap();

        let transaction = DeclareTransaction {
            sender_address: contract_address_bytes,
            contract_class: erc20_class,
            version: 1,
            compiled_class_hash: erc20_class_hash,
            nonce: U256::zero(),
            max_fee: U256::from(u128::MAX),
            signature: bounded_vec!(),
        };

        assert_err!(Starknet::declare(none_origin, transaction), Error::<MockRuntime>::AccountNotDeployed);
    })
}

#[test]
fn given_contract_declare_tx_fails_wrong_tx_version() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();
        let (account_addr, _, _) = account_helper(TEST_ACCOUNT_SALT, AccountType::ArgentV0);

        let erc20_class = ContractClassWrapper::try_from(get_contract_class("erc20/erc20.json")).unwrap();
        // TODO: Delete when the class hash can be derived from ContractClass
        let erc20_class_hash =
            <[u8; 32]>::from_hex("057eca87f4b19852cfd4551cf4706ababc6251a8781733a0a11cf8e94211da95").unwrap();

        let wrong_tx_version = 50_u8;

        let transaction = DeclareTransaction {
            sender_address: account_addr,
            contract_class: erc20_class,
            version: wrong_tx_version,
            compiled_class_hash: erc20_class_hash,
            nonce: U256::zero(),
            max_fee: U256::from(u128::MAX),
            signature: bounded_vec!(),
        };

        assert_err!(Starknet::declare(none_origin, transaction), Error::<MockRuntime>::TransactionExecutionFailed);
    })
}

#[test]
fn given_contract_declare_on_openzeppelin_account_then_it_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);
        let none_origin = RuntimeOrigin::none();

        let account_addr = get_account_address(AccountType::Openzeppelin);

        let erc20_class = ContractClassWrapper::try_from(get_contract_class("erc20/erc20.json")).unwrap();
        let erc20_class_hash =
            <[u8; 32]>::from_hex("057eca87f4b19852cfd4551cf4706ababc6251a8781733a0a11cf8e94211da95").unwrap();

        let tx_hash = H256::from_str("0x04b6608f43263d19966c6cc30f3619c29e8ced2e07a4947b8c0c2fd56d44d4fb").unwrap();
        let transaction = DeclareTransaction {
            sender_address: account_addr,
            contract_class: erc20_class,
            version: 1,
            compiled_class_hash: erc20_class_hash,
            nonce: U256::zero(),
            max_fee: U256::from(u128::MAX),
            signature: sign_message_hash(tx_hash),
        };

        assert_ok!(Starknet::declare(none_origin, transaction));
        assert_eq!(
            Starknet::contract_class_by_class_hash(erc20_class_hash).unwrap(),
            ContractClassWrapper::try_from(get_contract_class("erc20/erc20.json")).unwrap()
        );
    });
}

#[test]
fn given_contract_declare_on_openzeppelin_account_with_incorrect_signature_then_it_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);
        let none_origin = RuntimeOrigin::none();

        let account_addr = get_account_address(AccountType::Openzeppelin);

        let erc20_class = ContractClassWrapper::try_from(get_contract_class("erc20/erc20.json")).unwrap();
        let erc20_class_hash =
            <[u8; 32]>::from_hex("057eca87f4b19852cfd4551cf4706ababc6251a8781733a0a11cf8e94211da95").unwrap();

        let transaction = DeclareTransaction {
            sender_address: account_addr,
            contract_class: erc20_class,
            version: 1,
            compiled_class_hash: erc20_class_hash,
            nonce: U256::zero(),
            max_fee: U256::from(u128::MAX),
            signature: bounded_vec!(H256::from_low_u64_be(0), H256::from_low_u64_be(1)),
        };

        assert_err!(Starknet::declare(none_origin, transaction), Error::<MockRuntime>::TransactionExecutionFailed);
    });
}

#[test]
fn given_contract_declare_on_braavos_account_then_it_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);
        let none_origin = RuntimeOrigin::none();

        let account_addr = get_account_address(AccountType::Braavos);

        let erc20_class = ContractClassWrapper::try_from(get_contract_class("erc20/erc20.json")).unwrap();
        let erc20_class_hash =
            <[u8; 32]>::from_hex("057eca87f4b19852cfd4551cf4706ababc6251a8781733a0a11cf8e94211da95").unwrap();

        let tx_hash = H256::from_str("0x076975b47411feb8dab2633a7b8a2db3d8112a2492d1ccc2bdb832bbc5db0cff").unwrap();
        let transaction = DeclareTransaction {
            sender_address: account_addr,
            contract_class: erc20_class,
            version: 1,
            compiled_class_hash: erc20_class_hash,
            nonce: U256::zero(),
            max_fee: U256::from(u128::MAX),
            signature: sign_message_hash(tx_hash),
        };

        assert_ok!(Starknet::declare(none_origin, transaction));
        assert_eq!(
            Starknet::contract_class_by_class_hash(erc20_class_hash).unwrap(),
            ContractClassWrapper::try_from(get_contract_class("erc20/erc20.json")).unwrap()
        );
    });
}

#[test]
fn given_contract_declare_on_braavos_account_with_incorrect_signature_then_it_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);
        let none_origin = RuntimeOrigin::none();

        let account_addr = get_account_address(AccountType::Braavos);

        let erc20_class = ContractClassWrapper::try_from(get_contract_class("erc20/erc20.json")).unwrap();
        let erc20_class_hash =
            <[u8; 32]>::from_hex("057eca87f4b19852cfd4551cf4706ababc6251a8781733a0a11cf8e94211da95").unwrap();

        let transaction = DeclareTransaction {
            sender_address: account_addr,
            contract_class: erc20_class,
            version: 1,
            compiled_class_hash: erc20_class_hash,
            nonce: U256::zero(),
            max_fee: U256::from(u128::MAX),
            signature: bounded_vec!(H256::from_low_u64_be(0), H256::from_low_u64_be(1)),
        };

        assert_err!(Starknet::declare(none_origin, transaction), Error::<MockRuntime>::TransactionExecutionFailed);
    });
}

#[test]
fn given_contract_declare_on_argent_account_then_it_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);
        let none_origin = RuntimeOrigin::none();

        let account_addr = get_account_address(AccountType::Argent);

        let erc20_class = ContractClassWrapper::try_from(get_contract_class("erc20/erc20.json")).unwrap();
        let erc20_class_hash =
            <[u8; 32]>::from_hex("057eca87f4b19852cfd4551cf4706ababc6251a8781733a0a11cf8e94211da95").unwrap();

        let tx_hash = H256::from_str("0x02fc479a47d17efd76b69a1eb7731f1ac592948ab19b1047a087a43378d5a61a").unwrap();
        let transaction = DeclareTransaction {
            sender_address: account_addr,
            contract_class: erc20_class,
            version: 1,
            compiled_class_hash: erc20_class_hash,
            nonce: U256::zero(),
            max_fee: U256::from(u128::MAX),
            signature: sign_message_hash(tx_hash),
        };

        assert_ok!(Starknet::declare(none_origin, transaction));
        assert_eq!(
            Starknet::contract_class_by_class_hash(erc20_class_hash).unwrap(),
            ContractClassWrapper::try_from(get_contract_class("erc20/erc20.json")).unwrap()
        );
    });
}

#[test]
fn given_contract_declare_on_argent_account_with_incorrect_signature_then_it_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);
        let none_origin = RuntimeOrigin::none();

        let account_addr = get_account_address(AccountType::Argent);

        let erc20_class = ContractClassWrapper::try_from(get_contract_class("erc20/erc20.json")).unwrap();
        let erc20_class_hash =
            <[u8; 32]>::from_hex("057eca87f4b19852cfd4551cf4706ababc6251a8781733a0a11cf8e94211da95").unwrap();

        let transaction = DeclareTransaction {
            sender_address: account_addr,
            contract_class: erc20_class,
            version: 1,
            compiled_class_hash: erc20_class_hash,
            nonce: U256::zero(),
            max_fee: U256::from(u128::MAX),
            signature: bounded_vec!(H256::from_low_u64_be(0), H256::from_low_u64_be(1)),
        };

        assert_err!(Starknet::declare(none_origin, transaction), Error::<MockRuntime>::TransactionExecutionFailed);
    });
}

#[test]
fn given_contract_declare_on_braavos_account_validate_with_incorrect_signature_should_fail() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let account_addr = get_account_address(AccountType::Braavos);

        let erc20_class = ContractClassWrapper::try_from(get_contract_class(ERC20_CONTRACT_PATH)).unwrap();
        let erc20_class_hash =
            <[u8; 32]>::from_hex("057eca87f4b19852cfd4551cf4706ababc6251a8781733a0a11cf8e94211da95").unwrap();

        let transaction = DeclareTransaction {
            sender_address: account_addr,
            contract_class: erc20_class,
            version: 1,
            compiled_class_hash: erc20_class_hash,
            nonce: U256::zero(),
            max_fee: U256::from(u128::MAX),
            signature: bounded_vec!(H256::from_low_u64_be(0), H256::from_low_u64_be(1)),
        };

        let validate_result =
            Starknet::validate_unsigned(TransactionSource::InBlock, &crate::Call::declare { transaction });
        assert!(matches!(validate_result.unwrap_err(), TransactionValidityError::Invalid(_)));
    });
}
