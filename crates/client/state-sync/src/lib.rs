//! # State Sync
//!
//! The state sync module facilitates synchronization of state data between Layer 1 (L1) and Layer 2
//! (L2) blockchains. It includes components for fetching state differences, processing them, and
//! updating the substrate storage accordingly.
//!
//! ## Modules
//!
//! - [`ethereum`](ethereum): Contains the Ethereum state fetcher implementation for fetching state
//!   differences from an Ethereum node.
//! - [`parser`](parser): Placeholder for any parsing-related functionality.
//! - [`writer`](writer): Implements the `StateWriter` responsible for applying state differences to
//!   the substrate storage.
//!
//! ## Public Interface
//!
//! The main entry point for using the state sync module is through the [`create_and_run`] function.
//!
//! ## Functions
//!
//! - [`create_and_run`](create_and_run): Initializes and runs the state sync task.
//! - [`run`](run): Defines the main logic of the state sync task.
//! - [`u256_to_h256`](u256_to_h256): Converts a `U256` to `H256`.
//!
//! ## Enums
//!
//! - [`Error`](Error): Represents errors encountered during state syncing.
//! - [`SyncStatus`](SyncStatus): Represents the current status of state synchronization.
//!
//! ## Structs
//!
//! - [`StateSyncConfig`](StateSyncConfig): Configuration struct for starting the state sync task.
//! - [`SyncStatusOracle`](SyncStatusOracle): Provides information about the sync status.
//!
//! ## Examples
//!
//! ```rust
//! # use std::path::PathBuf;
//! # use std::sync::Arc;
//! # use sc_client_api::backend::Backend;
//! # use sp_blockchain::HeaderBackend;
//! # use sp_runtime::generic::Header as GenericHeader;
//! # use sp_runtime::traits::BlakeTwo256;
//! # use starknet_api::state::StateDiff;
//! # use tokio::runtime::Runtime;
//! # use state_sync::*;
//!
//! # fn main() {
//! let config_path = PathBuf::from("config.json");
//! let madara_backend: Arc<mc_db::Backend<TestBlock>> = Arc::new(mc_db::Backend::new());
//! let substrate_client: Arc<TestClient> = Arc::new(TestClient);
//! let substrate_backend: Arc<TestBackend> = Arc::new(TestBackend);
//!
//! let result = create_and_run(
//!     config_path,
//!     madara_backend.clone(),
//!     substrate_client.clone(),
//!     substrate_backend.clone(),
//! );
//!
//! match result {
//!     Ok((future, _sync_status_oracle)) => {
//!         let mut rt = Runtime::new().unwrap();
//!         rt.block_on(future);
//!     }
//!     Err(e) => eprintln!("Error: {:?}", e),
//! }
//! # }
//! ```
//!
//! [`create_and_run`]: fn.create_and_run.html
//! [`run`]: fn.run.html
//! [`u256_to_h256`]: fn.u256_to_h256.html
//! [`Error`]: enum.Error.html
//! [`SyncStatus`]: enum.SyncStatus.html
//! [`StateSyncConfig`]: struct.StateSyncConfig.html
//! [`SyncStatusOracle`]: struct.SyncStatusOracle.html
//! ```

mod errors;
mod ethereum;
mod parser;
mod writer;

#[cfg(test)]
mod tests;

use std::cmp::Ordering;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use ethers::types::{Address, H256, U256};
use futures::channel::mpsc;
use futures::prelude::*;
use log::error;
use mc_db::L1L2BlockMapping;
use pallet_starknet_runtime_api::StarknetRuntimeApi;
use parking_lot::Mutex;
use sc_client_api::backend::Backend;
use serde::{Deserialize, Serialize};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_consensus::SyncOracle;
use sp_runtime::generic::Header as GenericHeader;
use sp_runtime::traits::{BlakeTwo256, Block as BlockT};
use starknet_api::state::StateDiff;

