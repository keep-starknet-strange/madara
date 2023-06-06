use alloc::sync::Arc;

use blockifier::abi::abi_utils::selector_from_name;
use blockifier::execution::entry_point::{CallEntryPoint, CallType};
use frame_support::{assert_ok, bounded_vec};
use starknet_api::api_core::{ChainId, ClassHash, ContractAddress, EntryPointSelector, PatriciaKey};
use starknet_api::deprecated_contract_class::EntryPointType;
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::transaction::Calldata;
use starknet_api::{patricia_key, stark_felt};

use crate::block::Block;
use crate::execution::call_entrypoint_wrapper::CallEntryPointWrapper;
use crate::execution::entrypoint_wrapper::EntryPointTypeWrapper;
use crate::execution::types::{ContractAddressWrapper, Felt252Wrapper};
use crate::tests::utils::{create_test_state, TEST_CLASS_HASH, TEST_CONTRACT_ADDRESS};

#[test]
fn test_call_entry_point_execute_works() {
    let mut test_state = create_test_state();

    let class_hash = Felt252Wrapper::from_hex_be(TEST_CLASS_HASH).unwrap();
    let address = Felt252Wrapper::from_hex_be(TEST_CONTRACT_ADDRESS).unwrap();
    let selector = selector_from_name("return_result").0.into();
    let calldata = bounded_vec![42_u128.into()];

    let chain_id = ChainId("0x1".to_string());

    let entrypoint = CallEntryPointWrapper::new(
        Some(class_hash),
        EntryPointTypeWrapper::External,
        Some(selector),
        calldata,
        address,
        ContractAddressWrapper::default(),
    );

    let block = Block::create_for_testing();

    assert_ok!(entrypoint.execute(&mut test_state, block, Felt252Wrapper::ZERO, chain_id));
}

#[test]
fn test_call_entry_point_execute_fails_undeclared_class_hash() {
    let mut test_state = create_test_state();

    let address = Felt252Wrapper::from_hex_be(TEST_CONTRACT_ADDRESS).unwrap();
    let selector = selector_from_name("return_result").0.into();
    let calldata = bounded_vec![42_u128.into()];

    let entrypoint = CallEntryPointWrapper::new(
        Some(Felt252Wrapper::ZERO),
        EntryPointTypeWrapper::External,
        Some(selector),
        calldata,
        address,
        ContractAddressWrapper::default(),
    );

    let block = Block::create_for_testing();

    let chain_id = ChainId("0x1".to_string());

    assert!(entrypoint.execute(&mut test_state, block, Felt252Wrapper::ZERO, chain_id).is_err());
}

#[test]
fn test_try_into_entrypoint_default() {
    let entrypoint_wrapper = CallEntryPointWrapper::default();
    let entrypoint: CallEntryPoint = entrypoint_wrapper.try_into().unwrap();
    pretty_assertions::assert_eq!(entrypoint, CallEntryPoint::default());
}

#[test]
fn test_try_into_entrypoint_works() {
    let entrypoint_wrapper = CallEntryPointWrapper {
        class_hash: Some(Felt252Wrapper::from_hex_be("0x1").unwrap()),
        entrypoint_type: EntryPointTypeWrapper::External,
        entrypoint_selector: None,
        calldata: bounded_vec![Felt252Wrapper::ONE, Felt252Wrapper::TWO, Felt252Wrapper::THREE],
        storage_address: Felt252Wrapper::from_hex_be("0x1").unwrap(),
        caller_address: Felt252Wrapper::from_hex_be("0x2").unwrap(),
    };
    let entrypoint: CallEntryPoint = entrypoint_wrapper.try_into().unwrap();
    let expected_entrypoint = CallEntryPoint {
        call_type: CallType::Call,
        calldata: Calldata(Arc::new(vec![stark_felt!(1), stark_felt!(2), stark_felt!(3)])),
        caller_address: ContractAddress(patricia_key!(2)),
        storage_address: ContractAddress(patricia_key!(1)),
        class_hash: Some(ClassHash(stark_felt!(1))),
        code_address: None,
        entry_point_selector: EntryPointSelector(stark_felt!(0)),
        entry_point_type: EntryPointType::External,
    };

    pretty_assertions::assert_eq!(entrypoint, expected_entrypoint);
}

// #[test]
// #[ignore = "flemme"]
// fn test_contract_class_wrapper_try_from_contract_class() {
//     let json_content: &str = r#"
// 	{
// 	"entry_points_by_type": {
// 		"CONSTRUCTOR": [
// 			{
// 				"offset": "0x147",
// 				"selector": "0x28ffe4ff0f226a9107253e17a904099aa4f63a02a5621de0576e5aa71bc5194"
// 			}
// 		],
// 		"EXTERNAL": [
// 			{
// 				"offset": "0x16e",
// 				"selector": "0x966af5d72d3975f70858b044c77785d3710638bbcebbd33cc7001a91025588"
// 			}
// 		],
// 		"L1_HANDLER": []
// 	},
// 	"program": {
// 		"main_scope": "__main__",
//     "reference_manager": {
//       "references": []
//     },
// 		"builtins": [],
// 		"debug_info": null,
// 		"hints": {},
// 		"compiler_version": "0.10.3",
// 		"prime": "0x800000000000011000000000000000000000000000000000000000000000001",
//     "identifiers": {},
//     "data": [],
//     "attributes": []
// 	}
// }"#;
//     let contract_class: ContractClass = serde_json::from_str(json_content).unwrap();
//     let contract_class_wrapper: ContractClassWrapper = contract_class.try_into().unwrap();

//     let mut entrypoints = BTreeMap::new();
//     let iter: Vec<(EntryPointTypeWrapper, bounded_vec::BoundedVec<EntryPointWrapper,
// sp_core::ConstU32<4294967295>>)> = vec![         (
//             EntryPointTypeWrapper::Constructor,
//             bounded_vec![EntryPointWrapper {
//                 offset: 0x147,
//                 selector: Felt252Wrapper::from_hex_be(
//                     "0x028ffe4ff0f226a9107253e17a904099aa4f63a02a5621de0576e5aa71bc5194"
//                 )
//                 .unwrap()
//                 .into(),
//             }],
//         ),
//         (
//             EntryPointTypeWrapper::External,
//             bounded_vec![EntryPointWrapper {
//                 offset: 0x16e,
//                 selector: Felt252Wrapper::from_hex_be(
//                     "0x00966af5d72d3975f70858b044c77785d3710638bbcebbd33cc7001a91025588"
//                 )
//                 .unwrap()
//                 .into(),
//             }],
//         ),
//         (EntryPointTypeWrapper::L1Handler, bounded_vec![]),
//     ];

//     for (entrypoint_type, entrypoint_wrappers) in iter.iter() {
//         entrypoints.insert(entrypoint_type.clone(), entrypoint_wrappers.clone());
//     }

//     let expected_contract_class_wrapper = ContractClassWrapper {
//         entry_points_by_type: BoundedBTreeMap::try_from(entrypoints).unwrap(),
//         program: ProgramWrapper::default(),
//     };

//     pretty_assertions::assert_eq!(contract_class_wrapper, expected_contract_class_wrapper);
// }
