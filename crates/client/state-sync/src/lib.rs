mod ethereum;
mod sync;

pub mod parse_da;
#[cfg(test)]
mod tests;

use std::sync::Arc;

use async_trait::async_trait;
use ethers::types::U256;
use futures::channel::mpsc;
use futures::prelude::*;
use mc_db::L1L2BlockMapping;
use sc_client_api::backend::Backend;
use sp_blockchain::HeaderBackend;
use sp_core::H256;
use sp_runtime::generic::Header as GenericHeader;
use sp_runtime::traits::{BlakeTwo256, Block as BlockT};
use starknet_api::state::StateDiff;
use sync::StateWriter;

use crate::sync::SyncStateDiff;

type EncodeStateDiff = Vec<U256>;

#[derive(Debug, Clone)]
pub struct FetchState {
    pub l1_l2_block_mapping: L1L2BlockMapping,
    pub post_state_root: U256,
    pub state_diff: StateDiff,
}

#[async_trait]
pub trait StateFetcher {
    async fn state_diff(&self, l1_from: u64, l2_start: u64) -> Result<Vec<FetchState>, Error>;
}

// TODO pass a config then create state_fetcher
pub async fn run<B, C, BE, SF>(
    state_fetcher: Arc<SF>,
    madara_backend: Arc<mc_db::Backend<B>>,
    substrate_client: Arc<C>,
    substrate_backend: Arc<BE>,
) -> Result<impl Future<Output = ()> + Send, Error>
where
    B: BlockT<Hash = H256, Header = GenericHeader<u32, BlakeTwo256>>,
    C: HeaderBackend<B> + 'static,
    BE: Backend<B> + 'static,
    SF: StateFetcher + Send + Sync + 'static,
{
    let (mut tx, mut rx) = mpsc::unbounded::<FetchState>();

    let state_writer = StateWriter::new(substrate_client, substrate_backend, madara_backend);
    let state_writer = Arc::new(state_writer);
    let state_fetcher_clone = state_fetcher.clone();

    let fetcher_task = async move {
        loop {
            if let Ok(fs) = state_fetcher_clone.state_diff(10, 11).await {
                // TODO channel send vec. not a loop?
                for s in fs.iter() {
                    let _ = tx.send(s.clone());
                }
            }
            // TODO time.sleep() need sleep ??
        }
    };

    let state_write_task = async move {
        loop {
            if let Some(s) = rx.next().await {
                let _ = state_writer.apply_state_diff(0, SyncStateDiff::default());
                // TODO after apply state diff success. write sync state to madara backend.
            }
        }
    };

    let task =
        future::ready(()).then(move |_| future::select(Box::pin(fetcher_task), Box::pin(state_write_task))).map(|_| ());

    Ok(task)
}

#[derive(Debug, Clone)]
pub enum Error {
    AlreadyInChain,
    UnknownBlock,
    ConstructTransaction(String),
    CommitStorage(String),
    L1Connection(String),
    L1EventDecode,
    L1StateError(String),
    Other(String),
}