use crate::errors::Error;
use crate::ethereum::EthereumStateFetcher;
use crate::writer::StateWriter;

const LOG_TARGET: &str = "state-sync";

/// StateSyncConfig defines the parameters to start the task of syncing states from L1.
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct StateSyncConfig {
    /// The block from which syncing starts on L1.
    pub l1_start: u64,
    /// The address of the core contract on L1.
    pub core_contract: String,
    /// The address of the verifier contract on L1.
    pub verifier_contract: String,
    /// The address of the memory page contract on L1.
    pub memory_page_contract: String,
    /// The block from which syncing starts on L2.
    pub l2_start: u64,
    /// The RPC URLs for L1.
    pub l1_url_list: Vec<String>,
    /// The starknet state diff format changed in L1 block height.
    pub v011_diff_format_height: u64,
    /// The starknet state diff contains contract construct args.
    pub constructor_args_diff_height: u64,
    /// The number of blocks to query from L1 each time.
    #[serde(default)]
    pub fetch_block_step: u64,
    /// The time interval for each query.
    #[serde(default)]
    pub syncing_fetch_interval: u64,
    /// The time interval for each query.
    #[serde(default)]
    pub synced_fetch_interval: u64,
}

impl TryFrom<&PathBuf> for StateSyncConfig {
    type Error = Error;

    fn try_from(path: &PathBuf) -> Result<Self, Self::Error> {
        let file = File::open(path).map_err(|e| Error::DeserializeConf(e.to_string()))?;
        serde_json::from_reader(file).map_err(|e| Error::DeserializeConf(e.to_string()))
    }
}

/// Struct representing the fetched state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchState {
    /// Mapping between blocks from L1 to L2.
    pub l1_l2_block_mapping: L1L2BlockMapping,
    /// State root after applying the state diff.
    pub post_state_root: U256,
    /// State difference between two blocks.
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

/// StateFetcher defines a trait for fetching StateDiff from L1.
#[async_trait]
pub trait StateFetcher {
    /// Retrieves the StateDiff from L1.
    ///
    /// # Arguments
    ///
    /// * `l1_from` - The starting block number on L1.
    /// * `l2_start` - The starting block number on L2.
    /// * `client` - An Arc pointer to a type that implements ProvideRuntimeApi and HeaderBackend
    ///   traits, where its associated Api implements StarknetRuntimeApi.
    ///
    /// # Returns
    ///
    /// A Result containing a Vec of FetchState or an Error.
    async fn state_diff<B, C>(&mut self, l1_from: u64, l2_start: u64, client: Arc<C>) -> Result<Vec<FetchState>, Error>
    where
        B: BlockT,
        C: ProvideRuntimeApi<B> + HeaderBackend<B>,
        C::Api: StarknetRuntimeApi<B>;

    async fn get_highest_block_number(&mut self) -> Result<u64, Error>;
}

/// Enum representing the synchronization status.
#[derive(Debug, PartialEq, Eq)]
pub enum SyncStatus {
    /// Represents the status when synchronization is in progress.
    SYNCING,
    /// Represents the status when synchronization is completed.
    SYNCED,
}

impl Default for SyncStatus {
    fn default() -> Self {
        Self::SYNCING
    }
}

