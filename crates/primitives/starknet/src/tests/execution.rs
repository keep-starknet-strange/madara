use blockifier::abi::abi_utils::selector_from_name;
use blockifier::execution::entry_point::CallEntryPoint;
use frame_support::{assert_ok, bounded_vec};
use sp_core::{H256, U256};
use starknet_api::serde_utils::bytes_from_hex_str;

use crate::block::Block;
use crate::execution::call_entrypoint_wrapper::CallEntryPointWrapper;
use crate::execution::entrypoint_wrapper::EntryPointTypeWrapper;
use crate::execution::types::ContractAddressWrapper;
use crate::tests::utils::{create_test_state, TEST_CLASS_HASH, TEST_CONTRACT_ADDRESS};

#[test]
fn test_call_entry_point_execute_works() {
    let mut test_state = create_test_state();

    let class_hash = bytes_from_hex_str::<32, true>(TEST_CLASS_HASH).unwrap();
    let address = bytes_from_hex_str::<32, true>(TEST_CONTRACT_ADDRESS).unwrap();
    let selector = H256::from_slice(selector_from_name("return_result").0.bytes());
    let calldata = bounded_vec![U256::from(42)];

    let entrypoint = CallEntryPointWrapper::new(
        Some(class_hash),
        EntryPointTypeWrapper::External,
        Some(selector),
        calldata,
        address,
        ContractAddressWrapper::default(),
    );

    let block = Block::create_for_testing();

    assert_ok!(entrypoint.execute(&mut test_state, block, [0; 32]));
}

#[test]
fn test_call_entry_point_execute_fails_undeclared_class_hash() {
    let mut test_state = create_test_state();

    let address = bytes_from_hex_str::<32, true>(TEST_CONTRACT_ADDRESS).unwrap();
    let selector = H256::from_slice(selector_from_name("return_result").0.bytes());
    let calldata = bounded_vec![U256::from(42)];

    let entrypoint = CallEntryPointWrapper::new(
        Some([0; 32]),
        EntryPointTypeWrapper::External,
        Some(selector),
        calldata,
        address,
        ContractAddressWrapper::default(),
    );

    let block = Block::create_for_testing();

    assert!(entrypoint.execute(&mut test_state, block, [0; 32]).is_err());
}

#[test]
fn test_try_into_entrypoint_default() {
    let entrypoint_wrapper = CallEntryPointWrapper::default();
    let entrypoint: CallEntryPoint = entrypoint_wrapper.try_into().unwrap();
    pretty_assertions::assert_eq!(entrypoint, CallEntryPoint::default());
}

#[test]
fn test_try_into_entrypoint_fails() {
    let entrypoint_wrapper = CallEntryPointWrapper {
        class_hash: None,
        entrypoint_type: EntryPointTypeWrapper::External,
        entrypoint_selector: None,
        calldata: bounded_vec![],
        storage_address: [u8::MAX; 32], // Bigger than felt
        caller_address: ContractAddressWrapper::default(),
    };
    let entrypoint: Result<CallEntryPoint, _> = entrypoint_wrapper.try_into();
    assert!(entrypoint.is_err());

    let entrypoint_wrapper = CallEntryPointWrapper {
        class_hash: None,
        entrypoint_type: EntryPointTypeWrapper::External,
        entrypoint_selector: None,
        calldata: bounded_vec![],
        storage_address: ContractAddressWrapper::default(),
        caller_address: [u8::MAX; 32], // Bigger than felt
    };
    let entrypoint: Result<CallEntryPoint, _> = entrypoint_wrapper.try_into();
    assert!(entrypoint.is_err());

    let entrypoint_wrapper = CallEntryPointWrapper {
        class_hash: None,
        entrypoint_type: EntryPointTypeWrapper::External,
        entrypoint_selector: Some(H256::from([u8::MAX; 32])), // Bigger than felt
        calldata: bounded_vec![],
        storage_address: ContractAddressWrapper::default(),
        caller_address: ContractAddressWrapper::default(),
    };
    let entrypoint: Result<CallEntryPoint, _> = entrypoint_wrapper.try_into();
    assert!(entrypoint.is_err());
}
