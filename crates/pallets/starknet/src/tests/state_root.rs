use mp_starknet::execution::types::Felt252Wrapper;

use super::mock::state_root_mock::{basic_test_setup_state_root, new_test_ext_with_state_root};
use super::mock::*;
use crate::tests::mock::state_root_mock::MockStateRootRuntime;

#[test]
fn given_default_runtime_with_state_root_disabled_default_value_is_correct() {
    new_test_ext().execute_with(|| {
        basic_test_setup_state_root::<MockRuntime>(2);

        // Check that state root is not set when disabled
        pretty_assertions::assert_eq!(Starknet::compute_and_store_state_root(), Felt252Wrapper::default());
    });
}

#[test]
fn given_default_runtime_with_state_root_enabled_default_value_is_correct() {
    new_test_ext_with_state_root().execute_with(|| {
        basic_test_setup_state_root::<MockStateRootRuntime>(2);

        // Check the default state root value when enabled
        // We fetch this value using current genesis state and starkware python package
        pretty_assertions::assert_eq!(
            Starknet::compute_and_store_state_root(),
            Felt252Wrapper::from_hex_be("0x06119516e37ae3039860d7ef27197a9c78ea9601ec147e3c2e2b0b65ad3bfe9d").unwrap()
        );
    });
}
