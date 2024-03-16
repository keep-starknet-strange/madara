use std::sync::Arc;

use blockifier::transaction::transactions::DeployAccountTransaction;
use frame_support::{assert_err, assert_ok};
use mp_felt::Felt252Wrapper;
use mp_transactions::compute_hash::ComputeTransactionHash;
use sp_runtime::traits::ValidateUnsigned;
use sp_runtime::transaction_validity::{InvalidTransaction, TransactionSource, TransactionValidityError};
use starknet_api::core::{calculate_contract_address, ClassHash, ContractAddress, Nonce};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::{
    Calldata, ContractAddressSalt, DeployAccountTransactionV1, Event as StarknetEvent, EventContent, EventData,
    EventKey, Fee, TransactionSignature,
};
use starknet_core::utils::get_selector_from_name;
use starknet_crypto::FieldElement;

use super::mock::default_mock::*;
use super::mock::*;
use super::utils::{sign_message_hash, sign_message_hash_braavos};
use crate::tests::constants::{ACCOUNT_PUBLIC_KEY, SALT, TRANSFER_SELECTOR_NAME};
use crate::tests::{get_deploy_account_dummy, set_infinite_tokens, set_nonce};
use crate::{Error, StorageView};

fn deploy_v1_to_blockifier_deploy(
    tx: DeployAccountTransactionV1,
    chain_id: Felt252Wrapper,
) -> DeployAccountTransaction {
    let tx_hash = tx.compute_hash(chain_id, false);
    let contract_address = calculate_contract_address(
        tx.contract_address_salt,
        tx.class_hash,
        &tx.constructor_calldata,
        Default::default(),
    )
    .unwrap();

    DeployAccountTransaction::new(
        starknet_api::transaction::DeployAccountTransaction::V1(tx),
        tx_hash,
        contract_address,
    )
}

fn helper_create_deploy_account_tx(
    chain_id: Felt252Wrapper,
    salt: ContractAddressSalt,
    calldata: Calldata,
    account_class_hash: ClassHash,
) -> DeployAccountTransaction {
    let tx = DeployAccountTransactionV1 {
        nonce: Nonce(StarkFelt::ZERO),
        max_fee: Fee(u128::MAX),
        signature: TransactionSignature(vec![]),
        contract_address_salt: salt,
        constructor_calldata: calldata,
        class_hash: account_class_hash,
    };

    deploy_v1_to_blockifier_deploy(tx, chain_id)
}

#[test]
fn given_contract_run_deploy_account_tx_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();
        let chain_id = Starknet::chain_id();
        // TEST ACCOUNT CONTRACT
        // - ref testnet tx(0x0751b4b5b95652ad71b1721845882c3852af17e2ed0c8d93554b5b292abb9810)
        let salt = ContractAddressSalt(
            StarkFelt::try_from("0x03b37cbe4e9eac89d54c5f7cc6329a63a63e8c8db2bf936f981041e086752463").unwrap(),
        );
        let (account_class_hash, calldata) = account_helper(AccountType::V0(AccountTypeV0Inner::NoValidate));

        let deploy_tx = helper_create_deploy_account_tx(chain_id, salt, calldata, account_class_hash);
        let tx_hash = deploy_tx.tx_hash;
        let contract_address = deploy_tx.contract_address;

        set_infinite_tokens::<MockRuntime>(&contract_address);

        assert_ok!(Starknet::deploy_account(none_origin, deploy_tx));
        assert_eq!(Starknet::contract_class_hash_by_address(contract_address), account_class_hash.0);

        let expected_fee_transfer_event = StarknetEvent {
            content: EventContent {
                keys: vec![EventKey(
                    Felt252Wrapper::from(get_selector_from_name(TRANSFER_SELECTOR_NAME).unwrap()).into(),
                )],
                data: EventData(vec![
                    contract_address.0.0,                   // From
                    StarkFelt::try_from("0xdead").unwrap(), // To
                    StarkFelt::try_from("0xb64e").unwrap(), // Amount low
                    StarkFelt::from(0u128),                 // Amount high
                ]),
            },
            from_address: Starknet::fee_token_addresses().eth_fee_token_address,
        };

        let events = Starknet::tx_events(tx_hash);
        assert_eq!(expected_fee_transfer_event, events.last().unwrap().clone());
    });
}

