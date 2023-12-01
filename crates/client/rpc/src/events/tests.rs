use mp_felt::Felt252Wrapper;
use rstest::*;
use starknet_core::types::EmittedEvent;
use starknet_ff::FieldElement;

use crate::events::filter_events_by_params;

#[derive(Debug, Clone)]
struct TestCase<'a> {
    _name: &'a str,
    events: Vec<EmittedEvent>,
    filter_keys: Vec<Vec<FieldElement>>,
    filter_address: Option<Felt252Wrapper>,
    max_results: usize,
    expected_events: Vec<EmittedEvent>,
    n_visited: usize,
}

#[fixture]
#[once]
fn build_test_case() -> Vec<TestCase<'static>> {
    let event1 = build_event_wrapper_for_test(&["0x1", "0x2", "0x3"], 1);
    let event2 = build_event_wrapper_for_test(&["0x1", "", "0x3"], 2);
    let event3 = build_event_wrapper_for_test(&["0x2", "", "0x3"], 3);
    let event4 = build_event_wrapper_for_test(&["0x1"], 4);
    let event5 = build_event_wrapper_for_test(&["0x1", "", "0x3"], 3);

    let events = vec![event1.clone(), event2.clone(), event3, event4.clone(), event5.clone()];
    vec![
        TestCase {
            _name: "filter events by keys",
            events: events.clone(),
            filter_keys: vec![vec![FieldElement::from(1_u32)], vec![], vec![FieldElement::from(3_u32)]],
            filter_address: None,
            max_results: 100,
            expected_events: vec![event1.clone(), event2.clone(), event5.clone()],
            n_visited: 5,
        },
        TestCase {
            _name: "filter events by address",
            events: events.clone(),
            filter_keys: vec![],
            filter_address: Some(Felt252Wrapper::from_dec_str("2").unwrap()),
            max_results: 100,
            expected_events: vec![event2.clone()],
            n_visited: 5,
        },
        TestCase {
            _name: "filter events by address and keys",
            events: events.clone(),
            filter_keys: vec![vec![FieldElement::from(1_u32)], vec![]],
            filter_address: Some(Felt252Wrapper::from_dec_str("3").unwrap()),
            max_results: 100,
            expected_events: vec![event5.clone()],
            n_visited: 5,
        },
        TestCase {
            _name: "filter events by max results where max results is met",
            events: events.clone(),
            filter_keys: vec![vec![FieldElement::from(1_u32)], vec![]],
            filter_address: None,
            max_results: 1,
            expected_events: vec![event1.clone()],
            n_visited: 1,
        },
        TestCase {
            _name: "filter events by max results where max results is not met",
            events: events.clone(),
            filter_keys: vec![vec![FieldElement::from(10_u32)], vec![]],
            filter_address: None,
            max_results: 1,
            expected_events: vec![],
            n_visited: 5,
        },
        TestCase {
            _name: "filter events where filter_keys.len() < event.keys.len()",
            events: events.clone(),
            filter_keys: vec![vec![FieldElement::from(1_u32)]],
            filter_address: None,
            max_results: 100,
            expected_events: vec![event1, event2, event4, event5],
            n_visited: 5,
        },
        TestCase {
            _name: "filter events where filter_keys.len() > event.keys.len()",
            events: events.clone(),
            filter_keys: vec![vec![FieldElement::from(1_u32)], vec![], vec![], vec![]],
            filter_address: None,
            max_results: 100,
            expected_events: vec![],
            n_visited: 5,
        },
        TestCase {
            _name: "filter events without any filters",
            events: events.clone(),
            filter_keys: vec![],
            filter_address: None,
            max_results: 100,
            expected_events: events,
            n_visited: 5,
        },
        TestCase {
            _name: "filter events without any events",
            events: vec![],
            filter_keys: vec![vec![FieldElement::from(1_u32)], vec![], vec![], vec![]],
            filter_address: None,
            max_results: 100,
            expected_events: vec![],
            n_visited: 0,
        },
    ]
}

#[rstest]
#[case::filter_keys(build_test_case()[0].clone())]
#[case::filter_address(build_test_case()[1].clone())]
#[case::filters_keys_and_address(build_test_case()[2].clone())]
#[case::filter_max_results_met(build_test_case()[3].clone())]
#[case::filter_max_results_not_met(build_test_case()[4].clone())]
#[case::filter_keys_less_than_actual(build_test_case()[5].clone())]
#[case::filter_keys_more_than_actual(build_test_case()[6].clone())]
#[case::filter_with_no_filters(build_test_case()[7].clone())]
#[case::filter_with_no_events(build_test_case()[8].clone())]
fn filter_events_by_test_case(#[case] params: TestCase) {
    let mut n_visited = 0;
    #[allow(clippy::iter_skip_zero)]
    let filtered_events = filter_events_by_params(
        params.events.into_iter().skip(0),
        params.filter_address,
        &params.filter_keys,
        params.max_results,
        &mut n_visited,
    );
    assert_eq!(filtered_events, params.expected_events);
    pretty_assertions::assert_eq!(n_visited, params.n_visited);
}

fn build_event_wrapper_for_test(keys: &[&str], address_int: u64) -> EmittedEvent {
    let keys = keys.iter().map(|key| FieldElement::from_hex_be(key).unwrap()).collect::<Vec<_>>();

    EmittedEvent {
        from_address: FieldElement::from(address_int),
        keys,
        data: vec![],
        block_hash: Default::default(),
        block_number: Default::default(),
        transaction_hash: Default::default(),
    }
}
