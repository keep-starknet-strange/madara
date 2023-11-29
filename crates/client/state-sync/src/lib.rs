mod ethereum;
mod parser;
mod writer;

#[cfg(test)]
mod tests;

use std::cmp::Ordering;
use std::fs::File;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use ethereum::EthereumStateFetcher;
use ethers::types::{Address, H256, U256};
use futures::channel::mpsc;
use futures::prelude::*;
use log::error;
use mc_db::L1L2BlockMapping;
use pallet_starknet::runtime_api::StarknetRuntimeApi;
use sc_client_api::backend::Backend;
use serde::{Deserialize, Serialize};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_consensus::SyncOracle;
use sp_runtime::generic::Header as GenericHeader;
use sp_runtime::traits::{BlakeTwo256, Block as BlockT};
use starknet_api::state::StateDiff;
use writer::StateWriter;

const LOG_TARGET: &str = "state-sync";

// StateSyncConfig defines the parameters to start the task of syncing states from L1.
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct StateSyncConfig {
    // The block from which syncing starts on L1.
    pub l1_start: u64,
    // The address of core contract on L1.
    pub core_contract: String,
    // The address of verifier contract on L1.
    pub verifier_contract: String,
    // The address of memory page contract on L1.
    pub memory_page_contract: String,
    // The block from which syncing starts on L2.
    pub l2_start: u64,
    // The RPC url for L1.
    pub l1_url: String,
    // The starknet state diff format changed in l1 block height.
    pub v011_diff_format_height: u64,
    // The number of blocks to query from L1 each time.
    pub fetch_block_step: u64,
    // The time interval for each query.
    pub syncing_fetch_interval: u64,
    // The time interval for each query.
    pub synced_fetch_interval: u64,
}

impl TryFrom<&PathBuf> for StateSyncConfig {
    type Error = String;