#[test]
fn given_contract_run_deploy_account_tx_twice_fails() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let chain_id = Starknet::chain_id();
        let (account_class_hash, calldata) = account_helper(AccountType::V0(AccountTypeV0Inner::NoValidate));
        let deploy_tx = helper_create_deploy_account_tx(chain_id, *SALT, calldata, account_class_hash);
        set_infinite_tokens::<MockRuntime>(&deploy_tx.contract_address);

        assert_ok!(Starknet::deploy_account(RuntimeOrigin::none(), deploy_tx.clone()));
        assert_eq!(Starknet::contract_class_hash_by_address(deploy_tx.contract_address), account_class_hash.0);
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

        let chain_id = Starknet::chain_id();
        let account_class_hash = get_account_class_hash(AccountType::V0(AccountTypeV0Inner::Argent));

        let deploy_tx = helper_create_deploy_account_tx(
            chain_id,
            ContractAddressSalt(StarkFelt::ZERO),
            Calldata(Default::default()),
            account_class_hash,
        );

        assert_err!(Starknet::deploy_account(none_origin, deploy_tx), Error::<MockRuntime>::TransactionExecutionFailed);
    });
}

#[test]
fn given_contract_run_deploy_account_tx_fails_wrong_tx_version() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();
        let chain_id = Starknet::chain_id();

        let deploy_tx = get_deploy_account_dummy(
            chain_id,
            Nonce(StarkFelt::ZERO),
            *SALT,
            AccountType::V0(AccountTypeV0Inner::Argent),
        );

        assert_err!(Starknet::deploy_account(none_origin, deploy_tx), Error::<MockRuntime>::TransactionExecutionFailed);
    });
}

#[test]
fn given_contract_run_deploy_account_openzeppelin_tx_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();
        let chain_id = Starknet::chain_id();

        let (account_class_hash, calldata) = account_helper(AccountType::V0(AccountTypeV0Inner::Openzeppelin));

        let deploy_tx = {
            let mut tx = DeployAccountTransactionV1 {
                nonce: Nonce(StarkFelt::ZERO),
                max_fee: Fee(u128::MAX),
                signature: TransactionSignature(vec![]),
                contract_address_salt: *SALT,
                constructor_calldata: calldata,
                class_hash: account_class_hash,
            };
            let tx_hash = tx.compute_hash(chain_id, false);
            tx.signature = sign_message_hash(tx_hash);
            let contract_address = calculate_contract_address(
                tx.contract_address_salt,
                tx.class_hash,
                &tx.constructor_calldata,
                Default::default(),
            )
            .unwrap();

            DeployAccountTransaction::new(
                starknet_api::transaction::DeployAccountTransaction::V1(tx),
                tx_hash,
                contract_address,
            )
        };
        let contract_address = deploy_tx.contract_address;

        set_infinite_tokens::<MockRuntime>(&deploy_tx.contract_address);
        set_signer(deploy_tx.contract_address, AccountType::V0(AccountTypeV0Inner::Openzeppelin));

        assert_ok!(Starknet::deploy_account(none_origin, deploy_tx));
        assert_eq!(Starknet::contract_class_hash_by_address(contract_address), account_class_hash.0);
    });
}

#[test]
fn given_contract_run_deploy_account_openzeppelin_with_incorrect_signature_then_it_fails() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();
        let chain_id = Starknet::chain_id();
        let (account_class_hash, calldata) = account_helper(AccountType::V0(AccountTypeV0Inner::Openzeppelin));

        let deploy_tx = {
            let tx = DeployAccountTransactionV1 {
                nonce: Nonce(StarkFelt::ZERO),
                max_fee: Fee(u128::MAX),
                signature: TransactionSignature(vec![StarkFelt::ONE, StarkFelt::ONE]),
                contract_address_salt: *SALT,
                constructor_calldata: calldata,
                class_hash: account_class_hash,
            };
            deploy_v1_to_blockifier_deploy(tx, chain_id)
        };

        set_signer(deploy_tx.contract_address, AccountType::V0(AccountTypeV0Inner::Openzeppelin));

        assert_err!(Starknet::deploy_account(none_origin, deploy_tx), Error::<MockRuntime>::TransactionExecutionFailed);
    });
}

