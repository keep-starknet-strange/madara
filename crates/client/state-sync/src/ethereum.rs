use std::result::Result;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use ethers::abi::RawLog;
use ethers::contract::{BaseContract, EthEvent, EthLogDecode};
use ethers::core::abi::parse_abi;
use ethers::providers::{Http, JsonRpcClient, Middleware, Provider};
use ethers::types::{Address, Filter, Log, H256, I256, U256};
use log::{debug, info};
use parking_lot::Mutex;
use sp_runtime::generic::BlockId;
use sp_runtime::traits::Block as BlockT;
use starknet_api::state::StateDiff;
use tokio::time::sleep;

use super::*;
use crate::errors::Error;

/// Constants for state and log search steps
const STATE_SEARCH_STEP: u64 = 10;
const LOG_SEARCH_STEP: u64 = 1000;

/// Describes a transaction on the ethereum blockchain,
/// this transaction triggered the StateUpdate event.
#[derive(Debug)]
pub struct StateUpdateTxInfo {
    block_hash: H256,
    block_number: u64,
    transaction_index: u64,
}

/// Represents a state update, combining ethereum transaction information with log state update
/// data.
#[derive(Debug)]
pub struct StateUpdate {
    l1_tx_info: StateUpdateTxInfo,
    l2_state: LogStateUpdate,
}

/// Ethereum contract event representing a log state update for l2.
///
/// In the testnet/mainnet, we noticed an upgrade in the Starknet Core contract,
/// leading to an additional field in the LogStateUpdate event.
/// This adjustment is made solely for forward compatibility with the older events.
/// If there is no need to maintain consistency with the Starknet mainnet
/// and testnet, you may safely ignore LogStateUpdateOld.
#[derive(Clone, Debug, PartialEq, Eq, EthEvent)]
#[ethevent(name = "LogStateUpdate")]
pub struct LogStateUpdate {
    pub global_root: U256,
    pub block_number: I256,
    pub block_hash: U256,
}

/// Ethereum contract event representing a log state update in old contract.
#[derive(Clone, Debug, PartialEq, Eq, EthEvent)]
#[ethevent(name = "LogStateUpdate")]
pub struct LogStateUpdateOld {
    pub global_root: U256,
    pub block_number: I256,
}

fn decode_log_state_update(raw_log: &RawLog) -> Result<LogStateUpdate, Error> {
    if let Ok(update) = <LogStateUpdate as EthLogDecode>::decode_log(raw_log) {
        return Ok(update);
    }

    // Attempt to decode the old version
    if let Ok(update) = <LogStateUpdateOld as EthLogDecode>::decode_log(raw_log) {
        return Ok(LogStateUpdate {
            global_root: update.global_root,
            block_number: update.block_number,
            block_hash: Default::default(),
        });
    }

    Err(Error::UnknownStateUpdateEvent)
}

/// Ethereum contract event representing a log state transition fact.
#[derive(Clone, Debug, PartialEq, Eq, EthEvent)]
#[ethevent(name = "LogStateTransitionFact")]
pub struct LogStateTransitionFact {
    pub fact: [u8; 32],
}

/// Ethereum contract event representing a log of memory pages hashes.
#[derive(Clone, Debug, PartialEq, Eq, EthEvent)]
#[ethevent(name = "LogMemoryPagesHashes")]
pub struct LogMemoryPagesHashes {
    pub fact: [u8; 32],
    pub pages_hashes: Vec<[u8; 32]>,
}

/// Ethereum contract event representing a continuous log of memory page facts.
#[derive(Clone, Debug, PartialEq, Eq, EthEvent)]
#[ethevent(name = "LogMemoryPageFactContinuous")]
pub struct LogMemoryPageFactContinuous {
    pub fact_hash: [u8; 32],
    pub memory_hash: U256,
    pub prod: U256,
}

/// Represents a continuous log of memory page facts with a corresponding transaction hash.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogMemoryPageFactContinuousWithTxHash {
    pub log_memory_page_fact_continuous: LogMemoryPageFactContinuous,
    pub tx_hash: H256,
}

