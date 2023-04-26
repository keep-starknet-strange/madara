use blockifier::test_utils::{get_contract_class, ERC20_CONTRACT_PATH};
use frame_support::{assert_err, assert_ok, bounded_vec};
use hex::FromHex;
use mp_starknet::execution::{CallEntryPointWrapper, ContractClassWrapper, EntryPointTypeWrapper};
use mp_starknet::transaction::types::Transaction;

use super::mock::*;
use crate::Error;

#[test]
fn given_contract_declare_tx_works_once_not_twice() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();
        let (account_addr, _, _) = account_helper(TEST_ACCOUNT_SALT);

        let erc20_class = ContractClassWrapper::from(get_contract_class(ERC20_CONTRACT_PATH));
        let erc20_class_hash =
            <[u8; 32]>::from_hex("057eca87f4b19852cfd4551cf4706ababc6251a8781733a0a11cf8e94211da95").unwrap();

        let mut transaction = Transaction {
            sender_address: account_addr,
            call_entrypoint: CallEntryPointWrapper::new(
                Some(erc20_class_hash),
                EntryPointTypeWrapper::External,
                None,
                bounded_vec![],
                account_addr,
                account_addr,
            ),
            ..Transaction::default()
        };
        // Cannot declare a class with None
        assert_err!(
            Starknet::declare(none_origin.clone(), transaction.clone()),
            Error::<Test>::ContractClassMustBeSpecified
        );

        transaction.contract_class = Some(erc20_class);

        assert_ok!(Starknet::declare(none_origin.clone(), transaction.clone()));
        // TODO: Uncomment once we have ABI support
        // assert_eq!(Starknet::contract_class_by_class_hash(erc20_class_hash), erc20_class);
        assert_err!(Starknet::declare(none_origin, transaction), Error::<Test>::ClassHashAlreadyDeclared);
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

        let erc20_class = ContractClassWrapper::from(get_contract_class(ERC20_CONTRACT_PATH));
        let erc20_class_hash =
            <[u8; 32]>::from_hex("057eca87f4b19852cfd4551cf4706ababc6251a8781733a0a11cf8e94211da95").unwrap();

        let transaction = Transaction {
            sender_address: contract_address_bytes,
            contract_class: Some(erc20_class),
            call_entrypoint: CallEntryPointWrapper::new(
                Some(erc20_class_hash),
                EntryPointTypeWrapper::External,
                None,
                bounded_vec![],
                contract_address_bytes,
                contract_address_bytes,
            ),
            ..Transaction::default()
        };

        assert_err!(Starknet::declare(none_origin, transaction), Error::<Test>::AccountNotDeployed);
    })
}

#[test]
fn given_contract_declare_tx_fails_wrong_tx_version() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();
        let (account_addr, _, _) = account_helper(TEST_ACCOUNT_SALT);

        let erc20_class = ContractClassWrapper::from(get_contract_class(ERC20_CONTRACT_PATH));
        // TODO: Delete when the class hash can be derived from ContractClass
        let erc20_class_hash =
            <[u8; 32]>::from_hex("057eca87f4b19852cfd4551cf4706ababc6251a8781733a0a11cf8e94211da95").unwrap();

        let wrong_tx_version = 50_u8;

        let transaction = Transaction {
            sender_address: account_addr,
            contract_class: Some(erc20_class),
            version: wrong_tx_version,
            call_entrypoint: CallEntryPointWrapper::new(
                // TODO: change to `None` when the class hash can be derived from ContractClass
                Some(erc20_class_hash),
                EntryPointTypeWrapper::External,
                None,
                bounded_vec![],
                account_addr,
                account_addr,
            ),
            ..Transaction::default()
        };

        assert_err!(Starknet::declare(none_origin, transaction), Error::<Test>::TransactionExecutionFailed);
    })
}

#[test]
fn given_contract_declare_tx_fails_if_invalid_class() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();
        let (account_addr, _, _) = account_helper(TEST_ACCOUNT_SALT);

        let mut erc20_class = ContractClassWrapper::from(get_contract_class(ERC20_CONTRACT_PATH));
        // Transform erc20_class into an invalid class
        erc20_class.program = bounded_vec![];

        let erc20_class_hash =
            <[u8; 32]>::from_hex("057eca87f4b19852cfd4551cf4706ababc6251a8781733a0a11cf8e94211da95").unwrap();

        let transaction = Transaction {
            sender_address: account_addr,
            contract_class: Some(erc20_class),
            call_entrypoint: CallEntryPointWrapper::new(
                Some(erc20_class_hash),
                EntryPointTypeWrapper::External,
                None,
                bounded_vec![],
                account_addr,
                account_addr,
            ),
            ..Transaction::default()
        };

        assert_err!(Starknet::declare(none_origin.clone(), transaction.clone()), Error::<Test>::InvalidContractClass);
    });
}