#[test]
fn given_contract_run_deploy_account_argent_tx_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();
        let chain_id = Starknet::chain_id();

        let (account_class_hash, calldata) = account_helper(AccountType::V0(AccountTypeV0Inner::Argent));

        let deploy_tx = {
            let mut tx = DeployAccountTransactionV1 {
                nonce: Nonce(StarkFelt::ZERO),
                max_fee: Fee(u128::MAX),
                signature: TransactionSignature(vec![]),
                contract_address_salt: *SALT,
                constructor_calldata: calldata,
                class_hash: account_class_hash,
            };
            let tx_hash = tx.compute_hash(chain_id, false);
            tx.signature = sign_message_hash(tx_hash);
            let contract_address = calculate_contract_address(
                tx.contract_address_salt,
                tx.class_hash,
                &tx.constructor_calldata,
                Default::default(),
            )
            .unwrap();

            DeployAccountTransaction::new(
                starknet_api::transaction::DeployAccountTransaction::V1(tx),
                tx_hash,
                contract_address,
            )
        };
        let contract_address = deploy_tx.contract_address;

        set_infinite_tokens::<MockRuntime>(&contract_address);
        set_signer(deploy_tx.contract_address, AccountType::V0(AccountTypeV0Inner::Argent));

        assert_ok!(Starknet::deploy_account(none_origin, deploy_tx));
        assert_eq!(Starknet::contract_class_hash_by_address(contract_address), account_class_hash.0);
    });
}

#[test]
fn given_contract_run_deploy_account_argent_with_incorrect_signature_then_it_fails() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();
        let chain_id = Starknet::chain_id();
        let (account_class_hash, calldata) = account_helper(AccountType::V0(AccountTypeV0Inner::Argent));

        let deploy_tx = {
            let tx = DeployAccountTransactionV1 {
                nonce: Nonce(StarkFelt::ZERO),
                max_fee: Fee(u128::MAX),
                signature: TransactionSignature(vec![StarkFelt::ONE, StarkFelt::ONE]),
                contract_address_salt: *SALT,
                constructor_calldata: calldata,
                class_hash: account_class_hash,
            };
            deploy_v1_to_blockifier_deploy(tx, chain_id)
        };

        set_signer(deploy_tx.contract_address, AccountType::V0(AccountTypeV0Inner::Argent));

        assert_err!(Starknet::deploy_account(none_origin, deploy_tx), Error::<MockRuntime>::TransactionExecutionFailed);
    });
}

#[test]
fn given_contract_run_deploy_account_braavos_tx_works() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();
        let chain_id = Starknet::chain_id();
        let (proxy_class_hash, calldata) = account_helper(AccountType::V0(AccountTypeV0Inner::BraavosProxy));
        let mut calldata = Arc::into_inner(calldata.0).unwrap();
        calldata.push(StarkFelt::ONE);
        calldata.push(StarkFelt::try_from(ACCOUNT_PUBLIC_KEY).unwrap());
        let calldata = Calldata(Arc::new(calldata));

        let deploy_tx = {
            let mut tx = DeployAccountTransactionV1 {
                nonce: Nonce(StarkFelt::ZERO),
                max_fee: Fee(u128::MAX),
                signature: TransactionSignature(vec![]),
                contract_address_salt: *SALT,
                constructor_calldata: calldata,
                class_hash: proxy_class_hash,
            };
            let tx_hash = tx.compute_hash(chain_id, false);
            tx.signature = sign_message_hash_braavos(tx_hash, StarkFelt::ZERO, &[StarkFelt::ZERO; 7]);
            let contract_address = calculate_contract_address(
                tx.contract_address_salt,
                tx.class_hash,
                &tx.constructor_calldata,
                Default::default(),
            )
            .unwrap();

            DeployAccountTransaction::new(
                starknet_api::transaction::DeployAccountTransaction::V1(tx),
                tx_hash,
                contract_address,
            )
        };
        let contract_address = deploy_tx.contract_address;

        set_infinite_tokens::<MockRuntime>(&contract_address);
        set_signer(deploy_tx.contract_address, AccountType::V0(AccountTypeV0Inner::Braavos));

        assert_ok!(Starknet::deploy_account(none_origin, deploy_tx));
        assert_eq!(Starknet::contract_class_hash_by_address(contract_address), proxy_class_hash.0);
    });
}