/// Struct responsible for fetching and decoding Ethereum state information.
#[derive(Debug, Clone)]
pub struct EthereumStateFetcher<P: JsonRpcClient> {
    // HTTP provider for Ethereum
    http_provider: Provider<P>,

    // Address of starknet core contract on l1.
    core_contract: Address,

    // Address of starknet verifier contract on l1.
    verifier_contract: Address,

    // Address of starknet memory page contract on l1,
    // which store state diff of l2.
    memory_page_contract: Address,

    // Ethereum Node URL List
    eth_url_list: Vec<String>,

    // The index of `eth_url_list`, which `http_provider` used for connect ethereum.
    current_http_provider_index: Arc<AtomicUsize>,

    // The sync status.
    sync_status: Arc<Mutex<SyncStatus>>,

    // The Starknet block height after which StarkNet starts using the v0.11 state diff format.
    v011_diff_format_height: u64,

    // The Starknet block height after which constructor function details are excluded from the
    // state diff during contract deployment.
    constructor_args_diff_height: u64,
}

impl EthereumStateFetcher<Http> {
    /// Creates a new `EthereumStateFetcher` instance.
    ///
    /// # Arguments
    ///
    /// * `core_contract` - Address of the core contract on L1.
    /// * `verifier_contract` - Address of the verifier contract on L1.
    /// * `memory_page_contract` - Address of the memory page contract on L1.
    /// * `eth_url_list` - List of Ethereum node URLs.
    ///
    /// # Returns
    ///
    /// A `Result` containing the initialized `EthereumStateFetcher` or an `Error` if initialization
    /// fails.
    pub fn new(
        core_contract: Address,
        verifier_contract: Address,
        memory_page_contract: Address,
        eth_urls: Vec<String>,
        v011_diff_format_height: u64,
        sync_status: Arc<Mutex<SyncStatus>>,
        constructor_args_diff_height: u64,
    ) -> Result<Self, Error> {
        let provider = Provider::<Http>::try_from(eth_urls[0].clone()).map_err(Error::from)?;

        Ok(Self {
            http_provider: provider,
            core_contract,
            verifier_contract,
            memory_page_contract,
            eth_url_list: eth_urls,
            current_http_provider_index: Arc::new(AtomicUsize::new(0)),
            sync_status,
            v011_diff_format_height,
            constructor_args_diff_height,
        })
    }
}

impl<P: JsonRpcClient + Clone> EthereumStateFetcher<P> {
    /// Attempts to get Ethereum logs using the provided filter. If the initial attempt fails,
    /// it will retry by cycling through a list of Ethereum nodes until a successful response is received.
    ///
    /// # Arguments
    ///
    /// * `filter`: The `Filter` object defining the criteria for fetching logs from the Ethereum network.
    ///
    /// # Returns
    ///
    /// A `Result<Vec<Log>, Error>` containing either the fetched logs or an error if all providers fail.

    pub async fn get_logs_retry(&mut self, filter: &Filter) -> Result<Vec<Log>, Error> {
        // Try to fetch logs using the current HTTP provider.
        if let Ok(res) = self.http_provider.get_logs(filter).await {
            return Ok(res);
        }

        let mut retries = 0;
        loop {
            // Get the current Ethereum node URL and create a Provider instance.
            let index = self.current_http_provider_index.load(Ordering::Relaxed);
            let provider = Provider::<Http>::try_from(&self.eth_url_list[index]).map_err(Error::from)?;

            // Attempt to fetch logs using the current provider.
            match provider.get_logs(filter).await {
                Ok(logs) => return Ok(logs),
                Err(_e) => {
                    // If all Ethereum nodes have been tried, return an error.
                    retries += 1;
                    if retries > self.eth_url_list.len() {
                        return Err(Error::MaxRetryReached);
                    }

                    // change to next Ethereum node
                    self.current_http_provider_index.store((index + 1) % self.eth_url_list.len(), Ordering::Release);

                    // Calculate the wait time manually
                    let wait_time = self.calculate_backoff(retries);

                    // Wait for the calculated duration before the next attempt.
                    sleep(wait_time).await;
                }
            }
        }
    }

