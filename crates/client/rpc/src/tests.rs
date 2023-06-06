// use mp_starknet::execution::types::{ContractAddressWrapper, Felt252Wrapper};
// use mp_starknet::transaction::types::EventWrapper;
// use sp_core::bounded_vec;
// use sp_runtime::BoundedVec;
//
// use crate::Starknet;
//
// // write a test case to test filter_events_by_params in lib.rs file
// #[test]
// fn filter_events_by_params_test() {
//     let event1 = build_event_wrapper_for_test(&["0x1"], 1);
//     let event2 = build_event_wrapper_for_test(&["0x2"], 1);
//     let event3 = build_event_wrapper_for_test(&["0x3"], 1);
//     let event4 = build_event_wrapper_for_test(&["0x4"], 1);
//
//     let events = vec![event1, event2, event3, event4];
//
//     // call the filter_events_by_params method on Starknet struct
//
//     let x = Starknet::filter_events_by_params(
//         events,
//         None,
//         vec![vec![Felt252Wrapper::from_hex_be("0x1").unwrap().into()]],
//         None,
//     );
// }
//
// fn build_event_wrapper_for_test(keys: &[&str], address_int: u64) -> EventWrapper {
//     let keys_felt = keys.iter().map(|key|
// Felt252Wrapper::from_hex_be(key).unwrap()).collect::<Vec<Felt252Wrapper>>();     EventWrapper {
//         keys: BoundedVec::try_from(keys_felt).unwrap(),
//         data: bounded_vec!(),
//         from_address: ContractAddressWrapper::from(address_int),
//         transaction_hash: Felt252Wrapper::from(1_u64),
//     }
// }

#[test]
fn sample_test() {
    assert_eq!(1, 1);
}
