use core::str::FromStr;

use frame_support::{assert_err, assert_ok, bounded_vec, BoundedVec};
use hex::FromHex;
use mp_starknet::transaction::types::{DeployAccountTransaction, EventWrapper};
use sp_core::{H256, U256};
use sp_runtime::traits::ValidateUnsigned;
use sp_runtime::transaction_validity::{TransactionSource, TransactionValidityError};

use super::mock::*;
use crate::{Error, Event, StorageView};

#[test]
fn given_contract_run_deploy_account_tx_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);
        let none_origin = RuntimeOrigin::none();
        // TEST ACCOUNT CONTRACT
        // - ref testnet tx(0x0751b4b5b95652ad71b1721845882c3852af17e2ed0c8d93554b5b292abb9810)
        let salt = "0x03b37cbe4e9eac89d54c5f7cc6329a63a63e8c8db2bf936f981041e086752463";
        let (test_addr, account_class_hash, calldata) = account_helper(salt, AccountType::NoValidate);

        set_infinite_tokens(test_addr);

        let transaction = DeployAccountTransaction {
            account_class_hash,
            sender_address: test_addr,
            salt: U256::from_str(salt).unwrap(),
            version: 1,
            calldata: BoundedVec::try_from(calldata.clone().into_iter().map(U256::from).collect::<Vec<U256>>())
                .unwrap(),
            nonce: U256::zero(),
            signature: bounded_vec!(),
            max_fee: U256::from(u128::MAX),
        };

        assert_ok!(Starknet::deploy_account(none_origin, transaction));
        assert_eq!(Starknet::contract_class_hash_by_address(test_addr).unwrap(), account_class_hash);
        let expected_fee_transfer_event = Event::StarknetEvent(EventWrapper {
                keys: bounded_vec![
                    H256::from_str("0x0099cd8bde557814842a3121e8ddfd433a539b8c9f14bf31ebf108d12e6196e9").unwrap()
                ],
                data: bounded_vec!(
                    H256::from_slice(&test_addr), // From
                    H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000002").unwrap(), // To
                    H256::from_str("0x000000000000000000000000000000000000000000000000000000000000d3b8").unwrap(), // Amount low
                    H256::zero(), // Amount high
                ),
                from_address: Starknet::fee_token_address(),
            }).into();
        System::assert_last_event(expected_fee_transfer_event)
    });
}

#[test]
fn given_contract_run_deploy_account_tx_twice_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);
        let none_origin = RuntimeOrigin::none();
        let salt = "0x03b37cbe4e9eac89d54c5f7cc6329a63a63e8c8db2bf936f981041e086752463";
        let (test_addr, account_class_hash, calldata) = account_helper(salt, AccountType::NoValidate);

        set_infinite_tokens(test_addr);

        // TEST ACCOUNT CONTRACT
        // - ref testnet tx(0x0751b4b5b95652ad71b1721845882c3852af17e2ed0c8d93554b5b292abb9810)
        let transaction = DeployAccountTransaction {
            account_class_hash,
            sender_address: test_addr,
            calldata: BoundedVec::try_from(calldata.clone().into_iter().map(U256::from).collect::<Vec<U256>>())
                .unwrap(),

            salt: U256::from_str(salt).unwrap(),
            version: 1,
            nonce: U256::zero(),
            signature: bounded_vec!(),
            max_fee: U256::from(u128::MAX),
        };

        assert_ok!(Starknet::deploy_account(none_origin.clone(), transaction.clone()));
        // Check that the account was created
        assert_eq!(Starknet::contract_class_hash_by_address(test_addr).unwrap(), account_class_hash);
        assert_err!(Starknet::deploy_account(none_origin, transaction), Error::<MockRuntime>::AccountAlreadyDeployed);
    });
}

#[test]
fn given_contract_run_deploy_account_tx_undeclared_then_it_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);
        let salt = "0x03b37cbe4e9eac89d54c5f7cc6329a63a63e8c8db2bf936f981041e086752463";
        let none_origin = RuntimeOrigin::none();
        let rand_address =
            <[u8; 32]>::from_hex("0000000000000000000000000000000000000000000000000000000000001234").unwrap();
        let (_, account_class_hash, _) = account_helper(salt, AccountType::ArgentV0);
        let transaction = DeployAccountTransaction {
            account_class_hash,
            sender_address: rand_address,
            version: 1,
            calldata: bounded_vec!(),
            nonce: U256::zero(),
            salt: U256::zero(),
            signature: bounded_vec!(),
            max_fee: U256::from(u128::MAX),
        };

        assert_err!(
            Starknet::deploy_account(none_origin, transaction),
            Error::<MockRuntime>::TransactionExecutionFailed
        );
    });
}