    fn try_from(path: &PathBuf) -> Result<Self, Self::Error> {
        let file = File::open(path).map_err(|e| format!("error opening state sync config: {e}"))?;
        serde_json::from_reader(file).map_err(|e| format!("error parsing state sync config: {e}"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchState {
    pub l1_l2_block_mapping: L1L2BlockMapping,
    pub post_state_root: U256,
    pub state_diff: StateDiff,
}

impl PartialOrd for FetchState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.l1_l2_block_mapping.l2_block_number.cmp(&other.l1_l2_block_mapping.l2_block_number))
    }
}

impl Ord for FetchState {
    fn cmp(&self, other: &Self) -> Ordering {
        self.l1_l2_block_mapping.l2_block_number.cmp(&other.l1_l2_block_mapping.l2_block_number)
    }
}

#[async_trait]
pub trait StateFetcher {
    async fn state_diff<B, C>(&mut self, l1_from: u64, l2_start: u64, client: Arc<C>) -> Result<Vec<FetchState>, Error>
    where
        B: BlockT,
        C: ProvideRuntimeApi<B> + HeaderBackend<B>,
        C::Api: StarknetRuntimeApi<B>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum SyncStatus {
    SYNCING,
    SYNCED,
}

pub fn create_and_run<B, C, BE>(
    config_path: PathBuf,
    madara_backend: Arc<mc_db::Backend<B>>,
    substrate_client: Arc<C>,
    substrate_backend: Arc<BE>,
) -> Result<(impl Future<Output = ()> + Send, Arc<dyn SyncOracle + Send + Sync>), Error>
where
    B: BlockT<Hash = H256, Header = GenericHeader<u32, BlakeTwo256>>,
    C: HeaderBackend<B> + ProvideRuntimeApi<B> + 'static,
    C::Api: StarknetRuntimeApi<B>,
    BE: Backend<B> + 'static,
{
    let state_sync_config = StateSyncConfig::try_from(&config_path).map_err(|e| Error::Other(e.to_string()))?;

    let contract_address =
        state_sync_config.core_contract.parse::<Address>().map_err(|e| Error::Other(e.to_string()))?;
    let verifier_address =
        state_sync_config.verifier_contract.parse::<Address>().map_err(|e| Error::Other(e.to_string()))?;
    let memory_page_address =
        state_sync_config.memory_page_contract.parse::<Address>().map_err(|e| Error::Other(e.to_string()))?;

    let eth_url_list = vec![state_sync_config.l1_url];

    let sync_status = Arc::new(Mutex::new(SyncStatus::SYNCING));
    let state_fetcher: EthereumStateFetcher<ethers::providers::Http> = EthereumStateFetcher::new(
        contract_address,
        verifier_address,
        memory_page_address,
        eth_url_list,
        state_sync_config.v011_diff_format_height,
        sync_status.clone(),
    )?;

    let sync_status_oracle = Arc::new(SyncStatusOracle { sync_status });

    let mut mapping = L1L2BlockMapping {
        l1_block_hash: Default::default(),
        l1_block_number: state_sync_config.l1_start,
        l2_block_hash: Default::default(),
        l2_block_number: state_sync_config.l2_start,
    };

    if let Ok(last_mapping) = madara_backend.meta().last_l1_l2_mapping() {
        if last_mapping.l1_block_number < mapping.l1_block_number
            || last_mapping.l2_block_number < mapping.l2_block_number
        {
            mapping.l1_block_number = state_sync_config.l1_start;
            mapping.l2_block_number = state_sync_config.l2_start;
        }
    }

    madara_backend.meta().write_last_l1_l2_mapping(&mapping).map_err(|e| Error::Other(e.to_string()))?;

    Ok((run(state_fetcher, madara_backend, substrate_client, substrate_backend), sync_status_oracle))
}

// TODO pass a config then create state_fetcher
pub fn run<B, C, BE, SF>(
    mut state_fetcher: SF,
    madara_backend: Arc<mc_db::Backend<B>>,
    substrate_client: Arc<C>,
    substrate_backend: Arc<BE>,
) -> impl Future<Output = ()> + Send
where
    B: BlockT<Hash = H256, Header = GenericHeader<u32, BlakeTwo256>>,
    C: HeaderBackend<B> + ProvideRuntimeApi<B> + 'static,
    C::Api: StarknetRuntimeApi<B>,
    BE: Backend<B> + 'static,
    SF: StateFetcher + Send + Sync + 'static,
{
    let (mut tx, mut rx) = mpsc::unbounded::<Vec<FetchState>>();

    let state_writer = StateWriter::new(substrate_client.clone(), substrate_backend, madara_backend.clone());
    let state_writer = Arc::new(state_writer);

    let madara_backend_clone = madara_backend.clone();
    let fetcher_task = async move {
        let mut eth_from_height: u64 = 0;
        let mut starknet_start_height: u64 = 0;

        match madara_backend_clone.clone().meta().last_l1_l2_mapping() {
            Ok(mapping) => {
                eth_from_height = mapping.l1_block_number + 1;
                starknet_start_height = mapping.l2_block_number + 1;
            }
            Err(e) => {
                error!(target: LOG_TARGET, "read last l1 l2 mapping failed, error {:#?}.", e);
            }
        }

        loop {
            match state_fetcher.state_diff(eth_from_height, starknet_start_height, substrate_client.clone()).await {
                Ok(mut fetched_states) => {
                    if fetched_states.is_empty() {
                        eth_from_height += 10;
                        continue;
                    }
                    fetched_states.sort();

                    if let Some(last) = fetched_states.last() {
                        eth_from_height = last.l1_l2_block_mapping.l1_block_number + 1;
                        starknet_start_height = last.l1_l2_block_mapping.l2_block_number + 1;
                    }

                    let _res = tx.send(fetched_states).await;
                }
                Err(e) => {
                    error!(target: LOG_TARGET, "fetch state diff from l1 has error {:#?}", e);
                }
            }
            // TODO syncing, synced
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    };

    let state_write_task = async move {
        loop {
            if let Some(fetched_states) = rx.next().await {
                for state in fetched_states.iter() {
                    let _ = state_writer.apply_state_diff(
                        state.l1_l2_block_mapping.l2_block_number,
                        state.l1_l2_block_mapping.l2_block_hash,
                        &state.state_diff,
                    );
                }

                if let Some(last) = fetched_states.last() {
                    if let Err(e) = madara_backend.meta().write_last_l1_l2_mapping(&last.l1_l2_block_mapping) {
                        error!(target: LOG_TARGET, "write to madara backend has error {:#?}", e);
                        break;
                    }
                }
            }
        }
    };

    future::select(Box::pin(fetcher_task), Box::pin(state_write_task)).map(|_| ())
}

fn u256_to_h256(u256: U256) -> H256 {
    let mut bytes = [0; 32];
    u256.to_big_endian(&mut bytes);
    let mut h256_bytes = [0; 32];
    h256_bytes.copy_from_slice(&bytes[..32]);
    H256::from(h256_bytes)
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
    TypeError(String),
    Other(String),
}

struct SyncStatusOracle {
    sync_status: Arc<Mutex<SyncStatus>>,
}

impl SyncOracle for SyncStatusOracle {
    fn is_major_syncing(&self) -> bool {
        self.sync_status.lock().map(|status| *status == SyncStatus::SYNCING).unwrap_or_default()
    }

    fn is_offline(&self) -> bool {
        false
    }
}
