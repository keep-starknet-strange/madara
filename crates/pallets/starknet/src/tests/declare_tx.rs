use frame_support::{assert_err, assert_ok, bounded_vec};
use hex::FromHex;
use mp_starknet::execution::types::ContractClassWrapper;
use mp_starknet::transaction::types::DeclareTransaction;
use sp_core::U256;

use super::mock::*;
use crate::Error;
pub const ERC20_CONTRACT_PATH: &[u8] = include_bytes!("../../../../../resources/erc20/erc20.json");
#[test]
fn given_contract_declare_tx_works_once_not_twice() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);
        let none_origin = RuntimeOrigin::none();
        let (account_addr, _, _) = no_validate_account_helper(TEST_ACCOUNT_SALT);

        let erc20_class = ContractClassWrapper::try_from(get_contract_class(ERC20_CONTRACT_PATH)).unwrap();
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

        let erc20_class = ContractClassWrapper::try_from(get_contract_class(ERC20_CONTRACT_PATH)).unwrap();
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
        let (account_addr, _, _) = account_helper(TEST_ACCOUNT_SALT);

        let erc20_class = ContractClassWrapper::try_from(get_contract_class(ERC20_CONTRACT_PATH)).unwrap();
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
