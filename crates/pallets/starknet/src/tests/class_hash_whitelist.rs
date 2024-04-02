use frame_support::{assert_err, assert_ok};
use mp_felt::Felt252Wrapper;
use mp_transactions::DeclareTransactionV1;
use starknet_api::api_core::ClassHash;
use starknet_api::hash::StarkFelt;

use super::mock::default_mock::*;
use super::mock::*;
use crate::tests::utils::get_contract_class;
use crate::Error;

#[test]
fn whitelist_is_not_enabled_by_default() {
    let mut ext = new_test_ext::<MockRuntime>();
    ext.execute_with(|| {
        basic_test_setup(2);
        assert_eq!(Starknet::whitelisted_class_hashes(), vec![]);

        let none_origin = RuntimeOrigin::none();
        let account_addr = get_account_address(None, AccountType::V0(AccountTypeV0Inner::NoValidate));

        let erc20_class = get_contract_class("ERC20.json", 0);
        let erc20_class_hash =
            Felt252Wrapper::from_hex_be("0x057eca87f4b19852cfd4551cf4706ababc6251a8781733a0a11cf8e94211da95").unwrap();

        let transaction = DeclareTransactionV1 {
            sender_address: account_addr.into(),
            class_hash: erc20_class_hash,
            nonce: Felt252Wrapper::ZERO,
            max_fee: u128::MAX,
            signature: vec![],
            offset_version: false,
        };

        assert_ok!(Starknet::declare(none_origin.clone(), transaction.clone().into(), erc20_class.clone()));
    });
}

#[test]
fn class_declaration_works_when_whitelisted() {
    let mut ext = new_test_ext::<MockRuntime>();
    ext.execute_with(|| {
        basic_test_setup(2);
        assert_eq!(Starknet::whitelisted_class_hashes(), vec![]);

        let none_origin = RuntimeOrigin::none();
        let account_addr = get_account_address(None, AccountType::V0(AccountTypeV0Inner::NoValidate));

        // Whitelist class hash
        let root_origin = RuntimeOrigin::root();
        let erc20_class_hash = ClassHash(StarkFelt::from(
            Felt252Wrapper::from_hex_be("0x057eca87f4b19852cfd4551cf4706ababc6251a8781733a0a11cf8e94211da95").unwrap(),
        ));

        assert_ok!(Starknet::whitelist_class_hash(root_origin, erc20_class_hash));
        assert_eq!(Starknet::whitelisted_class_hashes(), vec![erc20_class_hash]);

        let erc20_class = get_contract_class("ERC20.json", 0);
        let erc20_class_hash =
            Felt252Wrapper::from_hex_be("0x057eca87f4b19852cfd4551cf4706ababc6251a8781733a0a11cf8e94211da95").unwrap();

        let transaction = DeclareTransactionV1 {
            sender_address: account_addr.into(),
            class_hash: erc20_class_hash,
            nonce: Felt252Wrapper::ZERO,
            max_fee: u128::MAX,
            signature: vec![],
            offset_version: false,
        };

        assert_ok!(Starknet::declare(none_origin.clone(), transaction.clone().into(), erc20_class.clone()));
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

        assert_ok!(Starknet::whitelist_class_hash(root_origin, erc20_class_hash));
        assert_eq!(Starknet::whitelisted_class_hashes(), vec![erc20_class_hash]);

        let none_origin = RuntimeOrigin::none();
        let account_addr = get_account_address(None, AccountType::V0(AccountTypeV0Inner::NoValidate));

        let erc721 = get_contract_class("ERC721.json", 0);
        let erc721_hash =
            Felt252Wrapper::from_hex_be("0x057eca87f4b19852cfd4551cf5706ababc6251a8781733a0a11cf8e94211da95").unwrap();

        let transaction = DeclareTransactionV1 {
            sender_address: account_addr.into(),
            class_hash: erc721_hash,
            nonce: Felt252Wrapper::ZERO,
            max_fee: u128::MAX,
            signature: vec![],
            offset_version: false,
        };

        assert_err!(
            Starknet::declare(none_origin.clone(), transaction.clone().into(), erc721.clone()),
            Error::<MockRuntime>::ClassHashNotWhitelisted
        );
    });
}
