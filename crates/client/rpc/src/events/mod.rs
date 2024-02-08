#[cfg(test)]
mod tests;

use std::iter::Skip;
use std::vec::IntoIter;

use jsonrpsee::core::RpcResult;
use log::error;
use mc_rpc_core::utils::get_block_by_block_hash;
use mp_felt::Felt252Wrapper;
use mp_hashers::HasherT;
use pallet_starknet_runtime_api::{ConvertTransactionRuntimeApi, StarknetRuntimeApi};
use sc_client_api::backend::{Backend, StorageProvider};
use sc_client_api::BlockBackend;
use sc_transaction_pool::ChainApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use starknet_core::types::{BlockId, EmittedEvent, EventsPage};
use starknet_ff::FieldElement;

use crate::errors::StarknetRpcApiError;
use crate::types::{ContinuationToken, RpcEventFilter};
use crate::Starknet;

impl<A: ChainApi, B, BE, G, C, P, H> Starknet<A, B, BE, G, C, P, H>
where
    B: BlockT,
    C: HeaderBackend<B> + BlockBackend<B> + StorageProvider<B, BE> + 'static,
    C: ProvideRuntimeApi<B>,
    C::Api: StarknetRuntimeApi<B> + ConvertTransactionRuntimeApi<B>,
    BE: Backend<B>,
    H: HasherT + Send + Sync + 'static,
{
    /// Helper function to get Starknet block details
    ///
    /// # Arguments
    ///
    /// * `block_id` - The Starknet block id
    ///
    /// # Returns
    ///
    /// * `(transaction_receipts: Vec<TransactionReceiptWrapper>, block: Block)` - A tuple of the
    ///   block transaction receipts with events in block_id and an instance of Block
    pub fn get_block_events(&self, block_number: u64) -> Result<Vec<EmittedEvent>, StarknetRpcApiError> {
        let substrate_block_hash =
            self.substrate_block_hash_from_starknet_block(BlockId::Number(block_number)).map_err(|e| {
                error!("'{e}'");
                StarknetRpcApiError::BlockNotFound
            })?;

        let block_extrinsics = self
            .client
            .block_body(substrate_block_hash)
            .map_err(|e| {
                error!("Failed to retrieve block body. Substrate block hash: {substrate_block_hash}, error: {e}");
                StarknetRpcApiError::InternalServerError
            })?
            .ok_or(StarknetRpcApiError::BlockNotFound)?;

        let chain_id = self.client.runtime_api().chain_id(substrate_block_hash).map_err(|_| {
            error!("Failed to retrieve chain id");
            StarknetRpcApiError::InternalServerError
        })?;

        let index_and_events =
            self.client.runtime_api().get_starknet_events_and_their_associated_tx_index(substrate_block_hash).map_err(
                |e| {
                    error!(
                        "Failed to retrieve starknet events and their associated transaction index. Substrate block \
                         hash: {substrate_block_hash}, chain ID: {chain_id:?}, error: {e}"
                    );
                    StarknetRpcApiError::InternalServerError
                },
            )?;

        let starknet_block = get_block_by_block_hash(self.client.as_ref(), substrate_block_hash).map_err(|e| {
            error!("Failed to retrieve starknet block from substrate block hash: error: {e}");
            StarknetRpcApiError::InternalServerError
        })?;
        let txn_hashes = self.get_cached_transaction_hashes(starknet_block.header().hash::<H>().into());
        let block_extrinsics_len = block_extrinsics.len();
        let starknet_txs =
            self.client.runtime_api().extrinsic_filter(substrate_block_hash, block_extrinsics).map_err(|e| {
                error!("Failed to filter extrinsics. Substrate block hash: {substrate_block_hash}, error: {e}");
                StarknetRpcApiError::InternalServerError
            })?;
        let inherent_count = block_extrinsics_len - starknet_txs.len();
        let mut tx_hash_and_events: Vec<(Felt252Wrapper, starknet_api::transaction::Event)> = vec![];
        for (index, event) in index_and_events {
            let tx_index = index as usize - inherent_count;
            let tx_hash = self.try_txn_hash_from_cache(tx_index, &txn_hashes, &starknet_txs, chain_id)?;
            tx_hash_and_events.push((tx_hash, event));
        }

        let starknet_block = match get_block_by_block_hash(self.client.as_ref(), substrate_block_hash) {
            Ok(block) => block,
            Err(_) => return Err(StarknetRpcApiError::BlockNotFound),
        };

        let block_hash = starknet_block.header().hash::<H>();

        let emitted_events = tx_hash_and_events
            .into_iter()
            .map(|(tx_hash, event)| EmittedEvent {
                from_address: Felt252Wrapper::from(event.from_address).0,
                keys: event.content.keys.into_iter().map(|felt| Felt252Wrapper::from(felt).0).collect(),
                data: event.content.data.0.into_iter().map(|felt| Felt252Wrapper::from(felt).0).collect(),
                block_hash: block_hash.0,
                block_number,
                transaction_hash: tx_hash.0,
            })
            .collect();

        Ok(emitted_events)
    }

    /// Helper function to filter Starknet events provided a RPC event filter
    ///
    /// # Arguments
    ///
    /// * `filter` - The RPC event filter
    ///
    /// # Returns
    ///
    /// * `EventsPage` - The filtered events with continuation token
    pub fn filter_events(&self, filter: RpcEventFilter) -> RpcResult<EventsPage> {
        // get filter values
        let continuation_token = filter.continuation_token;
        // skip blocks with continuation token block number
        let from_block = filter.from_block + continuation_token.block_n;
        let mut current_block = from_block;
        let to_block = filter.to_block;
        let from_address = filter.from_address;
        let keys = filter.keys;
        let chunk_size = filter.chunk_size;

        let mut filtered_events = Vec::new();

        // Iterate on block range
        while current_block <= to_block {
            let emitted_events = self.get_block_events(current_block)?;
            let mut unchecked_events = emitted_events.len();
            let events = if current_block == from_block {
                // check if continuation_token.event_n is not too big
                if (unchecked_events as u64) < continuation_token.event_n {
                    return Err(StarknetRpcApiError::InvalidContinuationToken.into());
                }
                unchecked_events -= continuation_token.event_n as usize;
                emitted_events.into_iter().skip(continuation_token.event_n as usize)
            } else {
                #[allow(clippy::iter_skip_zero)]
                emitted_events.into_iter().skip(0)
            };

            let mut n_visited = 0;
            let block_filtered_events = filter_events_by_params(
                events,
                from_address,
                &keys,
                chunk_size as usize - filtered_events.len(),
                &mut n_visited,
            );

            filtered_events.extend(block_filtered_events);

            if filtered_events.len() == chunk_size as usize {
                let token = if current_block < to_block || n_visited < unchecked_events {
                    let mut event_n = n_visited as u64;
                    if continuation_token.block_n == current_block {
                        event_n += continuation_token.event_n;
                    }
                    Some(ContinuationToken { block_n: current_block - from_block, event_n }.to_string())
                } else {
                    None
                };

                return Ok(EventsPage { events: filtered_events, continuation_token: token });
            }

            current_block += 1;
        }

        Ok(EventsPage { events: filtered_events, continuation_token: None })
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
pub fn filter_events_by_params<'a, 'b: 'a>(
    events: Skip<IntoIter<EmittedEvent>>,
    address: Option<Felt252Wrapper>,
    keys: &'a [Vec<FieldElement>],
    max_results: usize,
    n_visited: &'b mut usize,
) -> Vec<EmittedEvent> {
    let mut filtered_events = vec![];

    // Iterate on block events.
    for event in events {
        *n_visited += 1;
        let match_from_address = address.map_or(true, |addr| addr.0 == event.from_address);
        // Based on https://github.com/starkware-libs/papyrus
        let match_keys = keys
            .iter()
            .enumerate()
            .all(|(i, keys)| event.keys.len() > i && (keys.is_empty() || keys.contains(&event.keys[i])));

        if match_from_address && match_keys {
            filtered_events.push(event);
            if filtered_events.len() >= max_results {
                break;
            }
        }
    }
    filtered_events
}
