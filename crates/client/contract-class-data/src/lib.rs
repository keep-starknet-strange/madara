//! A worker to manage additional contact class data
//!
//! When receiving an `add_decalre_transaction` we convert the contract class we receive into a
//! struct that is executable by the blockifier.
//! During this process there is a loss of data, but we need to store this data somewhere in order
//! to rebuild the original struct and answer the `get_class_at` rpc.
//!
//! To make sure no data is missing, we store the extra data in some temporary storage before
//! pushing the tx into the TransactionPool, while creating a watcher to this Substrate extrinsinc.
//! Then we wait for the tx to be executed, and it's block to be finalized. Once it's done, we check
//! that the Starknet tx was successful. If yes, we move it to a more perenial storage, if not we
//! remove it altogether from the temporary storage.
//! The logic described above is splited beetween:
//! - mc_rpc::add_declare_transaction
//! - mc_db::ContractClassDataDb
//! - mc_contract_class_data::run_worker (the present crate)
//!
//! The present worker does the following inside an infinite loop:
//! - Poll the channel for new declare transactions and add them to a queue
//! - Iterate over this queue and poll each individual tx watcher of update in the tx status.
//!  * If the Substrate tx failed in anyway, remove the contact class data.
//!  * Else if it has been finalize check if the Substrate block has been synced in our backend.
//!   + If it is not sync yet, moved it to another buffer, that will be checked again on each
//!     iteration of the loop.
//!   + Else, check if the Starknet transaction was successfully included in the Starknet block.
//!     > If yes, move the pending contract class data to the final storage.
//!     > Else, remove it from the pending storage altogether.

#![feature(iter_collect_into)]

use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use futures::poll;
use futures::stream::{Stream, StreamExt};
use log::error;
use mc_db::contract_class_data_db::ContractClassDataDb;
use mc_db::MappingDb;
use sc_transaction_pool_api::{TransactionPool, TransactionStatus, TransactionStatusStreamFor};
use sp_runtime::traits::Block as BlockT;
use starknet_api::core::ClassHash;
use starknet_api::transaction::TransactionHash;
use tokio::sync::mpsc::UnboundedReceiver;

// Content of the message sent through the mpsc channel
// It contains everything we need to decide what to do with the ContractClassData
// (hash of the declare transaction,
//  hash of the class declared,
//  watcher of the matching Substrate transaction status)
type DeclareTransactionStatusStream<P> = (TransactionHash, ClassHash, Pin<Box<TransactionStatusStreamFor<P>>>);

// Wrapper arround an `UnboundedReceiver` and a buffer.
// It encapsulates the logic for pulling multiple elements out of the channel at the same time.
//
// The buffer must have a capacity > 0. Use `new` to make sure it is conscructed correctly.
pub struct DeclareTransactionJobReceiverFuture<B, P>
where
    P: TransactionPool<Block = B>,
    B: BlockT,
{
    receiver: UnboundedReceiver<DeclareTransactionStatusStream<P>>,
    buffer: Vec<DeclareTransactionStatusStream<P>>,
}

impl<B, P> DeclareTransactionJobReceiverFuture<B, P>
where
    P: TransactionPool<Block = B>,
    B: BlockT,
{
    #[inline(always)]
    pub fn new(receiver: UnboundedReceiver<DeclareTransactionStatusStream<P>>, limit: NonZeroUsize) -> Self {
        Self { receiver, buffer: Vec::with_capacity(limit.into()) }
    }

    #[inline(always)]
    pub fn buffer_as_mut_ref(&mut self) -> &mut Vec<DeclareTransactionStatusStream<P>> {
        &mut self.buffer
    }
}

// When calling `poll_next`, if new values have been pushed to the channel, stores up to limit of
// them in the buffer, and returns the exact amount.
//
// This whole struct only makes sense if limit > 0.
// We use `buffer.capacity()` as limit, so  use the `new` method
// that won't let you pass a non allocated `Vec`.
impl<B, P> Stream for DeclareTransactionJobReceiverFuture<B, P>
where
    P: TransactionPool<Block = B>,
    B: BlockT,
{
    type Item = usize;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let DeclareTransactionJobReceiverFuture { receiver, buffer, .. } = &mut *self;

        match receiver.poll_recv_many(cx, buffer, buffer.capacity()) {
            Poll::Pending => Poll::Pending,
            // For limit > 0, this means that the channel has been closed
            Poll::Ready(0) => Poll::Ready(None),
            Poll::Ready(amount) => Poll::Ready(Some(amount)),
        }
    }
}

