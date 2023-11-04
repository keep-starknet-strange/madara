//! Contains the code required to fetch data from the network efficiently.

use std::sync::Arc;
use std::time::Duration;

use reqwest::Url;
use sp_core::H256;
use starknet_gateway::sequencer::models::BlockId;
use starknet_gateway::SequencerGatewayProvider;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

use crate::CommandSink;

/// The configuration of the worker responsible for fetching new blocks from the network.
pub struct BlockFetchConfig {
    /// The URL of the sequencer gateway.
    pub gateway: Url,
    /// The URL of the feeder gateway.
    pub feeder_gateway: Url,
    /// The ID of the chain served by the sequencer gateway.
    pub chain_id: starknet_ff::FieldElement,
    /// The number of tasks spawned to fetch blocks.
    pub workers: u32,
    /// Whether to play a sound when a new block is fetched.
    pub sound: bool,
}

/// Used to determine which Ids are required to be fetched.
struct IdServer {
    /// When `failed_ids` is empty, the next ID to fetch.
    next_id: u64,
    /// A list of IDs that have failed.
    failed_ids: Vec<u64>,
}

impl IdServer {
    /// Creates a new ID server.
    pub fn new(start_id: u64) -> Self {
        Self { next_id: start_id, failed_ids: Vec::new() }
    }

    /// Acquires an ID to fetch.
    pub fn acquire(&mut self) -> u64 {
        match self.failed_ids.pop() {
            Some(id) => id,
            None => {
                let id = self.next_id;
                self.next_id += 1;
                id
            }
        }
    }

    /// Releases an ID, scheduling it for a retry.
    pub fn release(&mut self, id: u64) {
        self.failed_ids.push(id);
    }
}

/// The state required to syncronize worker threads.
struct SyncState {
    /// The hash of the last sealed block.
    last_hash: Option<H256>,
    /// The block number of the next block to be sealed.
    next_number: u64,
}

impl SyncState {
    /// Creates a new sync state.
    #[inline]
    pub fn new(start_at: u64) -> Self {
        Self { last_hash: None, next_number: start_at }
    }
}

/// The state that is shared between fetch workers.
struct WorkerSharedState {
    /// The ID server.
    ids: Mutex<IdServer>,
    /// The client used to perform requests.
    client: SequencerGatewayProvider,
    /// The block sender.
    sender: Sender<mp_block::Block>,
    /// The state of the last block.
    sync_state: Mutex<SyncState>,
}

/// Fetches blocks from the network and sends them to the given sender.
pub async fn fetch_blocks(
    command_sink: CommandSink,
    sender: Sender<mp_block::Block>,
    config: BlockFetchConfig,
    start_at: u64,
) {
    let shared_state = Arc::new(WorkerSharedState {
        client: SequencerGatewayProvider::new(config.gateway, config.feeder_gateway, config.chain_id),
        sender,
        ids: Mutex::new(IdServer::new(start_at)),
        sync_state: Mutex::new(SyncState::new(start_at)),
    });

    for _ in 0..config.workers {
        let state = shared_state.clone();
        tokio::spawn(start_worker(state, command_sink.clone()));
    }
}

/// Starts a worker task.
async fn start_worker(state: Arc<WorkerSharedState>, mut command_sink: CommandSink) {
    // Retry with the same block id.
    loop {
        let block_id = state.ids.lock().await.acquire();
        match get_and_dispatch_block(&state, block_id, &mut command_sink).await {
            Ok(()) => (),
            Err(err) => {
                state.ids.lock().await.release(block_id);
                eprintln!("Error sending block #{}: {:?}", block_id, err);
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
        }
    }
}

/// Gets a block from the network and sends it to the other half of the channel.
async fn get_and_dispatch_block(
    state: &WorkerSharedState,
    block_id: u64,
    command_sink: &mut CommandSink,
) -> Result<(), String> {
    let block =
        state.client.get_block(BlockId::Number(block_id)).await.map_err(|e| format!("failed to get block: {e}"))?;
    let block = super::convert::block(&block);

    let mut lock;
    loop {
        lock = state.sync_state.lock().await;
        if lock.next_number == block_id {
            break;
        }
        drop(lock);
        tokio::task::yield_now().await;
    }

    state.sender.send(block).await.map_err(|e| format!("failed to dispatch block: {e}"))?;
    let hash = create_block(command_sink, lock.last_hash).await?;

    lock.last_hash = Some(hash);
    lock.next_number += 1;

    Ok(())
}

/// Notifies the consensus engine that a new block should be created.
async fn create_block(command_sink: &mut CommandSink, _parent_hash: Option<H256>) -> Result<H256, String> {
    let (sender, receiver) = futures_channel::oneshot::channel();

    command_sink
        .try_send(sc_consensus_manual_seal::rpc::EngineCommand::SealNewBlock {
            create_empty: true,
            finalize: true,
            parent_hash: None,
            sender: Some(sender),
        })
        .unwrap();

    let create_block_info = receiver
        .await
        .map_err(|err| format!("failed to seal block: {err}"))?
        .map_err(|err| format!("failed to seal block: {err}"))?;

    #[cfg(feature = "m")]
    {
        super::m::play_note(create_block_info.hash.to_low_u64_ne());
    }

    Ok(create_block_info.hash)
}
