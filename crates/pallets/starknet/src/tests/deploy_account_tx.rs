use std::str::FromStr;

use frame_support::{assert_err, assert_ok, bounded_vec, BoundedVec};
use mp_starknet::execution::types::Felt252Wrapper;
use mp_starknet::transaction::types::{DeployAccountTransaction, EventWrapper};
use sp_core::U256;

use super::mock::*;
use super::utils::sign_message_hash;
use crate::tests::constants::ACCOUNT_PUBLIC_KEY;
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
            // Calldata is hex so this works fine
            calldata: BoundedVec::try_from(
                calldata
                    .clone()
                    .into_iter()
                    .map(|e| Felt252Wrapper::from_hex_be(e).unwrap())
                    .collect::<Vec<Felt252Wrapper>>(),
            )
            .unwrap(),
            nonce: Felt252Wrapper::ZERO,
            max_fee: Felt252Wrapper::from(u128::MAX),
            signature: bounded_vec!(),
        };

        assert_ok!(Starknet::deploy_account(none_origin, transaction));
        assert_eq!(Starknet::contract_class_hash_by_address(test_addr).unwrap(), account_class_hash);
        let expected_fee_transfer_event = Event::StarknetEvent(EventWrapper {
            keys: bounded_vec![
                Felt252Wrapper::from_hex_be("0x0099cd8bde557814842a3121e8ddfd433a539b8c9f14bf31ebf108d12e6196e9")
                    .unwrap()
            ],
            data: bounded_vec!(
                test_addr,                                      // From
                Felt252Wrapper::from_hex_be("0x2").unwrap(),    // To
                Felt252Wrapper::from_hex_be("0xd3b8").unwrap(), // Amount low
                Felt252Wrapper::ZERO,                           // Amount high
            ),
            from_address: Starknet::fee_token_address(),
        })
        .into();
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
            calldata: BoundedVec::try_from(
                calldata
                    .clone()
                    .into_iter()
                    .map(|e| Felt252Wrapper::from_hex_be(e).unwrap())
                    .collect::<Vec<Felt252Wrapper>>(),
            )
            .unwrap(),
            salt: U256::from_str(salt).unwrap(),
            version: 1,
            nonce: Felt252Wrapper::ZERO,
            max_fee: Felt252Wrapper::from(u128::MAX),
            signature: bounded_vec!(),
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
        let rand_address = Felt252Wrapper::from_hex_be("0x1234").unwrap();
        let (_, account_class_hash, _) = account_helper(salt, AccountType::ArgentV0);
        let transaction = DeployAccountTransaction {
            account_class_hash,
            sender_address: rand_address,
            version: 1,
            calldata: bounded_vec!(),
            salt: U256::zero(),
            nonce: Felt252Wrapper::ZERO,
            max_fee: Felt252Wrapper::from(u128::MAX),
            signature: bounded_vec!(),
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
            calldata: BoundedVec::try_from(
                calldata
                    .clone()
                    .into_iter()
                    .map(|e| Felt252Wrapper::from_hex_be(e).unwrap())
                    .collect::<Vec<Felt252Wrapper>>(),
            )
            .unwrap(),
            salt: U256::zero(),
            nonce: Felt252Wrapper::ZERO,
            max_fee: Felt252Wrapper::from(u128::MAX),
            signature: bounded_vec!(),
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
        let tx_hash =
            Felt252Wrapper::from_hex_be("0x06ff0e0245daed20c0b4f21ae5c9286ba3a03e0c62b2bec2d0dcec2a4d6b9889").unwrap();

        let transaction = DeployAccountTransaction {
            account_class_hash,
            sender_address: test_addr,
            salt: U256::from_str(salt).unwrap(),
            version: 1,
            calldata: BoundedVec::try_from(
                calldata
                    .clone()
                    .into_iter()
                    .map(|e| Felt252Wrapper::from_hex_be(e).unwrap())
                    .collect::<Vec<Felt252Wrapper>>(),
            )
            .unwrap(),
            nonce: Felt252Wrapper::ZERO,
            max_fee: Felt252Wrapper::from(u128::MAX),
            signature: sign_message_hash(tx_hash),
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
            calldata: BoundedVec::try_from(
                calldata
                    .clone()
                    .into_iter()
                    .map(|e| Felt252Wrapper::from_hex_be(e).unwrap())
                    .collect::<Vec<Felt252Wrapper>>(),
            )
            .unwrap(),
            nonce: Felt252Wrapper::ZERO,
            max_fee: Felt252Wrapper::from(u128::MAX),
            signature: bounded_vec!(Felt252Wrapper::ONE, Felt252Wrapper::ONE),
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
        let tx_hash =
            Felt252Wrapper::from_hex_be("0x0781152a4f3fc0dada10f24a40f7499ce3c17c3867acae82024f5507475f89da").unwrap();

        let transaction = DeployAccountTransaction {
            account_class_hash,
            sender_address: test_addr,
            salt: U256::from_str(salt).unwrap(),
            version: 1,
            calldata: BoundedVec::try_from(
                calldata
                    .clone()
                    .into_iter()
                    .map(|e| Felt252Wrapper::from_hex_be(e).unwrap())
                    .collect::<Vec<Felt252Wrapper>>(),
            )
            .unwrap(),
            nonce: Felt252Wrapper::ZERO,
            max_fee: Felt252Wrapper::from(u128::MAX),
            signature: sign_message_hash(tx_hash),
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
            calldata: BoundedVec::try_from(
                calldata
                    .clone()
                    .into_iter()
                    .map(|e| Felt252Wrapper::from_hex_be(e).unwrap())
                    .collect::<Vec<Felt252Wrapper>>(),
            )
            .unwrap(),
            nonce: Felt252Wrapper::ZERO,
            max_fee: Felt252Wrapper::from(u128::MAX),
            signature: bounded_vec!(Felt252Wrapper::ONE, Felt252Wrapper::ONE),
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

        let tx_hash =
            Felt252Wrapper::from_hex_be("0x06ae3d81978d498def89e1121b2d84a873d63c30d80f7ed81e2dc9be6a961770").unwrap();

        let mut signatures: Vec<Felt252Wrapper> = sign_message_hash(tx_hash).into();
        let empty_signatures = [Felt252Wrapper::ZERO; 8];
        signatures.append(&mut empty_signatures.to_vec());

        let transaction = DeployAccountTransaction {
            account_class_hash: proxy_class_hash,
            sender_address: test_addr,
            salt: U256::from_str(salt).unwrap(),
            version: 1,
            calldata: BoundedVec::try_from(
                calldata
                    .clone()
                    .into_iter()
                    .map(|e| Felt252Wrapper::from_hex_be(e).unwrap())
                    .collect::<Vec<Felt252Wrapper>>(),
            )
            .unwrap(),
            nonce: Felt252Wrapper::ZERO,
            max_fee: Felt252Wrapper::from(u128::MAX),
            signature: signatures.try_into().unwrap(),
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
            calldata: BoundedVec::try_from(
                calldata
                    .clone()
                    .into_iter()
                    .map(|e| Felt252Wrapper::from_hex_be(e).unwrap())
                    .collect::<Vec<Felt252Wrapper>>(),
            )
            .unwrap(),
            nonce: Felt252Wrapper::ZERO,
            max_fee: Felt252Wrapper::from(u128::MAX),
            signature: [Felt252Wrapper::ZERO; 10].to_vec().try_into().unwrap(),
        };

        assert_err!(
            Starknet::deploy_account(none_origin, transaction),
            Error::<MockRuntime>::TransactionExecutionFailed
        );
    });
}

fn set_infinite_tokens(address: Felt252Wrapper) {
    StorageView::<MockRuntime>::insert(
        get_storage_key(&Starknet::fee_token_address(), "ERC20_balances", &[address], 0),
        Felt252Wrapper::from(u128::MAX),
    );
    StorageView::<MockRuntime>::insert(
        get_storage_key(&Starknet::fee_token_address(), "ERC20_balances", &[address], 1),
        Felt252Wrapper::from(u128::MAX),
    );
}

fn set_signer(address: Felt252Wrapper, account_type: AccountType) {
    let (var_name, args) = match account_type {
        AccountType::Argent => ("_signer", vec![]),
        AccountType::Braavos => ("Account_signers", vec![Felt252Wrapper::ZERO]),
        AccountType::Openzeppelin => ("Account_public_key", vec![]),
        _ => return,
    };
    StorageView::<MockRuntime>::insert(
        get_storage_key(&address, var_name, &args, 0),
        Felt252Wrapper::from_hex_be(ACCOUNT_PUBLIC_KEY).unwrap(),
    );
}
