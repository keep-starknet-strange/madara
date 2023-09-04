use frame_support::{assert_err, assert_ok, bounded_vec, BoundedVec};
use mp_starknet::constants::SN_GOERLI_CHAIN_ID;
use mp_starknet::execution::types::Felt252Wrapper;
use mp_starknet::transaction::types::{DeployAccountTransaction, EventWrapper};
use sp_runtime::traits::ValidateUnsigned;
use sp_runtime::transaction_validity::TransactionSource;

use super::mock::default_mock::*;
use super::mock::*;
use super::utils::sign_message_hash;
use crate::tests::constants::{ACCOUNT_PUBLIC_KEY, SALT};
use crate::tests::{get_deploy_account_dummy, set_infinite_tokens};
use crate::{Error, Event, StorageView};

#[test]
fn given_contract_run_deploy_account_tx_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();
        // TEST ACCOUNT CONTRACT
        // - ref testnet tx(0x0751b4b5b95652ad71b1721845882c3852af17e2ed0c8d93554b5b292abb9810)
        let salt =
            Felt252Wrapper::from_hex_be("0x03b37cbe4e9eac89d54c5f7cc6329a63a63e8c8db2bf936f981041e086752463").unwrap();
        let (test_addr, account_class_hash, calldata) =
            account_helper(salt, AccountType::V0(AccountTypeV0Inner::NoValidate));

        set_infinite_tokens::<MockRuntime>(test_addr);

        let transaction = DeployAccountTransaction {
            account_class_hash,
            salt,
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
            max_fee: Felt252Wrapper::from(u64::MAX),
            signature: bounded_vec!(),
            is_query: false,
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
                Felt252Wrapper::from_hex_be("0xdead").unwrap(), // To
                Felt252Wrapper::from_hex_be("0xd552").unwrap(), // Amount low
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
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let transaction = get_deploy_account_dummy(*SALT, AccountType::V0(AccountTypeV0Inner::NoValidate));
        let account_class_hash = transaction.account_class_hash;

        let (address, _, _) = account_helper(*SALT, AccountType::V0(AccountTypeV0Inner::NoValidate));

        set_infinite_tokens::<MockRuntime>(address);

        assert_ok!(Starknet::deploy_account(RuntimeOrigin::none(), transaction.clone()));
        assert_eq!(Starknet::contract_class_hash_by_address(address).unwrap(), account_class_hash);
        assert_err!(
            Starknet::deploy_account(RuntimeOrigin::none(), transaction),
            Error::<MockRuntime>::AccountAlreadyDeployed
        );
    });
}

#[test]
fn given_contract_run_deploy_account_tx_undeclared_then_it_fails() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();

        let account_class_hash = get_account_class_hash(AccountType::V0(AccountTypeV0Inner::Argent));
        let transaction = DeployAccountTransaction {
            account_class_hash,
            version: 1,
            calldata: bounded_vec!(),
            salt: Felt252Wrapper::ZERO,
            nonce: Felt252Wrapper::ZERO,
            max_fee: Felt252Wrapper::from(u64::MAX),
            signature: bounded_vec!(),
            is_query: false,
        };

        assert_err!(
            Starknet::deploy_account(none_origin, transaction),
            Error::<MockRuntime>::TransactionExecutionFailed
        );
    });
}

#[test]
fn given_contract_run_deploy_account_tx_fails_wrong_tx_version() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();

        let transaction = get_deploy_account_dummy(*SALT, AccountType::V0(AccountTypeV0Inner::Argent));

        assert_err!(
            Starknet::deploy_account(none_origin, transaction),
            Error::<MockRuntime>::TransactionExecutionFailed
        );
    });
}

#[test]
fn given_contract_run_deploy_account_openzeppelin_tx_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();

        let mut transaction = get_deploy_account_dummy(*SALT, AccountType::V0(AccountTypeV0Inner::Openzeppelin));
        let account_class_hash = transaction.account_class_hash;

        let mp_transaction = transaction.clone().from_deploy(Starknet::chain_id()).unwrap();

        let tx_hash = mp_transaction.hash;
        transaction.signature = sign_message_hash(tx_hash);

        let address = mp_transaction.sender_address;
        set_infinite_tokens::<MockRuntime>(address);
        set_signer(address, AccountType::V0(AccountTypeV0Inner::Openzeppelin));

        assert_ok!(Starknet::deploy_account(none_origin, transaction));
        assert_eq!(Starknet::contract_class_hash_by_address(address).unwrap(), account_class_hash);
    });
}

#[test]
fn given_contract_run_deploy_account_openzeppelin_with_incorrect_signature_then_it_fails() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();

        let mut transaction = get_deploy_account_dummy(*SALT, AccountType::V0(AccountTypeV0Inner::Openzeppelin));
        transaction.signature = bounded_vec!(Felt252Wrapper::ONE, Felt252Wrapper::ONE);

        let address = transaction.clone().from_deploy(Starknet::chain_id()).unwrap().sender_address;
        set_signer(address, AccountType::V0(AccountTypeV0Inner::Openzeppelin));

        assert_err!(
            Starknet::deploy_account(none_origin, transaction),
            Error::<MockRuntime>::TransactionExecutionFailed
        );
    });
}

