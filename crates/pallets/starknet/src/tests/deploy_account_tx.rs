use frame_support::{assert_err, assert_ok};
use mp_felt::Felt252Wrapper;
use mp_transactions::compute_hash::ComputeTransactionHash;
use mp_transactions::DeployAccountTransaction;
use sp_runtime::traits::ValidateUnsigned;
use sp_runtime::transaction_validity::{InvalidTransaction, TransactionSource, TransactionValidityError};
use starknet_api::api_core::{ContractAddress, Nonce};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::{Event as StarknetEvent, EventContent, EventData, EventKey};
use starknet_crypto::FieldElement;

use super::mock::default_mock::*;
use super::mock::*;
use super::utils::sign_message_hash;
use crate::tests::constants::{ACCOUNT_PUBLIC_KEY, SALT};
use crate::tests::{get_deploy_account_dummy, set_infinite_tokens, set_nonce};
use crate::{Config, Error, Event, StorageView};

#[test]
fn given_contract_run_deploy_account_tx_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();
        // TEST ACCOUNT CONTRACT
        // - ref testnet tx(0x0751b4b5b95652ad71b1721845882c3852af17e2ed0c8d93554b5b292abb9810)
        let salt =
            Felt252Wrapper::from_hex_be("0x03b37cbe4e9eac89d54c5f7cc6329a63a63e8c8db2bf936f981041e086752463").unwrap();
        let (account_class_hash, calldata) = account_helper(AccountType::V0(AccountTypeV0Inner::NoValidate));

        let deploy_tx = DeployAccountTransaction {
            nonce: Felt252Wrapper::ZERO,
            max_fee: u128::MAX,
            signature: vec![],
            contract_address_salt: salt,
            constructor_calldata: calldata.0.iter().map(|e| Felt252Wrapper::from(*e)).collect(),
            class_hash: account_class_hash.into(),
        };

        let address = deploy_tx.account_address().into();
        set_infinite_tokens::<MockRuntime>(&address);

        assert_ok!(Starknet::deploy_account(none_origin, deploy_tx));
        assert_eq!(Starknet::contract_class_hash_by_address(address), account_class_hash);

        let expected_fee_transfer_event = Event::StarknetEvent(StarknetEvent {
            content: EventContent {
                keys: vec![EventKey(
                    StarkFelt::try_from("0x0099cd8bde557814842a3121e8ddfd433a539b8c9f14bf31ebf108d12e6196e9").unwrap(),
                )],
                data: EventData(vec![
                    address.0.0,                            // From
                    StarkFelt::try_from("0xdead").unwrap(), // To
                    StarkFelt::try_from("0x195a").unwrap(), // Amount low
                    StarkFelt::from(0u128),                 // Amount high
                ]),
            },
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

        let (account_class_hash, calldata) = account_helper(AccountType::V0(AccountTypeV0Inner::NoValidate));

        let deploy_tx = DeployAccountTransaction {
            max_fee: u128::MAX,
            signature: vec![],
            nonce: Felt252Wrapper::ZERO,
            contract_address_salt: *SALT,
            constructor_calldata: calldata.0.iter().map(|e| Felt252Wrapper::from(*e)).collect(),
            class_hash: account_class_hash.into(),
        };

        let address = deploy_tx.account_address().into();
        set_infinite_tokens::<MockRuntime>(&address);

        assert_ok!(Starknet::deploy_account(RuntimeOrigin::none(), deploy_tx.clone()));
        assert_eq!(Starknet::contract_class_hash_by_address(address), account_class_hash);
        assert_err!(
            Starknet::deploy_account(RuntimeOrigin::none(), deploy_tx),
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
            class_hash: account_class_hash.into(),
            constructor_calldata: vec![],
            contract_address_salt: Felt252Wrapper::ZERO,
            nonce: Felt252Wrapper::ZERO,
            max_fee: u128::MAX,
            signature: vec![],
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

        let transaction =
            get_deploy_account_dummy(Felt252Wrapper::ZERO, *SALT, AccountType::V0(AccountTypeV0Inner::Argent));

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

        let (account_class_hash, calldata) = account_helper(AccountType::V0(AccountTypeV0Inner::Openzeppelin));

        let mut deploy_tx = DeployAccountTransaction {
            max_fee: u128::MAX,
            signature: vec![],
            nonce: Felt252Wrapper::ZERO,
            contract_address_salt: *SALT,
            constructor_calldata: calldata.0.iter().map(|e| Felt252Wrapper::from(*e)).collect(),
            class_hash: account_class_hash.into(),
        };

        let chain_id = Starknet::chain_id();
        let tx_hash = deploy_tx.compute_hash::<<MockRuntime as Config>::SystemHash>(chain_id, false);
        deploy_tx.signature = sign_message_hash(tx_hash);
        let address = deploy_tx.account_address().into();

        set_infinite_tokens::<MockRuntime>(&address);
        set_signer(address, AccountType::V0(AccountTypeV0Inner::Openzeppelin));

        assert_ok!(Starknet::deploy_account(none_origin, deploy_tx));
        assert_eq!(Starknet::contract_class_hash_by_address(address), account_class_hash);
    });
}