    /// Calculates the backoff delay based on an exponential formula with a maximum limit.
    ///
    /// # Arguments
    ///
    /// * `retries`: The number of retries attempted so far.
    ///
    /// # Returns
    ///
    /// A `Duration` representing the amount of time to wait before the next retry attempt.
    ///
    /// # Note
    ///
    /// This function implements a simple exponential backoff strategy with a cap at 10 seconds.
    pub fn calculate_backoff(&self, retries: usize) -> Duration {
        // A simple exponential backoff with a maximum delay of 10 seconds
        let base_delay = 1.0; // in seconds
        let max_delay = 10; // in seconds
        let exponential_factor: f64 = 2.0;

        // Calculate the backoff using the exponential factor
        let backoff = (base_delay * exponential_factor.powf(retries as f64)) as u64;

        // Ensure the backoff does not exceed the maximum delay
        let final_backoff = backoff.min(max_delay);

        Duration::from_secs(final_backoff)
    }

    /// Queries Ethereum state updates within a specified range.
    /// The range starts from `eth_from` and increments by `STATE_SEARCH_STEP`
    /// until the log is successfully queried.
    ///
    /// # Arguments
    ///
    /// * `eth_from` - The starting block for querying state updates log on ethereum.
    /// * `starknet_from` - The block with the lowest starknet block number found in the logs.
    /// If the state update log of starknet on ethereum contains blocks with numbers lower than
    /// this, they will be discarded. This design is implemented to avoid processing blocks that
    /// have already been handled in subsequent processes.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of `StateUpdate` instances or an `Error` if the query fails.
    pub(crate) async fn query_state_update(
        &mut self,
        eth_from: u64,
        starknet_from: u64,
    ) -> Result<Vec<StateUpdate>, Error> {
        debug!(target: LOG_TARGET, "~~ query_state_update, eth_from {}, starknet_from {}", eth_from, starknet_from);
        let highest_eth_block_number = self.http_provider.get_block_number().await.map_err(Error::from)?.as_u64();

        let filter = Filter::new()
            .address(self.core_contract)
            .events(vec!["LogStateUpdate(uint256,int256,uint256)", "LogStateUpdate(uint256,int256)"]);

        let mut from = if eth_from > highest_eth_block_number {
            *self.sync_status.lock() = SyncStatus::SYNCED;
            highest_eth_block_number
        } else {
            *self.sync_status.lock() = SyncStatus::SYNCING;
            eth_from
        };

        let mut to = eth_from + STATE_SEARCH_STEP;
        if to > highest_eth_block_number {
            to = highest_eth_block_number
        }

        let mut search_res = Vec::new();
        loop {
            let filter = filter.clone().from_block(from).to_block(to);

            let state_update_logs = self.get_logs_retry(&filter).await?;

            for log in state_update_logs.iter() {
                let raw_log = RawLog { topics: log.topics.clone(), data: log.data.to_vec() };
                let state_update_log = decode_log_state_update(&raw_log).and_then(|log_state_update| {
                    Ok(StateUpdate {
                        l1_tx_info: StateUpdateTxInfo {
                            block_hash: log.block_hash.ok_or(Error::EmptyValue)?,
                            block_number: log.block_number.ok_or(Error::EmptyValue)?.as_u64(),
                            transaction_index: log.transaction_index.ok_or(Error::EmptyValue)?.as_u64(),
                        },
                        l2_state: log_state_update,
                    })
                })?;

                if state_update_log.l2_state.block_number.as_u64() < starknet_from {
                    continue;
                }

                search_res.push(state_update_log);
            }

            // If 'from' is equal to 'to', it means we have reached the highest block on ethereum.
            if from == to {
                break;
            }

            // If successfully query the state update log,
            // break out of the loop and return the query results.
            if !search_res.is_empty() {
                break;
            }

            from = to;
            to = from + STATE_SEARCH_STEP;
        }

        Ok(search_res)
    }