/// Creates and runs a state diff sync task along with a SyncOracle.
///
/// This function initializes a state diff sync task and SyncOracle based on the provided
/// parameters.
///
/// # Arguments
///
/// * `config_path` - The path to the configuration file.
/// * `madara_backend` - Arc pointer to the Madara backend.
/// * `substrate_client` - Arc pointer to the Substrate client.
/// * `substrate_backend` - Arc pointer to the Substrate backend.
///
/// # Returns
///
/// A Result containing a tuple with a Future and an Arc to the SyncOracle, or an Error.
///
/// # Generic Parameters
///
/// * `B` - Block type with Hash of H256 and Header of GenericHeader<u32, BlakeTwo256>.
/// * `C` - Type implementing HeaderBackend<B>, ProvideRuntimeApi<B>, and 'static.
/// * `BE` - Type implementing Backend<B> and 'static.
///
/// # Errors
///
/// Returns an Error if there are issues parsing configuration, contract addresses, or writing
/// mapping data.
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
    let config = StateSyncConfig::try_from(&config_path)?;

    let contract_address = config.core_contract.parse::<Address>().map_err(|e| Error::ParseAddress(e.to_string()))?;
    let verifier_address =
        config.verifier_contract.parse::<Address>().map_err(|e| Error::ParseAddress(e.to_string()))?;
    let memory_page_address =
        config.memory_page_contract.parse::<Address>().map_err(|e| Error::ParseAddress(e.to_string()))?;

    let sync_status = Arc::new(Mutex::new(SyncStatus::SYNCING));
    let state_fetcher: EthereumStateFetcher<ethers::providers::Http> = EthereumStateFetcher::new(
        contract_address,
        verifier_address,
        memory_page_address,
        config.l1_url_list,
        config.v011_diff_format_height,
        sync_status.clone(),
        config.constructor_args_diff_height,
    )?;

    let sync_status_oracle = Arc::new(SyncStatusOracle { sync_status });

    let mut mapping = L1L2BlockMapping {
        l1_block_hash: Default::default(),
        l1_block_number: config.l1_start,
        l2_block_hash: Default::default(),
        l2_block_number: config.l2_start,
    };

    if let Ok(last_mapping) = madara_backend.meta().last_l1_l2_mapping() {
        if last_mapping.l1_block_number < mapping.l1_block_number
            || last_mapping.l2_block_number < mapping.l2_block_number
        {
            mapping.l1_block_number = config.l1_start;
            mapping.l2_block_number = config.l2_start;
        }
    }

    madara_backend.meta().write_last_l1_l2_mapping(&mapping).map_err(|e| Error::CommitMadara(e.to_string()))?;

    Ok((run(state_fetcher, madara_backend, substrate_client, substrate_backend), sync_status_oracle))
}

/// Creates an asynchronous task for state synchronization and initiates its execution.
///
/// This function creates an asynchronous task for state synchronization using the provided
/// state_fetcher and other backend parameters. It then initiates the execution of this task
/// by returning it as a Future to be executed asynchronously by the runtime.
///
/// # Arguments
///
/// * `state_fetcher` - The state fetcher responsible for fetching state differences.
/// * `madara_backend` - Arc pointer to the Madara backend.
/// * `substrate_client` - Arc pointer to the Substrate client.
/// * `substrate_backend` - Arc pointer to the Substrate backend.
///
/// # Returns
///
/// A Future representing the asynchronous state synchronization task.
///
/// # Generic Parameters
///
/// * `B` - Block type with Hash of H256 and Header of GenericHeader<u32, BlakeTwo256>.
/// * `C` - Type implementing HeaderBackend<B>, ProvideRuntimeApi<B>, and 'static.
/// * `BE` - Type implementing Backend<B> and 'static.
/// * `SF` - Type implementing StateFetcher, Send, Sync, and 'static.
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
                eth_from_height = mapping.l1_block_number;
                starknet_start_height = mapping.l2_block_number;
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
            match state_fetcher.get_highest_block_number().await {
                Ok(highest_block_number) => {
                    if highest_block_number > eth_from_height {
                        continue;
                    }
                    if eth_from_height == highest_block_number {
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }
                Err(e) => {
                    error!(target: LOG_TARGET, "get highest block number from l1 has error {:#?}", e);
                }
            };
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

/// Represents a SyncStatusOracle used for querying synchronization status.
struct SyncStatusOracle {
    sync_status: Arc<Mutex<SyncStatus>>,
}

impl SyncOracle for SyncStatusOracle {
    fn is_major_syncing(&self) -> bool {
        *self.sync_status.lock() == SyncStatus::SYNCING
    }

    fn is_offline(&self) -> bool {
        false
    }
}