#[test]
fn given_contract_run_deploy_account_openzeppelin_with_incorrect_signature_then_it_fails() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();
        let (account_class_hash, calldata) = account_helper(AccountType::V0(AccountTypeV0Inner::Openzeppelin));

        let mut deploy_tx = DeployAccountTransaction {
            max_fee: u128::MAX,
            signature: vec![],
            nonce: Felt252Wrapper::ZERO,
            contract_address_salt: *SALT,
            constructor_calldata: calldata.0.iter().map(|e| Felt252Wrapper::from(*e)).collect(),
            class_hash: account_class_hash.into(),
        };
        deploy_tx.signature = vec![Felt252Wrapper::ONE, Felt252Wrapper::ONE];

        let address = deploy_tx.account_address().into();
        set_signer(address, AccountType::V0(AccountTypeV0Inner::Openzeppelin));

        assert_err!(Starknet::deploy_account(none_origin, deploy_tx), Error::<MockRuntime>::TransactionExecutionFailed);
    });
}

#[test]
fn given_contract_run_deploy_account_argent_tx_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();

        let (account_class_hash, calldata) = account_helper(AccountType::V0(AccountTypeV0Inner::Openzeppelin));

        let mut deploy_tx = DeployAccountTransaction {
            max_fee: u128::MAX,
            signature: vec![],
            nonce: Felt252Wrapper::ZERO,
            contract_address_salt: *SALT,
            constructor_calldata: calldata.0.iter().map(|e| Felt252Wrapper::from(*e)).collect(),
            class_hash: account_class_hash.into(),
        };

        let chain_id = Starknet::chain_id();
        let tx_hash = deploy_tx.compute_hash::<<MockRuntime as Config>::SystemHash>(chain_id, false);
        deploy_tx.signature = sign_message_hash(tx_hash);

        let address = deploy_tx.account_address().into();
        set_infinite_tokens::<MockRuntime>(&address);
        set_signer(address, AccountType::V0(AccountTypeV0Inner::Argent));

        assert_ok!(Starknet::deploy_account(none_origin, deploy_tx));
        assert_eq!(Starknet::contract_class_hash_by_address(address), account_class_hash);
    });
}

#[test]
fn given_contract_run_deploy_account_argent_with_incorrect_signature_then_it_fails() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();
        let (account_class_hash, calldata) = account_helper(AccountType::V0(AccountTypeV0Inner::Openzeppelin));

        let mut deploy_tx = DeployAccountTransaction {
            max_fee: u128::MAX,
            signature: vec![],
            nonce: Felt252Wrapper::ZERO,
            contract_address_salt: *SALT,
            constructor_calldata: calldata.0.iter().map(|e| Felt252Wrapper::from(*e)).collect(),
            class_hash: account_class_hash.into(),
        };

        deploy_tx.signature = vec![Felt252Wrapper::ONE, Felt252Wrapper::ONE];
        let address = deploy_tx.account_address().into();

        set_signer(address, AccountType::V0(AccountTypeV0Inner::Argent));

        assert_err!(Starknet::deploy_account(none_origin, deploy_tx), Error::<MockRuntime>::TransactionExecutionFailed);
    });
}

