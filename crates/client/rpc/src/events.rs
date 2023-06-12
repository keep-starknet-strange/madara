use std::iter::Skip;
use std::vec::IntoIter;

use log::error;
use mc_rpc_core::utils::get_block_by_block_hash;
use mp_starknet::block::Block;
use mp_starknet::execution::types::Felt252Wrapper;
use mp_starknet::traits::hash::HasherT;
use mp_starknet::traits::ThreadSafeCopy;
use mp_starknet::transaction::types::EventWrapper;
use pallet_starknet::runtime_api::{ConvertTransactionRuntimeApi, StarknetRuntimeApi};
use sc_client_api::backend::{Backend, StorageProvider};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use starknet_core::types::BlockId;
use starknet_ff::FieldElement;

use crate::errors::StarknetRpcApiError;
use crate::Starknet;

impl<B, BE, C, P, H> Starknet<B, BE, C, P, H>
where
    B: BlockT,
    C: HeaderBackend<B> + StorageProvider<B, BE> + 'static,
    C: ProvideRuntimeApi<B>,
    C::Api: StarknetRuntimeApi<B> + ConvertTransactionRuntimeApi<B>,
    BE: Backend<B>,
    H: HasherT + ThreadSafeCopy,
{
    /// Helper function to get Starknet block details
    ///
    /// # Arguments
    ///
    /// * `block_id` - The Starknet block id
    ///
    /// # Returns
    ///
    /// * `(block_events: Vec<EventWrapper>, block: Block)` - A tuple of the block events in
    ///   block_id and an instance of Block
    pub fn get_block_events(&self, block_id: u64) -> Result<(Vec<EventWrapper>, Block), StarknetRpcApiError> {
        let substrate_block_hash =
            self.substrate_block_hash_from_starknet_block(BlockId::Number(block_id)).map_err(|e| {
                error!("'{e}'");
                StarknetRpcApiError::BlockNotFound
            })?;

        let block = get_block_by_block_hash(self.client.as_ref(), substrate_block_hash).ok_or_else(|| {
            error!("Failed to retrieve block");
            StarknetRpcApiError::BlockNotFound
        })?;
        let block_events = self
            .overrides
            .for_block_hash(self.client.as_ref(), substrate_block_hash)
            .events(substrate_block_hash)
            .unwrap_or_else(|| {
                dbg!("No events found in block {}", block_id);
                Vec::new()
            });
        Ok((block_events, block))
    }
}

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
