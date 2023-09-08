use std::collections::HashMap;
use std::sync::Arc;

use mc_storage::{OverrideHandle, StorageOverride};
use mp_starknet::block::Block as StarknetBlock;
use mp_starknet::storage::StarknetStorageSchemaVersion;
use sc_service::SpawnTaskHandle;
use sp_runtime::traits::Block as BlockT;
use tokio::sync::{mpsc, oneshot};

use crate::cache::lru_cache::LRUCache;

mod lru_cache;

type WaitList<Hash, T> = HashMap<Hash, Vec<oneshot::Sender<Option<T>>>>;

enum StarknetDataCacheMessage<B: BlockT> {
    RequestBlockByHash {
        block_hash: B::Hash,
        schema: StarknetStorageSchemaVersion,
        response_tx: oneshot::Sender<Option<StarknetBlock>>,
    },
    FetchedBlockByHash {
        block_hash: B::Hash,
        block: Option<StarknetBlock>,
    },
}

/// Manage LRU caches for block data and their transaction statuses.
/// These are large and take a lot of time to fetch from the database.
/// Storing them in an LRU cache will allow to reduce database accesses
/// when many subsequent requests are related to the same blocks.
pub struct StarknetDataCacheTask<B: BlockT>(mpsc::Sender<StarknetDataCacheMessage<B>>);

impl<B: BlockT> StarknetDataCacheTask<B> {
    pub fn new(
        spawn_handle: SpawnTaskHandle,
        overrides: Arc<OverrideHandle<B>>,
        cache_max_sizes: (Option<usize>, Option<usize>),
        prometheus_registry: Option<prometheus_endpoint::Registry>,
    ) -> Self {
        let (task_tx, mut task_rx) = mpsc::channel(100);
        let outer_task_tx = task_tx.clone();
        let outer_spawn_handle = spawn_handle.clone();

        outer_spawn_handle.spawn("StarknetDataCacheTask", None, async move {
            let mut blocks_cache = LRUCache::<B::Hash, StarknetBlock>::new(
                "blocks_cache",
                cache_max_sizes.0,
                cache_max_sizes.1,
                prometheus_registry.clone(),
            );

            let mut awaiting_blocks = HashMap::<B::Hash, Vec<oneshot::Sender<Option<StarknetBlock>>>>::new();

            // Handle all incoming messages.
            // Exits when there are no more senders.
            // Any long computation should be spawned in a separate task
            // to keep this task handle messages as soon as possible.
            while let Some(message) = task_rx.recv().await {
                use StarknetDataCacheMessage::*;
                match message {
                    RequestBlockByHash { block_hash, schema, response_tx } => Self::request_current(
                        &spawn_handle,
                        &mut blocks_cache,
                        &mut awaiting_blocks,
                        Arc::clone(&overrides),
                        block_hash,
                        schema,
                        response_tx,
                        task_tx.clone(),
                        move |handler| FetchedBlockByHash { block_hash, block: handler.get_block_by_hash(block_hash) },
                    ),
                    FetchedBlockByHash { block_hash, block } => {
                        if let Some(wait_list) = awaiting_blocks.remove(&block_hash) {
                            for sender in wait_list {
                                let _ = sender.send(block.clone());
                            }
                        }

                        if let Some(block) = block {
                            // TODO: cache, log if insert fails ?
                            blocks_cache.insert(block_hash, block);
                        }
                    }
                }
            }
        });

        Self(outer_task_tx)
    }

    fn request_current<T, F>(
        spawn_handle: &SpawnTaskHandle,
        cache: &mut LRUCache<B::Hash, T>,
        wait_list: &mut WaitList<B::Hash, T>,
        overrides: Arc<OverrideHandle<B>>,
        block_hash: B::Hash,
        schema: StarknetStorageSchemaVersion,
        response_tx: oneshot::Sender<Option<T>>,
        task_tx: mpsc::Sender<StarknetDataCacheMessage<B>>,
        handler_call: F,
    ) where
        T: Clone + scale_codec::Encode,
        F: FnOnce(&Box<dyn StorageOverride<B>>) -> StarknetDataCacheMessage<B>,
        F: Send + 'static,
    {
        // Data is cached, we respond immediately.
        if let Some(data) = cache.get(&block_hash).cloned() {
            let _ = response_tx.send(Some(data));
            return;
        }

        // Another request already triggered caching but the
        // response is not known yet, we add the sender to the waiting
        // list.
        if let Some(waiting) = wait_list.get_mut(&block_hash) {
            waiting.push(response_tx);
            return;
        }

        // Data is neither cached nor already requested, so we start fetching
        // the data.
        wait_list.insert(block_hash, vec![response_tx]);

        spawn_handle.spawn("StarknetDataCacheTask Worker", None, async move {
            let handler = overrides.schemas.get(&schema).unwrap_or(&overrides.fallback);

            let message = handler_call(handler);
            let _ = task_tx.send(message).await;
        });
    }

    /// Cache for `get_block_by_block_hash`.
    pub async fn get_block_by_block_hash(
        &self,
        schema: StarknetStorageSchemaVersion,
        block_hash: B::Hash,
    ) -> Option<StarknetBlock> {
        let (response_tx, response_rx) = oneshot::channel();

        self.0.send(StarknetDataCacheMessage::RequestBlockByHash { block_hash, schema, response_tx }).await.ok()?;

        response_rx.await.ok()?
    }
}