#[test]
fn given_contract_run_deploy_account_tx_fails_wrong_tx_version() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();
        // TEST ACCOUNT CONTRACT
        // - ref testnet tx(0x0751b4b5b95652ad71b1721845882c3852af17e2ed0c8d93554b5b292abb9810)
        let salt = "0x03b37cbe4e9eac89d54c5f7cc6329a63a63e8c8db2bf936f981041e086752463";
        let (test_addr, account_class_hash, calldata) = account_helper(salt, AccountType::ArgentV0);

        let wrong_tx_version = 50_u8;

        let transaction = DeployAccountTransaction {
            account_class_hash,
            sender_address: test_addr,
            version: wrong_tx_version,
            calldata: BoundedVec::try_from(calldata.clone().into_iter().map(U256::from).collect::<Vec<U256>>())
                .unwrap(),
            nonce: U256::zero(),
            salt: U256::zero(),
            signature: bounded_vec!(),
            max_fee: U256::from(u128::MAX),
        };

        assert_err!(
            Starknet::deploy_account(none_origin, transaction),
            Error::<MockRuntime>::TransactionExecutionFailed
        );
    });
}

#[test]
fn given_contract_run_deploy_account_openzeppelin_tx_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();
        // TEST ACCOUNT CONTRACT
        // - ref testnet tx(0x0751b4b5b95652ad71b1721845882c3852af17e2ed0c8d93554b5b292abb9810)
        let salt = "0x03b37cbe4e9eac89d54c5f7cc6329a63a63e8c8db2bf936f981041e086752463";
        let (test_addr, account_class_hash, calldata) = account_helper(salt, AccountType::Openzeppelin);

        set_infinite_tokens(test_addr);
        set_signer(test_addr, AccountType::Openzeppelin);
        let tx_hash = H256::from_str("0x06ff0e0245daed20c0b4f21ae5c9286ba3a03e0c62b2bec2d0dcec2a4d6b9889").unwrap();

        let transaction = DeployAccountTransaction {
            account_class_hash,
            sender_address: test_addr,
            salt: U256::from_str(salt).unwrap(),
            version: 1,
            calldata: BoundedVec::try_from(calldata.clone().into_iter().map(U256::from).collect::<Vec<U256>>())
                .unwrap(),
            nonce: U256::zero(),
            signature: sign_message_hash(tx_hash),
            max_fee: U256::from(u128::MAX),
        };

        assert_ok!(Starknet::deploy_account(none_origin, transaction));
        assert_eq!(Starknet::contract_class_hash_by_address(test_addr).unwrap(), account_class_hash);
    });
}

#[test]
fn given_contract_run_deploy_account_openzeppelin_with_incorrect_signature_then_it_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();
        // TEST ACCOUNT CONTRACT
        // - ref testnet tx(0x0751b4b5b95652ad71b1721845882c3852af17e2ed0c8d93554b5b292abb9810)
        let salt = "0x03b37cbe4e9eac89d54c5f7cc6329a63a63e8c8db2bf936f981041e086752463";
        let (test_addr, account_class_hash, calldata) = account_helper(salt, AccountType::Openzeppelin);

        set_infinite_tokens(test_addr);
        set_signer(test_addr, AccountType::Openzeppelin);

        let transaction = DeployAccountTransaction {
            account_class_hash,
            sender_address: test_addr,
            salt: U256::from_str(salt).unwrap(),
            version: 1,
            calldata: BoundedVec::try_from(calldata.clone().into_iter().map(U256::from).collect::<Vec<U256>>())
                .unwrap(),
            nonce: U256::zero(),
            signature: bounded_vec!(H256::from_low_u64_be(1), H256::from_low_u64_be(1)),
            max_fee: U256::from(u128::MAX),
        };

        assert_err!(
            Starknet::deploy_account(none_origin, transaction),
            Error::<MockRuntime>::TransactionExecutionFailed
        );
    });
}

