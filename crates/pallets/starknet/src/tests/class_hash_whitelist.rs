use frame_support::{assert_err, assert_ok};
use mp_felt::Felt252Wrapper;
use starknet_api::core::ClassHash;
use starknet_api::hash::StarkFelt;

use super::mock::default_mock::*;
use super::mock::*;
use crate::tests::utils::{create_declare_erc20_v1_transaction, create_declare_erc721_v1_transaction};
use crate::Error;

#[test]
fn whitelist_is_not_enabled_by_default() {
    let mut ext = new_test_ext::<MockRuntime>();
    ext.execute_with(|| {
        basic_test_setup(2);
        assert_eq!(Starknet::whitelisted_class_hashes(), vec![]);

        let none_origin = RuntimeOrigin::none();
        let chain_id = Starknet::chain_id();
        let account_addr = get_account_address(None, AccountType::V0(AccountTypeV0Inner::NoValidate));

        let transaction = create_declare_erc20_v1_transaction(
            chain_id,
            AccountType::V0(AccountTypeV0Inner::NoValidate),
            Some(account_addr),
            None,
            None,
        );

        assert_ok!(Starknet::declare(none_origin.clone(), transaction));
    });
}

#[test]
fn class_declaration_works_when_whitelisted() {
    let mut ext = new_test_ext::<MockRuntime>();
    ext.execute_with(|| {
        basic_test_setup(2);
        assert_eq!(Starknet::whitelisted_class_hashes(), vec![]);

        let none_origin = RuntimeOrigin::none();
        let chain_id = Starknet::chain_id();
        let account_addr = get_account_address(None, AccountType::V0(AccountTypeV0Inner::NoValidate));

        // Whitelist class hash
        let root_origin = RuntimeOrigin::root();
        let erc20_class_hash = ClassHash(
            StarkFelt::try_from("0x057eca87f4b19852cfd4551cf4706ababc6251a8781733a0a11cf8e94211da95").unwrap(),
        );

        assert_ok!(Starknet::whitelist_class_hash(root_origin, *erc20_class_hash));
        assert_eq!(Starknet::whitelisted_class_hashes(), vec![*erc20_class_hash]);

        let transaction = create_declare_erc20_v1_transaction(
            chain_id,
            AccountType::V0(AccountTypeV0Inner::NoValidate),
            Some(account_addr),
            None,
            None,
        );

        assert_ok!(Starknet::declare(none_origin.clone(), transaction));
    });
}

#[test]
fn class_declaration_fails_when_not_whitelisted() {
    let mut ext = new_test_ext::<MockRuntime>();
    ext.execute_with(|| {
        basic_test_setup(2);
        assert_eq!(Starknet::whitelisted_class_hashes(), vec![]);

        // Whitelist class hash
        let root_origin = RuntimeOrigin::root();
        let erc20_class_hash = ClassHash(StarkFelt::from(
            Felt252Wrapper::from_hex_be("0x057eca87f4b19852cfd4551cf4706ababc6251a8781733a0a11cf8e94211da95").unwrap(),
        ));

        assert_ok!(Starknet::whitelist_class_hash(root_origin, *erc20_class_hash));
        assert_eq!(Starknet::whitelisted_class_hashes(), vec![*erc20_class_hash]);

        let none_origin = RuntimeOrigin::none();
        let chain_id = Starknet::chain_id();
        let account_addr = get_account_address(None, AccountType::V0(AccountTypeV0Inner::NoValidate));

        let transaction = create_declare_erc721_v1_transaction(
            chain_id,
            AccountType::V0(AccountTypeV0Inner::NoValidate),
            Some(account_addr),
            None,
            None,
        );
        assert_err!(Starknet::declare(none_origin.clone(), transaction), Error::<MockRuntime>::ClassHashNotWhitelisted);
    });
}
