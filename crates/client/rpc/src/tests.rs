use mp_starknet::execution::types::{ContractAddressWrapper, Felt252Wrapper};
use mp_starknet::transaction::types::EventWrapper;
use sp_core::bounded_vec;
use sp_runtime::BoundedVec;
use starknet_ff::FieldElement;

use crate::{constants, filter_events_by_params};

#[test]
fn filter_events_by_keys_no_chunk_size() {
    let filter_keys = vec![vec![FieldElement::from(1_u32)], vec![], vec![FieldElement::from(3_u32)]];

    let event1 = build_event_wrapper_for_test(&["0x1", "0x2", "0x3"], 1);
    let event2 = build_event_wrapper_for_test(&["0x1", "", "0x3"], 2);
    let event3 = build_event_wrapper_for_test(&["0x2", "", "0x3"], 3);
    let event4 = build_event_wrapper_for_test(&["0x1"], 4);

    let events = vec![event1.clone(), event2.clone(), event3.clone(), event4.clone()].into_iter().skip(0);

    let (filtered_events, _) = filter_events_by_params(events, None, filter_keys, None);
    assert_eq!(filtered_events.len(), 2);
    assert_eq!(filtered_events[0], event1);
    assert_eq!(filtered_events[1], event2);
}

#[test]
fn filter_events_by_address_no_chunk_size() {
    // the keys which should be filtered out
    let filter_keys = vec![vec![]];

    let event1 = build_event_wrapper_for_test(&["0x1", "0x2", "0x3"], 1);
    let event2 = build_event_wrapper_for_test(&["0x1", "", "0x3"], 2);
    let event3 = build_event_wrapper_for_test(&["0x2", "", "0x3"], 3);
    let event4 = build_event_wrapper_for_test(&["0x1"], 4);

    let events = vec![event1.clone(), event2.clone(), event3.clone(), event4.clone()].into_iter().skip(0);

    let (filtered_events, _) =
        filter_events_by_params(events, Some(Felt252Wrapper::from_dec_str("3").unwrap()), filter_keys, None);
    assert_eq!(filtered_events.len(), 1);
    assert_eq!(filtered_events[0], event3);
}

#[test]
fn filter_events_by_address_and_keys_no_chunk_size() {
    let filter_keys = vec![vec![FieldElement::from(1_u32)], vec![], vec![FieldElement::from(3_u32)]];

    let event1 = build_event_wrapper_for_test(&["0x1", "0x2", "0x3"], 1);
    let event2 = build_event_wrapper_for_test(&["0x1", "", "0x3"], 2);
    let event3 = build_event_wrapper_for_test(&["0x2", "", "0x3"], 3);
    let event4 = build_event_wrapper_for_test(&["0x1"], 4);
    let event5 = build_event_wrapper_for_test(&["0x1", "", "0x3"], 3);

    let events = vec![event1.clone(), event2.clone(), event3.clone(), event4.clone(), event5.clone()].into_iter().skip(0);

    let (filtered_events, _) =
        filter_events_by_params(events, Some(Felt252Wrapper::from_dec_str("3").unwrap()), filter_keys, None);
    assert_eq!(filtered_events.len(), 1);
    assert_eq!(filtered_events[0], event5);
}

#[test]
fn filter_events_by_keys_and_chunk_size() {
    let filter_keys = vec![vec![FieldElement::from(1_u32)], vec![], vec![FieldElement::from(3_u32)]];

    let event1 = build_event_wrapper_for_test(&["0x1", "0x2", "0x3"], 1);
    let event2 = build_event_wrapper_for_test(&["0x1", "", "0x3"], 2);
    let event3 = build_event_wrapper_for_test(&["0x2", "", "0x3"], 3);
    let event4 = build_event_wrapper_for_test(&["0x1"], 4);

    let events = vec![event1.clone(), event2.clone(), event3.clone(), event4.clone()].into_iter().skip(0);

    let (filtered_events, continuation_token) = filter_events_by_params(events, None, filter_keys, Some(1));
    assert_eq!(filtered_events.len(), 1);
    assert_eq!(filtered_events[0], event1);
    assert_eq!(continuation_token, 1);
}

fn build_event_wrapper_for_test(keys: &[&str], address_int: u64) -> EventWrapper {
    let keys_felt = keys.iter().map(|key| Felt252Wrapper::from_hex_be(key).unwrap()).collect::<Vec<Felt252Wrapper>>();
    EventWrapper {
        keys: BoundedVec::try_from(keys_felt).unwrap(),
        data: bounded_vec!(),
        from_address: ContractAddressWrapper::from(address_int),
        transaction_hash: Felt252Wrapper::from(1_u64),
    }
}
