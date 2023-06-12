use std::iter::Skip;
use std::vec::IntoIter;

use mp_starknet::execution::types::Felt252Wrapper;
use mp_starknet::transaction::types::EventWrapper;
use starknet_ff::FieldElement;

/// Helper function to get filter events using address and keys

/// # Arguments
///
/// * `events` - A vector of all events
/// * `address` - Address to use to filter the events
/// * `keys` - Keys to use to filter the events. An event is filtered if any key is present
/// * `max_results` - Optional, indicated the max events that need to be filtered
///
/// # Returns
///
/// * `(block_events: Vec<EventWrapper>, continuation_token: usize)` - A tuple of the filtered
///   events and the first index which still hasn't been processed block_id and an instance of Block
pub fn filter_events_by_params(
    events: Skip<IntoIter<EventWrapper>>,
    address: Option<Felt252Wrapper>,
    keys: Vec<Vec<FieldElement>>,
    max_results: Option<usize>,
) -> (Vec<EventWrapper>, usize) {
    let mut filtered_events = vec![];
    let mut index = 0;

    // Iterate on block events.
    for event in events {
        index += 1;
        let match_from_address = address.map_or(true, |addr| addr == event.from_address);
        // Based on https://github.com/starkware-libs/papyrus
        let match_keys = keys
            .iter()
            .enumerate()
            .all(|(i, keys)| event.keys.len() > i && (keys.is_empty() || keys.contains(&event.keys[i].into())));

        if match_from_address && match_keys {
            filtered_events.push(event);
            if let Some(max_results) = max_results {
                if filtered_events.len() >= max_results {
                    break;
                }
            }
        }
    }
    (filtered_events, index)
}