    /// Queries ethereum logs for a state transition fact in a specific ethereum block.
    ///
    /// # Arguments
    ///
    /// * `eth_block_number` - The specific ethereum block contains a state transition fact log.
    /// * `tx_index` - The index of the transaction which trigger state transition fact log.
    ///
    /// # Returns
    ///
    /// A `Result` containing the fetched `LogStateTransitionFact` or an `Error` if the query fails.
    pub async fn query_state_transition_fact(
        &mut self,
        eth_block_number: u64,
        tx_index: u64,
    ) -> Result<LogStateTransitionFact, Error> {
        let filter = Filter::new()
            .address(self.core_contract)
            .event("LogStateTransitionFact(bytes32)")
            .from_block(eth_block_number)
            .to_block(eth_block_number);

        self.get_logs_retry(&filter)
            .await?
            .iter()
            .find(|log| log.transaction_index.is_some_and(|index| index.as_u64() == tx_index))
            .ok_or(Error::FindFact { block_number: eth_block_number, tx_index })
            .and_then(|log| {
                <LogStateTransitionFact as EthLogDecode>::decode_log(&(log.topics.clone(), log.data.to_vec()).into())
                    .map_err(Error::from)
            })
    }

    /// Find ethereum logs containing memory pages hashes for a specific state transition fact.
    /// The range starts from `eth_from` and decreases by `LOG_SEARCH_STEP`
    /// until the state transition fact is successfully queried.
    ///
    /// # Arguments
    ///
    /// * `eth_from` - The starting block number on Ethereum.
    /// * `state_transition_fact` - The state transition fact for filtering logs.
    ///
    /// # Returns
    ///
    /// A `Result` containing the fetched `LogMemoryPagesHashes` or an `Error` if the query fails.
    pub async fn find_memory_pages_hashes(
        &mut self,
        eth_from: u64,
        state_transition_fact: LogStateTransitionFact,
    ) -> Result<LogMemoryPagesHashes, Error> {
        let filter = Filter::new().address(self.verifier_contract).event("LogMemoryPagesHashes(bytes32,bytes32[])");

        let mut from = eth_from.saturating_sub(LOG_SEARCH_STEP);
        let mut to = eth_from;

        loop {
            if to == 0 {
                return Err(Error::BadStarknetFact);
            }
            let filter = filter.clone().from_block(from).to_block(to);

            let logs = self.get_logs_retry(&filter).await?;

            for l in logs.iter() {
                let pages_hashes =
                    <LogMemoryPagesHashes as EthLogDecode>::decode_log(&(l.topics.clone(), l.data.to_vec()).into())
                        .map_err(Error::from)?;

                if pages_hashes.fact.eq(&state_transition_fact.fact) {
                    return Ok(pages_hashes);
                }
            }

            to = from;
            from -= LOG_SEARCH_STEP;
        }
    }

