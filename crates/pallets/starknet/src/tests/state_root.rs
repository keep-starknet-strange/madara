use mp_starknet::execution::types::Felt252Wrapper;

use super::mock::state_root_mock::{basic_test_setup_state_root, new_test_ext_with_state_root};
use super::mock::*;
use crate::tests::mock::state_root_mock::MockStateRootRuntime;

#[test]
fn given_default_runtime_with_state_root_disabled_default_value_is_correct() {
    new_test_ext().execute_with(|| {
        basic_test_setup_state_root::<MockRuntime>(2);
        let commitments = Starknet::current_state_commitments();

        // Check the default state root value
        pretty_assertions::assert_eq!(Starknet::compute_and_store_state_root(), Felt252Wrapper::default());
    });
}

#[test]
fn given_default_runtime_with_state_root_enabled_default_value_is_correct() {
    new_test_ext_with_state_root().execute_with(|| {
        basic_test_setup_state_root::<MockStateRootRuntime>(2);
        let commitments = Starknet::current_state_commitments();

        // Check the default state root value
        pretty_assertions::assert_eq!(
            Starknet::compute_and_store_state_root(),
            Felt252Wrapper::from_hex_be("0x0207137ad3e6ce7fdefc04e8e07e8590a310362629f00aacd486dd57378847b4").unwrap()
        );
    });
}