#[test]
fn given_contract_run_deploy_account_braavos_tx_works_whis_hardware_signer() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();
        let chain_id = Starknet::chain_id();
        let (proxy_class_hash, calldata) = account_helper(AccountType::V0(AccountTypeV0Inner::BraavosProxy));
        let mut calldata = Arc::into_inner(calldata.0).unwrap();
        calldata.push(StarkFelt::ONE);
        calldata.push(StarkFelt::try_from(ACCOUNT_PUBLIC_KEY).unwrap());
        let calldata = Calldata(Arc::new(calldata));

        let deploy_tx = {
            let mut tx = DeployAccountTransactionV1 {
                nonce: Nonce(StarkFelt::ZERO),
                max_fee: Fee(u128::MAX),
                signature: TransactionSignature(vec![]),
                contract_address_salt: *SALT,
                constructor_calldata: calldata,
                class_hash: proxy_class_hash,
            };
            let tx_hash = tx.compute_hash(chain_id, false);
            // signer fields are hardware public key generated from some random private key
            // it's possible to add only one additional secp256r1 signer
            let signer_model = [
                StarkFelt::try_from("0x23fc01adbb70af88935aeaecde1240ea").unwrap(), /* signer_0= pk_x_uint256
                                                                                     * low 128 bits */
                StarkFelt::try_from("0xea0cb2b3f76a88bba0d8dc7556c40df9").unwrap(), /* signer_1= pk_x_uint256
                                                                                     * high 128 bits */
                StarkFelt::try_from("0x663b66d81aa5eed14537e814b02745c0").unwrap(), /* signer_2= pk_y_uint256
                                                                                     * low 128 bits */
                StarkFelt::try_from("0x76d91b936d094b864af4cfaaeec89fb1").unwrap(), /* signer_3= pk_y_uint256
                                                                                     * high 128 bits */
                StarkFelt::TWO,  // type= SIGNER_TYPE_SECP256R1
                StarkFelt::ZERO, // reserved_0
                StarkFelt::ZERO, // reserved_1
            ];
            tx.signature = sign_message_hash_braavos(tx_hash, StarkFelt::ZERO, &signer_model);
            let contract_address = calculate_contract_address(
                tx.contract_address_salt,
                tx.class_hash,
                &tx.constructor_calldata,
                Default::default(),
            )
            .unwrap();

            DeployAccountTransaction::new(
                starknet_api::transaction::DeployAccountTransaction::V1(tx),
                tx_hash,
                contract_address,
            )
        };
        let contract_address = deploy_tx.contract_address;

        set_infinite_tokens::<MockRuntime>(&contract_address);
        set_signer(deploy_tx.contract_address, AccountType::V0(AccountTypeV0Inner::Braavos));

        assert_ok!(Starknet::deploy_account(none_origin, deploy_tx));
        assert_eq!(Starknet::contract_class_hash_by_address(contract_address), proxy_class_hash.0);
    });
}

#[test]
fn given_contract_run_deploy_account_braavos_with_incorrect_signature_then_it_fails() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();
        let chain_id = Starknet::chain_id();
        let (proxy_class_hash, calldata) = account_helper(AccountType::V0(AccountTypeV0Inner::BraavosProxy));
        let mut calldata = Arc::into_inner(calldata.0).unwrap();
        calldata.push(StarkFelt::ZERO);
        calldata.push(StarkFelt::try_from(ACCOUNT_PUBLIC_KEY).unwrap());
        let calldata = Calldata(Arc::new(calldata));

        let deploy_tx = {
            let tx = DeployAccountTransactionV1 {
                nonce: Nonce(StarkFelt::ZERO),
                max_fee: Fee(u128::MAX),
                signature: TransactionSignature([StarkFelt::ONE; 10].to_vec()),
                contract_address_salt: *SALT,
                constructor_calldata: calldata,
                class_hash: proxy_class_hash,
            };
            deploy_v1_to_blockifier_deploy(tx, chain_id)
        };

        set_infinite_tokens::<MockRuntime>(&deploy_tx.contract_address);
        set_signer(deploy_tx.contract_address, AccountType::V0(AccountTypeV0Inner::Braavos));

        assert_err!(Starknet::deploy_account(none_origin, deploy_tx), Error::<MockRuntime>::TransactionExecutionFailed);
    });
}

#[test]
fn test_verify_tx_longevity() {
    new_test_ext::<MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let chain_id = Starknet::chain_id();

        let transaction = get_deploy_account_dummy(
            chain_id,
            Nonce(StarkFelt::ZERO),
            *SALT,
            AccountType::V0(AccountTypeV0Inner::NoValidate),
        );

        set_infinite_tokens::<MockRuntime>(&transaction.contract_address);
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

        let chain_id = Starknet::chain_id();
        let transaction = get_deploy_account_dummy(
            chain_id,
            Nonce(StarkFelt::ZERO),
            *SALT,
            AccountType::V0(AccountTypeV0Inner::NoValidate),
        );
        let contract_address = transaction.contract_address;
        set_infinite_tokens::<MockRuntime>(&contract_address);

        let tx_source = TransactionSource::InBlock;
        let call = crate::Call::deploy_account { transaction };

        assert!(Starknet::validate_unsigned(tx_source, &call).is_ok());

        set_nonce::<MockRuntime>(&contract_address, &Nonce(StarkFelt::from(1u64)));

        assert_eq!(
            Starknet::validate_unsigned(tx_source, &call),
            Err(TransactionValidityError::Invalid(InvalidTransaction::BadProof))
        );
    });
}