    /// Queries continuous logs of memory page facts within a specific ethereum block range.
    ///     
    /// # Arguments
    ///
    /// * `eth_from` - The starting block number on Ethereum.
    /// * `target_memory_pages_hashes` - Mutable reference to a vector of memory pages hashes.
    /// When an element is found in Ethereum, it is removed from this vector.The process continues
    /// until the vector is empty, indicating that all memory pages have been found.
    ///
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of `LogMemoryPageFactContinuousWithTxHash` instances or an
    /// `Error` if the query fails.
    pub async fn find_memory_page_fact_continuous_logs(
        &mut self,
        eth_from: u64,
        mut target_memory_pages_hashes: Vec<U256>,
    ) -> Result<Vec<LogMemoryPageFactContinuousWithTxHash>, Error> {
        let filter = Filter::new()
            .address(self.memory_page_contract)
            .event("LogMemoryPageFactContinuous(bytes32,uint256,uint256)");

        let mut from = eth_from.saturating_sub(LOG_SEARCH_STEP);
        let mut to = eth_from;

        let mut match_pages_hashes = Vec::new();

        loop {
            if to == 0 {
                return Err(Error::BadStarknetFact);
            }
            let filter = filter.clone().from_block(from).to_block(to);

            let logs = self.get_logs_retry(&filter).await?;
            let mut memory_pages_hashes = Vec::new();

            for l in logs.iter() {
                let raw_log = RawLog::from(l.clone());
                let log_pages_fact_continuous =
                    <LogMemoryPageFactContinuous as EthLogDecode>::decode_log(&raw_log).map_err(Error::from)?;

                let pages_hashes_len = target_memory_pages_hashes.len();

                // If we find that the memory hash exists in target_memory_pages_hashes, it means we have found the
                // memory hash. We then remove it from target_memory_pages_hashes. When all elements
                // in target_memory_pages_hashes are removed, it signifies that all targets have
                // been found.
                target_memory_pages_hashes.retain(|&elem| elem != log_pages_fact_continuous.memory_hash);

                // If the length of target_memory_pages_hashes changes, it indicates that
                // `log_pages_fact_continuous` is the data we need.
                if pages_hashes_len != target_memory_pages_hashes.len() {
                    memory_pages_hashes.push(LogMemoryPageFactContinuousWithTxHash {
                        log_memory_page_fact_continuous: log_pages_fact_continuous,
                        tx_hash: l.transaction_hash.ok_or(Error::EmptyValue)?,
                    })
                }
            }
            match_pages_hashes.push(memory_pages_hashes);

            if target_memory_pages_hashes.is_empty() {
                break;
            }

            to = from;
            from -= LOG_SEARCH_STEP;
        }

        match_pages_hashes.reverse();

        Ok(match_pages_hashes.into_iter().flatten().collect())
    }

    /// Queries and decodes a transaction input data to obtain state diff information.
    ///
    /// # Arguments
    ///
    /// * `hash` - The hash of the transaction.
    ///
    /// # Returns
    ///
    /// A `Result` containing the decoded state diff data or an `Error` if the query fails.
    pub async fn query_and_decode_transaction(&mut self, hash: H256) -> Result<Vec<U256>, Error> {
        let tx = self.http_provider.get_transaction(hash).await.map_err(Error::from)?.ok_or(Error::EmptyValue)?;

        let abi = BaseContract::from(
            parse_abi(&["function registerContinuousMemoryPage(uint256 startAddr,uint256[] values,uint256 z,uint256 \
                         alpha,uint256 prime)"])
            .unwrap(),
        );

        let (_, data, _, _, _): (U256, Vec<U256>, U256, U256, U256) =
            abi.decode("registerContinuousMemoryPage", tx.input.as_ref()).unwrap();
        Ok(data)
    }

    /// Decodes the StarkNet state diff from raw data.
    ///
    /// # Arguments
    /// * `starknet_block_number` - The starknet block number.
    /// * `data` - The raw data containing the state diff information.
    /// * `client` - The StarkNet runtime client.
    ///
    /// # Returns
    ///
    /// A `Result` containing the decoded `StateDiff` or an `Error` if decoding fails.
    pub fn decode_state_diff<B, C>(
        &self,
        starknet_block_number: u64,
        data: Vec<U256>,
        client: Arc<C>,
    ) -> Result<StateDiff, Error>
    where
        B: BlockT,
        C: ProvideRuntimeApi<B> + HeaderBackend<B>,
        C::Api: StarknetRuntimeApi<B>,
    {
        if starknet_block_number < self.v011_diff_format_height {
            let with_constructor_args = starknet_block_number < self.constructor_args_diff_height;
            parser::decode_pre_011_diff(&data, with_constructor_args)
        } else {
            let parent_block_hash = client
                .block_hash_from_id(&BlockId::Number((starknet_block_number as u32).saturating_sub(1).into()))
                .map_err(|_| Error::UnknownBlock)?
                .unwrap_or_default();

            // decode v0.11 state diff basic parent block.
            parser::decode_011_diff(&data, parent_block_hash, client)
        }
    }

