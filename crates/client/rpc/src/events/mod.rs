#[cfg(test)]
mod tests;

use std::cmp::Ordering;
use std::iter::Skip;
use std::vec::IntoIter;

use jsonrpsee::core::RpcResult;
use log::error;
use mc_rpc_core::utils::get_block_by_block_hash;
use mc_transaction_pool::ChainApi;
use mp_starknet::block::Block;
use mp_starknet::execution::types::Felt252Wrapper;
use mp_starknet::traits::hash::HasherT;
use mp_starknet::traits::ThreadSafeCopy;
use mp_starknet::transaction::types::{EventWrapper, TransactionReceiptWrapper};
use pallet_starknet::runtime_api::{ConvertTransactionRuntimeApi, StarknetRuntimeApi};
use sc_client_api::backend::{Backend, StorageProvider};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use starknet_core::types::{BlockId, EventsPage};
use starknet_ff::FieldElement;

use crate::errors::StarknetRpcApiError;
use crate::types::RpcEventFilter;
use crate::{EmittedEvent, Starknet};

impl<A: ChainApi, B, BE, C, P, H> Starknet<A, B, BE, C, P, H>
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
    /// * `(transaction_receipts: Vec<TransactionReceiptWrapper>, block: Block)` - A tuple of the
    ///   block transaction receipts with events in block_id and an instance of Block
    pub fn get_block_receipts(
        &self,
        block_id: u64,
    ) -> Result<(Vec<TransactionReceiptWrapper>, Block), StarknetRpcApiError> {
        let substrate_block_hash =
            self.substrate_block_hash_from_starknet_block(BlockId::Number(block_id)).map_err(|e| {
                error!("'{e}'");
                StarknetRpcApiError::BlockNotFound
            })?;

        let block = get_block_by_block_hash(self.client.as_ref(), substrate_block_hash).ok_or_else(|| {
            error!("Failed to retrieve block");
            StarknetRpcApiError::BlockNotFound
        })?;

        let transaction_receipts = block.transaction_receipts().to_owned().into();

        Ok((transaction_receipts, block))
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
        let mut filtered_events = vec![];

        // get filter values
        let mut current_block = filter.from_block;
        let to_block = filter.to_block;
        let from_address = filter.from_address;
        let keys = filter.keys;
        let mut continuation_token = filter.continuation_token;
        let chunk_size = filter.chunk_size;

        // skip blocks with continuation token block number
        current_block += continuation_token.block_n;

        // Iterate on block range
        while current_block <= to_block {
            let (trx_receipts, block) = self.get_block_receipts(current_block)?;
            // check if continuation_token.receipt_n correct
            if (trx_receipts.len() as u64) < continuation_token.receipt_n {
                return Err(StarknetRpcApiError::InvalidContinuationToken.into());
            }

            let block_hash = block.header().hash(*self.hasher).into();
            let block_number = block.header().block_number;

            // skip transaction receipts
            for receipt in trx_receipts.iter().skip(continuation_token.receipt_n as usize) {
                let receipt_events_len: usize = receipt.events.len();
                // check if continuation_token.event_n is correct
                match (receipt_events_len as u64).cmp(&continuation_token.event_n) {
                    Ordering::Greater => (),
                    Ordering::Less => return Err(StarknetRpcApiError::InvalidContinuationToken.into()),
                    Ordering::Equal => {
                        continuation_token.receipt_n += 1;
                        continuation_token.event_n = 0;
                        continue;
                    }
                }

                let receipt_transaction_hash = receipt.transaction_hash;
                // skip events
                let receipt_events = receipt.events.clone().into_iter().skip(continuation_token.event_n as usize);

                let (new_filtered_events, continuation_index) = filter_events_by_params(
                    receipt_events,
                    from_address,
                    keys.clone(),
                    Some((chunk_size as usize) - filtered_events.len()),
                );

                filtered_events.extend(
                    new_filtered_events
                        .iter()
                        .map(|event| EmittedEvent {
                            from_address: event.from_address.into(),
                            keys: event.keys.clone().into_iter().map(|key| key.into()).collect(),
                            data: event.data.clone().into_iter().map(|data| data.into()).collect(),
                            block_hash,
                            block_number,
                            transaction_hash: receipt_transaction_hash.into(),
                        })
                        .collect::<Vec<EmittedEvent>>(),
                );

                if filtered_events.len() >= chunk_size as usize {
                    let token = if current_block < to_block
                        || continuation_token.receipt_n < trx_receipts.len() as u64 - 1
                        || continuation_index < receipt_events_len
                    {
                        continuation_token.event_n = continuation_index as u64;
                        Some(continuation_token.to_string())
                    } else {
                        None
                    };
                    return Ok(EventsPage { events: filtered_events, continuation_token: token });
                }

                continuation_token.receipt_n += 1;
                continuation_token.event_n = 0;
            }

            current_block += 1;
            continuation_token.block_n += 1;
            continuation_token.receipt_n = 0;
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