#[test]
fn given_contract_run_deploy_account_argent_tx_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();
        // TEST ACCOUNT CONTRACT
        // - ref testnet tx(0x0751b4b5b95652ad71b1721845882c3852af17e2ed0c8d93554b5b292abb9810)
        let salt = "0x03b37cbe4e9eac89d54c5f7cc6329a63a63e8c8db2bf936f981041e086752463";
        let (test_addr, account_class_hash, calldata) = account_helper(salt, AccountType::Argent);

        set_infinite_tokens(test_addr);
        set_signer(test_addr, AccountType::Argent);
        let tx_hash = H256::from_str("0x0781152a4f3fc0dada10f24a40f7499ce3c17c3867acae82024f5507475f89da").unwrap();

        let transaction = DeployAccountTransaction {
            account_class_hash,
            sender_address: test_addr,
            salt: U256::from_str(salt).unwrap(),
            version: 1,
            calldata: BoundedVec::try_from(calldata.clone().into_iter().map(U256::from).collect::<Vec<U256>>())
                .unwrap(),
            nonce: U256::zero(),
            signature: sign_message_hash(tx_hash),
            max_fee: U256::from(u128::MAX),
        };

        assert_ok!(Starknet::deploy_account(none_origin, transaction));
        assert_eq!(Starknet::contract_class_hash_by_address(test_addr).unwrap(), account_class_hash);
    });
}

#[test]
fn given_contract_run_deploy_account_argent_with_incorrect_signature_then_it_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();
        // TEST ACCOUNT CONTRACT
        // - ref testnet tx(0x0751b4b5b95652ad71b1721845882c3852af17e2ed0c8d93554b5b292abb9810)
        let salt = "0x03b37cbe4e9eac89d54c5f7cc6329a63a63e8c8db2bf936f981041e086752463";
        let (test_addr, account_class_hash, calldata) = account_helper(salt, AccountType::Argent);

        set_infinite_tokens(test_addr);
        set_signer(test_addr, AccountType::Argent);

        let transaction = DeployAccountTransaction {
            account_class_hash,
            sender_address: test_addr,
            salt: U256::from_str(salt).unwrap(),
            version: 1,
            calldata: BoundedVec::try_from(calldata.clone().into_iter().map(U256::from).collect::<Vec<U256>>())
                .unwrap(),
            nonce: U256::zero(),
            signature: bounded_vec!(H256::from_low_u64_be(1), H256::from_low_u64_be(1)),
            max_fee: U256::from(u128::MAX),
        };

        assert_err!(
            Starknet::deploy_account(none_origin, transaction),
            Error::<MockRuntime>::TransactionExecutionFailed
        );
    });
}

#[test]
fn given_contract_run_deploy_account_braavos_tx_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();
        // TEST ACCOUNT CONTRACT
        // - ref testnet tx(0x0751b4b5b95652ad71b1721845882c3852af17e2ed0c8d93554b5b292abb9810)
        let salt = "0x03b37cbe4e9eac89d54c5f7cc6329a63a63e8c8db2bf936f981041e086752463";
        let (test_addr, proxy_class_hash, mut calldata) = account_helper(salt, AccountType::BraavosProxy);
        calldata.push("0x1");
        calldata.push(ACCOUNT_PUBLIC_KEY);

        set_infinite_tokens(test_addr);
        set_signer(test_addr, AccountType::Braavos);

        let tx_hash = H256::from_str("0x06ae3d81978d498def89e1121b2d84a873d63c30d80f7ed81e2dc9be6a961770").unwrap();

        let mut signatures: Vec<H256> = sign_message_hash(tx_hash).into();
        let empty_signatures = [H256::from_low_u64_be(0); 8];
        signatures.append(&mut empty_signatures.to_vec());

        let transaction = DeployAccountTransaction {
            account_class_hash: proxy_class_hash,
            sender_address: test_addr,
            salt: U256::from_str(salt).unwrap(),
            version: 1,
            calldata: BoundedVec::try_from(calldata.clone().into_iter().map(U256::from).collect::<Vec<U256>>())
                .unwrap(),
            nonce: U256::zero(),
            signature: signatures.try_into().unwrap(),
            max_fee: U256::from(u128::MAX),
        };

        assert_ok!(Starknet::deploy_account(none_origin, transaction));
        assert_eq!(Starknet::contract_class_hash_by_address(test_addr).unwrap(), proxy_class_hash);
    });
}

