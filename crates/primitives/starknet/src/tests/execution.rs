use blockifier::abi::abi_utils::selector_from_name;
use blockifier::test_utils::{create_test_state, TEST_CLASS_HASH, TEST_CONTRACT_ADDRESS};
use frame_support::{assert_ok, bounded_vec};
use sp_core::{H256, U256};
use starknet_api::serde_utils::bytes_from_hex_str;

use crate::block::Block;
use crate::execution::{CallEntryPointWrapper, ContractAddressWrapper, EntryPointTypeWrapper};

#[test]
fn test_call_entry_point_execute_works() {
    let mut test_state = create_test_state();

    let class_hash = bytes_from_hex_str::<32, true>(TEST_CLASS_HASH).unwrap();
    let address = bytes_from_hex_str::<32, true>(TEST_CONTRACT_ADDRESS).unwrap();
    let selector = H256::from_slice(&selector_from_name("return_result").0.bytes());
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