    /// Queries Ethereum state differences for a specific StarkNet state update.
    ///
    /// # Arguments
    ///
    /// * `state_update` - The state update containing Ethereum origin and log state update data.
    /// * `client` - The StarkNet runtime client.
    ///
    /// # Returns
    ///
    /// A `Result` containing the fetched `FetchState` or an `Error` if the query fails.
    pub async fn query_state_diff<B, C>(
        &mut self,
        state_update: &StateUpdate,
        client: Arc<C>,
    ) -> Result<FetchState, Error>
    where
        B: BlockT,
        C: ProvideRuntimeApi<B> + HeaderBackend<B>,
        C::Api: StarknetRuntimeApi<B>,
    {
        debug!(target: LOG_TARGET,"~~ query state diff for starknet block {:#?}", state_update.l2_state.block_number);

        let fact = self
            .query_state_transition_fact(
                state_update.l1_tx_info.block_number,
                state_update.l1_tx_info.transaction_index,
            )
            .await?;

        let pages_hashes = self.find_memory_pages_hashes(state_update.l1_tx_info.block_number, fact).await?;

        let pages_hashes = pages_hashes.pages_hashes.iter().map(|data| U256::from_big_endian(data)).collect::<Vec<_>>();

        let continuous_logs_with_tx_hash =
            self.find_memory_page_fact_continuous_logs(state_update.l1_tx_info.block_number, pages_hashes).await?;

        let mut tx_input_data = Vec::new();

        for log in &continuous_logs_with_tx_hash[1..] {
            debug!(target: LOG_TARGET,"~~ decode state diff from tx: {:#?}", log.tx_hash);

            let mut data = self.query_and_decode_transaction(log.tx_hash).await?;
            tx_input_data.append(&mut data)
        }

        let state_diff = self.decode_state_diff(state_update.l2_state.block_number.as_u64(), tx_input_data, client)?;

        debug!(target: LOG_TARGET,"~~ decode state diff for starknet block {}", state_update.l2_state.block_number.as_u64());

        Ok(FetchState {
            l1_l2_block_mapping: L1L2BlockMapping {
                l1_block_hash: state_update.l1_tx_info.block_hash,
                l1_block_number: state_update.l1_tx_info.block_number,
                l2_block_hash: u256_to_h256(state_update.l2_state.block_hash),
                l2_block_number: state_update.l2_state.block_number.as_u64(),
            },
            post_state_root: state_update.l2_state.global_root,
            state_diff,
        })
    }
}

pub(crate) fn u256_to_h256(u256: U256) -> H256 {
    let mut bytes = [0; 32];
    u256.to_big_endian(&mut bytes);
    let mut h256_bytes = [0; 32];
    h256_bytes.copy_from_slice(&bytes[..32]);
    H256::from(h256_bytes)
}

#[async_trait]
impl<P: JsonRpcClient + Clone> StateFetcher for EthereumStateFetcher<P> {
    async fn state_diff<B, C>(&mut self, l1_from: u64, l2_start: u64, client: Arc<C>) -> Result<Vec<FetchState>, Error>
    where
        B: BlockT,
        C: ProvideRuntimeApi<B> + HeaderBackend<B>,
        C::Api: StarknetRuntimeApi<B>,
    {
        info!(target: LOG_TARGET, "~~ begin query state_diff. From L1: {}, From L2: {}", l1_from, l2_start);

        let state_updates = self.query_state_update(l1_from, l2_start).await?;
        let tasks = state_updates.iter().map(|updates| {
            debug!(target: LOG_TARGET, "crate task fro update l1:{} l2: {}", updates.l1_tx_info.block_number, updates.l2_state.block_number);
            let client_clone = client.clone();
            let mut fetcher = self.clone();
            async move { fetcher.query_state_diff(updates, client_clone).await }
        });

        futures::future::join_all(tasks).await.into_iter().collect()
    }

    async fn get_highest_block_number(&mut self) -> Result<u64, Error> {
        let highest_eth_block_number = self.http_provider.get_block_number().await.map_err(Error::from)?.as_u64();
        Ok(highest_eth_block_number)
    }
}
