use mp_starknet::execution::types::Felt252Wrapper;

use super::mock::{default_mock, state_root_mock, *};

#[test]
fn given_default_runtime_with_state_root_disabled_default_value_is_correct() {
    new_test_ext::<default_mock::MockRuntime>().execute_with(|| {
        default_mock::basic_test_setup(2);

        // Check that state root is not set when disabled
        pretty_assertions::assert_eq!(
            default_mock::Starknet::compute_and_store_state_root(),
            Felt252Wrapper::default()
        );
    });
}

#[test]
fn given_default_runtime_with_state_root_enabled_default_value_is_correct() {
    new_test_ext::<state_root_mock::MockRuntime>().execute_with(|| {
        state_root_mock::basic_test_setup(2);

        // Check the default state root value when enabled
        // We fetch this value using current genesis state and starkware python package
        pretty_assertions::assert_eq!(
            state_root_mock::Starknet::compute_and_store_state_root(),
            Felt252Wrapper::from_hex_be("0x066a4b57d6d9f0c3d15f1bebfac552d0e0e39ca89a1627b190838344620ecbe1").unwrap()
        );

        let account_address = get_account_address(AccountType::V0(AccountTypeV0Inner::Argent));

        pretty_assertions::assert_eq!(
            state_root_mock::Starknet::contract_state_root_by_address(account_address).unwrap(),
            Felt252Wrapper::from_hex_be("0x04b9de03767569b7b86924fd58d86cb1a0ba1b9c3eb3078187b4533d0d2af340").unwrap()
        )
    });
}