#[test]
fn given_contract_run_deploy_account_argent_tx_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();

        let mut transaction = get_deploy_account_dummy(*SALT, AccountType::V0(AccountTypeV0Inner::Argent));
        let account_class_hash = transaction.account_class_hash;

        let mp_transaction = transaction.clone().from_deploy(Starknet::chain_id()).unwrap();

        let tx_hash = mp_transaction.hash;
        transaction.signature = sign_message_hash(tx_hash);

        let address = mp_transaction.sender_address;
        set_infinite_tokens::<MockRuntime>(address);
        set_signer(address, AccountType::V0(AccountTypeV0Inner::Argent));

        assert_ok!(Starknet::deploy_account(none_origin, transaction));
        assert_eq!(Starknet::contract_class_hash_by_address(address).unwrap(), account_class_hash);
    });
}

#[test]
fn given_contract_run_deploy_account_argent_with_incorrect_signature_then_it_fails() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();
        let mut transaction = get_deploy_account_dummy(*SALT, AccountType::V0(AccountTypeV0Inner::Argent));
        transaction.signature = bounded_vec!(Felt252Wrapper::ONE, Felt252Wrapper::ONE);

        let address = transaction.clone().from_deploy(Starknet::chain_id()).unwrap().sender_address;
        set_signer(address, AccountType::V0(AccountTypeV0Inner::Argent));

        assert_err!(
            Starknet::deploy_account(none_origin, transaction),
            Error::<MockRuntime>::TransactionExecutionFailed
        );
    });
}

#[test]
fn given_contract_run_deploy_account_braavos_tx_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();
        let (_, proxy_class_hash, mut calldata) =
            account_helper(*SALT, AccountType::V0(AccountTypeV0Inner::BraavosProxy));
        calldata.push("0x1");
        calldata.push(ACCOUNT_PUBLIC_KEY);

        let tx_hash =
            Felt252Wrapper::from_hex_be("0x06a8bb3d81c2ad23db93f01f72f987feac5210a95bc530eabb6abfaa5a769944").unwrap();

        let mut signatures: Vec<Felt252Wrapper> = sign_message_hash(tx_hash).into();
        let empty_signatures = [Felt252Wrapper::ZERO; 8];
        signatures.append(&mut empty_signatures.to_vec());

        let transaction = DeployAccountTransaction {
            account_class_hash: proxy_class_hash,
            salt: *SALT,
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
            max_fee: Felt252Wrapper::from(u64::MAX),
            signature: signatures.try_into().unwrap(),
            is_query: false,
        };
        let transaction1 = transaction.clone().from_deploy(SN_GOERLI_CHAIN_ID);
        println!("this is transaction hash {}", transaction1.unwrap().hash.0);

        let address = transaction.clone().from_deploy(Starknet::chain_id()).unwrap().sender_address;
        set_infinite_tokens::<MockRuntime>(address);
        set_signer(address, AccountType::V0(AccountTypeV0Inner::Braavos));

        assert_ok!(Starknet::deploy_account(none_origin, transaction));
        assert_eq!(Starknet::contract_class_hash_by_address(address).unwrap(), proxy_class_hash);
    });
}

#[test]
fn given_contract_run_deploy_account_braavos_with_incorrect_signature_then_it_fails() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();
        let (test_addr, proxy_class_hash, mut calldata) =
            account_helper(*SALT, AccountType::V0(AccountTypeV0Inner::BraavosProxy));
        calldata.push("0x1");
        calldata.push(ACCOUNT_PUBLIC_KEY);

        set_infinite_tokens::<MockRuntime>(test_addr);
        set_signer(test_addr, AccountType::V0(AccountTypeV0Inner::Braavos));

        let transaction = DeployAccountTransaction {
            account_class_hash: proxy_class_hash,
            salt: *SALT,
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
            max_fee: Felt252Wrapper::from(u64::MAX),
            signature: [Felt252Wrapper::ZERO; 10].to_vec().try_into().unwrap(),
            is_query: false,
        };

        assert_err!(
            Starknet::deploy_account(none_origin, transaction),
            Error::<MockRuntime>::TransactionExecutionFailed
        );
    });
}

#[test]
fn test_verify_tx_longevity() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let transaction = get_deploy_account_dummy(*SALT, AccountType::V0(AccountTypeV0Inner::NoValidate));

        let validate_result =
            Starknet::validate_unsigned(TransactionSource::InBlock, &crate::Call::deploy_account { transaction });

        assert!(validate_result.unwrap().longevity == TransactionLongevity::get());
    });
}

fn set_signer(address: Felt252Wrapper, account_type: AccountType) {
    let (var_name, args) = match account_type {
        AccountType::V0(AccountTypeV0Inner::Argent) => ("_signer", vec![]),
        AccountType::V0(AccountTypeV0Inner::Braavos) => ("Account_signers", vec![Felt252Wrapper::ZERO]),
        AccountType::V0(AccountTypeV0Inner::Openzeppelin) => ("Account_public_key", vec![]),
        _ => return,
    };
    StorageView::<MockRuntime>::insert(
        get_storage_key(&address, var_name, &args, 0),
        Felt252Wrapper::from_hex_be(ACCOUNT_PUBLIC_KEY).unwrap(),
    );
}