pub async fn run_worker<B, P>(
    contract_class_db: Arc<ContractClassDataDb>,
    mapping_db: Arc<MappingDb<B>>,
    job_recv: UnboundedReceiver<DeclareTransactionStatusStream<P>>,
) where
    B: BlockT,
    P: TransactionPool<Block = B>,
{
    // We push everything we get from the receiver here, then repetitively iterate over it to poll
    // each transaction's status stream and take action when they end up being finalized.
    let mut unfinalized_declare_transactions = Vec::new();
    // Stores all the txs that were not yet mapped into our db when they finalized for later processing.
    let mut finalized_but_not_mapped_yet = HashMap::<B::Hash, Vec<(TransactionHash, ClassHash)>>::new();
    // Totally arbitrary limit.
    // The job_recv being unbounded, there is not risk of it being full.
    // Could be increased up to at most the max amount of declare tx possible to have in a single block.
    // More that this would be useless as the "poll receiver" task is woke up at least one (but most
    // likely multiple times) during the span of one blocktime.
    let streams_buffer_limit = unsafe { NonZeroUsize::new_unchecked(10) };
    let mut jobs_receiver = DeclareTransactionJobReceiverFuture::<B, P>::new(job_recv, streams_buffer_limit);

    loop {
        // Deal with new DeclareTx in the channel
        {
            // Check if new DeclareTx were created
            let opt_amount = match poll!(jobs_receiver.next()) {
                Poll::Pending => None,
                Poll::Ready(None) => panic!("the ContractClassData sender has been droped"),
                Poll::Ready(opt_amount) => opt_amount,
            };
            // If yes, add them to the queue
            if let Some(amount) = opt_amount {
                // Empty the buffer into `unfinalized_declare_transaction`
                jobs_receiver.buffer_as_mut_ref().drain(..amount).collect_into(&mut unfinalized_declare_transactions);
            }
        }

        // Deal with the finalized but not mapped
        // As soon as the Substrate block is synced, it will remove this block entry
        // and process every transaction in it.
        finalized_but_not_mapped_yet.retain(|substrate_block_hash, transactions: &mut Vec<_>| {
            let is_synced = mapping_db.is_synced(substrate_block_hash);
            // If it is now mapped, we take action
            if is_synced {
                for (transaction_hash, class_hash) in transactions {
                    persist_or_remove_pending_contract_class_data(
                        &contract_class_db,
                        &mapping_db,
                        *transaction_hash,
                        *class_hash,
                    );
                }
            }

            // Don't retain the synced enties
            !is_synced
        });

        // We did a bunch of blocking operations at this point,
        // let's yield the thread to the tokio runtime.
        tokio::task::yield_now().await;

        // Now deal with the transactions that are still not finalized
        // We use indexing rather than iterator to be able to remove entries while iterating.
        let mut i = 0;
        while i < unfinalized_declare_transactions.len() {
            let (transaction_hash, class_hash, tx_status_watcher_stream) = &mut unfinalized_declare_transactions[i];
            let transaction_hash = *transaction_hash;
            let class_hash = *class_hash;

            match poll!(tx_status_watcher_stream.next()) {
                // The Substrate block is final, we can decide what to do with those contract class data
                Poll::Ready(Some(TransactionStatus::Finalized((substrate_block_hash, _)))) => {
                    // Finalized but not synced yet, even if the Substrate tx is part of the Substrate block,
                    // we cannot know for sure that the matching Starknet tx is part of the Starknet matching block.
                    // Store it on the side to be processed later.
                    if !mapping_db.is_synced(&substrate_block_hash) {
                        finalized_but_not_mapped_yet
                            .entry(substrate_block_hash)
                            .and_modify(|v| v.push((transaction_hash, class_hash)))
                            .or_insert_with(|| vec![(transaction_hash, class_hash)]);
                    } else {
                        persist_or_remove_pending_contract_class_data(
                            &contract_class_db,
                            &mapping_db,
                            transaction_hash,
                            class_hash,
                        );
                    }

                    // `swap_remove` for perfs, therefore don't increment `i`
                    unfinalized_declare_transactions.swap_remove(i);
                    // We did some blocking work, let's yield
                    tokio::task::yield_now().await;
                }
                // Those are all final status, that mean the TX wasn't successfully executed.
                // We remove the related extra data we stored.
                Poll::Ready(None)
                | Poll::Ready(Some(TransactionStatus::Dropped))
                | Poll::Ready(Some(TransactionStatus::Invalid))
                | Poll::Ready(Some(TransactionStatus::Usurped(_)))
                | Poll::Ready(Some(TransactionStatus::FinalityTimeout(_))) => {
                    if let Err(e) = contract_class_db.remove_pending_contract_class_data(class_hash) {
                        error!(
                            "failed to remove pending contract class data for class with hashe
            {class_hash:?}: {e}"
                        )
                    };
                    unfinalized_declare_transactions.swap_remove(i);
                    tokio::task::yield_now().await;
                }
                // Here are all the intermediate status (Pending, InBlock, etc)
                // Do nothing, just keep iterating
                _ => i += 1,
            }
        }

        tokio::task::yield_now().await;
    }
}

// To avoid false negative, this method should only be called after you made sure the Substrate
// block containing the Starknet transaction with hash `transaction_hash` has been synced int
// the Madara backend.
fn persist_or_remove_pending_contract_class_data<B: BlockT>(
    contract_class_db: &Arc<ContractClassDataDb>,
    mapping_db: &Arc<MappingDb<B>>,
    transaction_hash: TransactionHash,
    class_hash: ClassHash,
) {
    // If the Starknet tx has been mapped, this means it didn't failed.
    // Declare being a non-revertible Tx, it wouldn't be in the block otherwise.
    #[allow(clippy::collapsible_else_if)]
    if mapping_db.transaction_is_mapped(transaction_hash) {
        if let Err(e) = contract_class_db.persist_pending_contract_class_data(class_hash) {
            error!(
                "failed to persist pending contract class data for class with hash
            {class_hash:?}: {e}"
            )
        }
    } else {
        if let Err(e) = contract_class_db.remove_pending_contract_class_data(class_hash) {
            error!(
                "failed to remove pending contract class data for class with hash
                {class_hash:?}: {e}"
            )
        }
    }
}