#[test]
fn given_contract_run_deploy_account_braavos_tx_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();
        let (proxy_class_hash, calldata) = account_helper(AccountType::V0(AccountTypeV0Inner::BraavosProxy));
        let mut calldata: Vec<_> = calldata.0.iter().map(|e| Felt252Wrapper::from(*e)).collect();
        calldata.push(Felt252Wrapper::ONE);
        calldata.push(Felt252Wrapper::from_hex_be(ACCOUNT_PUBLIC_KEY).unwrap());

        let mut deploy_tx = DeployAccountTransaction {
            max_fee: u64::MAX as u128,
            signature: vec![],
            nonce: Felt252Wrapper::ZERO,
            contract_address_salt: *SALT,
            constructor_calldata: calldata,
            class_hash: proxy_class_hash.into(),
        };

        // Braavos has a complicated signature mecanism, they add stuffs around the tx_hash and then sign
        // the whole thing. This hardcoded value is the expected "thing" that bravos expect you to
        // sign for this transaction. Roll with it for now
        let value_to_sign =
            Felt252Wrapper::from_hex_be("0x06a8bb3d81c2ad23db93f01f72f987feac5210a95bc530eabb6abfaa5a769944").unwrap();
        let mut signatures = sign_message_hash(value_to_sign);
        signatures.extend_from_slice(&[Felt252Wrapper::ZERO; 8]);
        deploy_tx.signature = signatures;

        let address = deploy_tx.account_address().into();
        set_infinite_tokens::<MockRuntime>(&address);
        set_signer(address, AccountType::V0(AccountTypeV0Inner::Braavos));

        assert_ok!(Starknet::deploy_account(none_origin, deploy_tx));
        assert_eq!(Starknet::contract_class_hash_by_address(address), proxy_class_hash);
    });
}

#[test]
fn given_contract_run_deploy_account_braavos_with_incorrect_signature_then_it_fails() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();
        let (proxy_class_hash, calldata) = account_helper(AccountType::V0(AccountTypeV0Inner::BraavosProxy));
        let mut calldata = calldata.0.iter().map(|e| Felt252Wrapper::from(*e)).collect::<Vec<_>>();
        calldata.push(Felt252Wrapper::ZERO);
        calldata.push(Felt252Wrapper::from_hex_be(ACCOUNT_PUBLIC_KEY).unwrap());

        let deploy_tx = DeployAccountTransaction {
            class_hash: proxy_class_hash.into(),
            contract_address_salt: *SALT,
            constructor_calldata: calldata,
            nonce: Felt252Wrapper::ZERO,
            max_fee: u128::MAX,
            signature: [Felt252Wrapper::ZERO; 10].to_vec(),
        };

        let address = deploy_tx.account_address().into();
        set_infinite_tokens::<MockRuntime>(&address);
        set_signer(address, AccountType::V0(AccountTypeV0Inner::Braavos));

        assert_err!(Starknet::deploy_account(none_origin, deploy_tx), Error::<MockRuntime>::TransactionExecutionFailed);
    });
}

#[test]
fn test_verify_tx_longevity() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let transaction =
            get_deploy_account_dummy(Felt252Wrapper::ZERO, *SALT, AccountType::V0(AccountTypeV0Inner::NoValidate));

        let validate_result =
            Starknet::validate_unsigned(TransactionSource::InBlock, &crate::Call::deploy_account { transaction });

        assert!(validate_result.unwrap().longevity == TransactionLongevity::get());
    });
}

fn set_signer(address: ContractAddress, account_type: AccountType) {
    let (var_name, args) = match account_type {
        AccountType::V0(AccountTypeV0Inner::Argent) => ("_signer", vec![]),
        AccountType::V0(AccountTypeV0Inner::Braavos) => ("Account_signers", vec![FieldElement::ZERO]),
        AccountType::V0(AccountTypeV0Inner::Openzeppelin) => ("Account_public_key", vec![]),
        _ => return,
    };
    StorageView::<MockRuntime>::insert(
        get_storage_key(&address, var_name, &args, 0),
        StarkFelt::try_from(ACCOUNT_PUBLIC_KEY).unwrap(),
    );
}

#[test]
fn test_verify_nonce_in_unsigned_tx() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let transaction =
            get_deploy_account_dummy(Felt252Wrapper::ZERO, *SALT, AccountType::V0(AccountTypeV0Inner::NoValidate));

        let tx_sender = transaction.account_address().into();
        let tx_source = TransactionSource::InBlock;
        let call = crate::Call::deploy_account { transaction };

        assert!(Starknet::validate_unsigned(tx_source, &call).is_ok());

        set_nonce::<MockRuntime>(&tx_sender, &Nonce(StarkFelt::from(1u64)));

        assert_eq!(
            Starknet::validate_unsigned(tx_source, &call),
            Err(TransactionValidityError::Invalid(InvalidTransaction::Stale))
        );
    });
}