#[test]
fn given_contract_run_deploy_account_braavos_with_incorrect_signature_then_it_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        let none_origin = RuntimeOrigin::none();
        // TEST ACCOUNT CONTRACT
        // - ref testnet tx(0x0751b4b5b95652ad71b1721845882c3852af17e2ed0c8d93554b5b292abb9810)
        let salt = "0x03b37cbe4e9eac89d54c5f7cc6329a63a63e8c8db2bf936f981041e086752463";
        let (test_addr, proxy_class_hash, mut calldata) = account_helper(salt, AccountType::BraavosProxy);
        calldata.push("0x1");
        calldata.push(ACCOUNT_PUBLIC_KEY);

        set_infinite_tokens(test_addr);
        set_signer(test_addr, AccountType::Braavos);

        let transaction = DeployAccountTransaction {
            account_class_hash: proxy_class_hash,
            sender_address: test_addr,
            salt: U256::from_str(salt).unwrap(),
            version: 1,
            calldata: BoundedVec::try_from(calldata.clone().into_iter().map(U256::from).collect::<Vec<U256>>())
                .unwrap(),
            nonce: U256::zero(),
            signature: [H256::from_low_u64_be(0); 10].to_vec().try_into().unwrap(),
            max_fee: U256::from(u128::MAX),
        };

        assert_err!(
            Starknet::deploy_account(none_origin, transaction),
            Error::<MockRuntime>::TransactionExecutionFailed
        );
    });
}

#[test]
fn given_contract_validate_deploy_account_openzeppelin_with_incorrect_signature_should_fail() {
    new_test_ext().execute_with(|| {
        System::set_block_number(0);
        run_to_block(2);

        // TEST ACCOUNT CONTRACT
        // - ref testnet tx(0x0751b4b5b95652ad71b1721845882c3852af17e2ed0c8d93554b5b292abb9810)
        let salt = "0x03b37cbe4e9eac89d54c5f7cc6329a63a63e8c8db2bf936f981041e086752463";
        let (test_addr, account_class_hash, calldata) = account_helper(salt, AccountType::Openzeppelin);

        set_infinite_tokens(test_addr);
        set_signer(test_addr, AccountType::Openzeppelin);

        let transaction = DeployAccountTransaction {
            account_class_hash,
            sender_address: test_addr,
            salt: U256::from_str(salt).unwrap(),
            version: 1,
            calldata: BoundedVec::try_from(calldata.clone().into_iter().map(U256::from).collect::<Vec<U256>>())
                .unwrap(),
            nonce: U256::zero(),
            signature: bounded_vec!(H256::from_low_u64_be(1), H256::from_low_u64_be(1)),
            max_fee: U256::from(u128::MAX),
        };

        let validate_result =
            Starknet::validate_unsigned(TransactionSource::InBlock, &crate::Call::deploy_account { transaction });
        assert!(std::matches!(validate_result.unwrap_err(), TransactionValidityError::Invalid(_)));
    });
}

fn set_infinite_tokens(address: [u8; 32]) {
    StorageView::<MockRuntime>::insert(
        get_storage_key(&Starknet::fee_token_address(), "ERC20_balances", &[address], 0),
        U256::from(u128::MAX),
    );
    StorageView::<MockRuntime>::insert(
        get_storage_key(&Starknet::fee_token_address(), "ERC20_balances", &[address], 1),
        U256::from(u128::MAX),
    );
}

fn set_signer(address: [u8; 32], account_type: AccountType) {
    let (var_name, args) = match account_type {
        AccountType::Argent => ("_signer", vec![]),
        AccountType::Braavos => ("Account_signers", vec![[0; 32]]),
        AccountType::Openzeppelin => ("Account_public_key", vec![]),
        _ => return,
    };
    StorageView::<MockRuntime>::insert(
        get_storage_key(&address, var_name, &args, 0),
        U256::from_str(ACCOUNT_PUBLIC_KEY).unwrap(),
    );
}
