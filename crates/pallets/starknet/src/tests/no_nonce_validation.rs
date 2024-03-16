use frame_support::assert_ok;
use starknet_api::core::Nonce;
use starknet_api::hash::StarkFelt;

use super::mock::{new_test_ext, no_nonce_validation_mock};
use crate::tests::constants::SALT;
use crate::tests::mock::no_nonce_validation_mock::{basic_test_setup, RuntimeOrigin, Starknet};
use crate::tests::mock::{get_account_address, AccountType, AccountTypeV0Inner};
use crate::tests::utils::get_contract_class;
use crate::tests::{get_declare_dummy, get_deploy_account_dummy, get_invoke_dummy, set_infinite_tokens};

#[test]
fn given_invoke_tx_with_invalid_nonce_then_it_does_nothing() {
    new_test_ext::<no_nonce_validation_mock::MockRuntime>().execute_with(|| {
        basic_test_setup(2);

        let none_origin = RuntimeOrigin::none();

        let transaction = get_invoke_dummy(Starknet::chain_id(), Nonce(StarkFelt::THREE));
        let sender_address = transaction.tx.sender_address();

        assert_ok!(Starknet::invoke(none_origin, transaction.into()));

        // check nonce is still 0
        let nonce = Starknet::nonce(sender_address);
        assert_eq!(nonce, Nonce(StarkFelt::ZERO));
    });
}

#[test]
fn given_declare_tx_with_invalid_nonce_then_it_works() {
    new_test_ext::<no_nonce_validation_mock::MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();

        let transaction = get_declare_dummy(
            Starknet::chain_id(),
            Nonce(StarkFelt::THREE),
            AccountType::V0(AccountTypeV0Inner::Openzeppelin),
        );
        let erc20_class_hash = transaction.class_hash();
        let sender_address = transaction.tx.sender_address();

        let contract_class = get_contract_class("ERC20.json", 0);

        assert_ok!(Starknet::declare(none_origin, transaction.clone()));
        assert_eq!(Starknet::contract_class_by_class_hash(erc20_class_hash.0).unwrap(), contract_class);

        // check nonce is still 0
        let nonce = Starknet::nonce(sender_address);
        assert_eq!(nonce, Nonce(StarkFelt::from(0u128)));
    });
}

#[test]
fn given_deploy_account_tx_with_invalid_nonce_then_it_works() {
    new_test_ext::<no_nonce_validation_mock::MockRuntime>().execute_with(|| {
        basic_test_setup(2);
        let none_origin = RuntimeOrigin::none();

        let transaction = get_deploy_account_dummy(
            Starknet::chain_id(),
            Nonce(StarkFelt::THREE),
            *SALT,
            AccountType::V0(AccountTypeV0Inner::NoValidate),
        );
        let account_class_hash = transaction.tx.class_hash();

        let address = get_account_address(Some(*SALT), AccountType::V0(AccountTypeV0Inner::NoValidate));
        set_infinite_tokens::<no_nonce_validation_mock::MockRuntime>(&address);

        assert_ok!(Starknet::deploy_account(none_origin, transaction));
        assert_eq!(Starknet::contract_class_hash_by_address(address), account_class_hash.0);

        // check nonce is still 0
        let nonce = Starknet::nonce(address);
        assert_eq!(nonce, Nonce(StarkFelt::from(0u128)));
    });
}
