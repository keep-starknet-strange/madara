pub mod avail;
pub mod celestia;
pub mod ethereum;
mod sharp;
pub mod utils;

use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use ethers::types::{I256, U256};
use futures::StreamExt;
use indexmap::{IndexMap, IndexSet};
use mp_storage::{
    PALLET_STARKNET, STARKNET_CONTRACT_CLASS, STARKNET_CONTRACT_CLASS_HASH, STARKNET_NONCE, STARKNET_STORAGE,
};
use sc_client_api::client::BlockchainEvents;
use sc_client_api::StorageKey as SubStorageKey;
use serde::Deserialize;
use sp_api::ProvideRuntimeApi;
use sp_io::hashing::twox_128;
use sp_runtime::traits::Block as BlockT;
use starknet_api::api_core::{ClassHash, CompiledClassHash, ContractAddress, Nonce};
use starknet_api::state::{StorageKey, ThinStateDiff};
use utils::{bytes_to_felt, bytes_to_key, safe_split, state_diff_to_calldata};

pub struct DataAvailabilityWorker<B, C>(PhantomData<(B, C)>);

#[derive(Debug, Copy, Clone, PartialEq, clap::ValueEnum)]
pub enum DaLayer {
    Celestia,
    Ethereum,
    Avail,
}

/// Data availability modes in which Madara can be initialized.
///
/// Default only mode currently implemented is Sovereign.
#[derive(Debug, Copy, Clone, PartialEq, Deserialize, Default)]
pub enum DaMode {
    /// Full Validity Rollup
    ///
    /// Generates a Cairo execution trace of the StarknetOS
    /// run for the given block as it is applied to the current Madara state.
    /// Once this execution trace is proved to the L1 Verifier(i.e. [Ethereum](https://goerli.etherscan.io/address/0x8f97970aC5a9aa8D130d35146F5b59c4aef57963))
    /// the relevant [state diff](https://docs.starknet.io/documentation/architecture_and_concepts/Network_Architecture/on-chain-data) can be written and validated against the on-chain
    /// proof verification of the block propogation.
    #[serde(rename = "validity")]
    Validity,
    /// Hybrid Volition
    ///
    /// Volitions allow applications and users to interoperate between on-chain data and off-chain
    /// da. Although full specs are not currently available, this mode will entail generating
    /// a StarknetOS execution trace for data elected to be on-chain and interaction w/ the prover
    /// will be necessary.
    #[serde(rename = "volition")]
    Volition,
    /// Sovereign Rollup
    ///
    /// Sovereign state diffs are untethered to an accompanying validity proof therefore
    /// they can simply be published to any da solution available. As this solution does not
    /// require an execution trace to be proved we can simply parse the state diff from the
    /// storage changes of the block.
    #[serde(rename = "sovereign")]
    #[default]
    Sovereign,
}

#[async_trait]
pub trait DaClient: Send + Sync {
    fn get_mode(&self) -> DaMode;
    async fn last_published_state(&self) -> Result<I256>;
    async fn publish_state_diff(&self, state_diff: Vec<U256>) -> Result<()>;
}

impl<B, C> DataAvailabilityWorker<B, C>
where
    B: BlockT,
    C: ProvideRuntimeApi<B>,
    C: BlockchainEvents<B> + 'static,
{
    pub async fn prove_current_block(da_mode: DaMode, client: Arc<C>, madara_backend: Arc<mc_db::Backend<B>>) {
        let pallet_starknet_key = twox_128(PALLET_STARKNET);

        let mut storage_event_st = client
            .storage_changes_notification_stream(None, Some(&[(SubStorageKey(pallet_starknet_key.into()), None)]))
            .expect("node has been initialized to prove state change, but can't read from notification stream");

        while let Some(storage_event) = storage_event_st.next().await {
            let mut accessed_addrs: IndexSet<ContractAddress> = IndexSet::new();
            let mut state_diff = ThinStateDiff {
                declared_classes: IndexMap::new(),
                storage_diffs: IndexMap::new(),
                nonces: IndexMap::new(),
                deployed_contracts: IndexMap::new(),
                deprecated_declared_classes: Vec::new(),
                replaced_classes: IndexMap::new(),
            };

            for (_, storage_key, storage_val) in storage_event.changes.iter() {
                // split storage key into the (starknet prefix) and (remaining tree path)
                let (child_key, rest_key) = safe_split(&storage_key.0);

                // saftey checks
                let storage_val = match storage_val {
                    Some(x) => x.0.clone(),
                    None => continue,
                };
                let rest_key = match rest_key {
                    Some(x) => x,
                    None => continue,
                };

                if child_key == twox_128(STARKNET_NONCE) {
                    // collect nonce information in state diff
                    state_diff
                        .nonces
                        .insert(ContractAddress(bytes_to_key(&rest_key)), Nonce(bytes_to_felt(&storage_val)));
                    accessed_addrs.insert(ContractAddress(bytes_to_key(&rest_key)));
                } else if child_key == twox_128(STARKNET_STORAGE) {
                    // collect storage update information in state diff
                    if rest_key.len() > 32 {
                        let (addr, key) = rest_key.split_at(32);
                        let (addr, key) = (bytes_to_key(addr), bytes_to_key(key));

                        state_diff
                            .storage_diffs
                            .entry(ContractAddress(addr))
                            .and_modify(|v| {
                                v.insert(StorageKey(key), bytes_to_felt(&storage_val));
                            })
                            .or_insert(IndexMap::from([(StorageKey(key), bytes_to_felt(&storage_val))]));
                        accessed_addrs.insert(ContractAddress(addr));
                    }
                } else if child_key == twox_128(STARKNET_CONTRACT_CLASS) {
                    // collect declared class information in state diff
                    state_diff
                        .declared_classes
                        .insert(ClassHash(bytes_to_felt(&rest_key)), CompiledClassHash(bytes_to_felt(&storage_val)));
                } else if child_key == twox_128(STARKNET_CONTRACT_CLASS_HASH) {
                    // collect deployed contract information in state diff
                    state_diff
                        .deployed_contracts
                        .insert(ContractAddress(bytes_to_key(&rest_key)), ClassHash(bytes_to_felt(&storage_val)));
                    accessed_addrs.insert(ContractAddress(bytes_to_key(&rest_key)));
                }
            }

            // store the da encoded calldata for the state update worker
            if let Err(db_err) = madara_backend
                .da()
                .store_state_diff(&storage_event.block, state_diff_to_calldata(state_diff, accessed_addrs.len()))
            {
                log::error!("db err: {db_err}");
            };

            match da_mode {
                DaMode::Validity => {
                    // TODO:
                    // - run the StarknetOs for this block
                    // - parse the PIE to `submit_pie` and zip/base64 internal
                    if let Ok(job_resp) = sharp::submit_pie("TODO") {
                        log::info!("Job Submitted: {}", job_resp.cairo_job_key);
                        // Store the cairo job key
                        if let Err(db_err) =
                            madara_backend.da().update_cairo_job(&storage_event.block, job_resp.cairo_job_key)
                        {
                            log::error!("db err: {db_err}");
                        };
                    }
                }
                _ => {
                    log::info!("don't prove in remaining DA modes")
                }
            }
        }
    }
}

impl<B, C> DataAvailabilityWorker<B, C>
where
    B: BlockT,
    C: ProvideRuntimeApi<B>,
    C: BlockchainEvents<B> + 'static,
{
    pub async fn update_state(
        da_client: Box<dyn DaClient + Send + Sync>,
        client: Arc<C>,
        madara_backend: Arc<mc_db::Backend<B>>,
    ) {
        let mut notification_st = client.import_notification_stream();

        while let Some(notification) = notification_st.next().await {
            // Query last written state
            // TODO: this value will be used to ensure the correct state diff is being written in Validity mode
            let _last_published_state = match da_client.last_published_state().await {
                Ok(last_published_state) => last_published_state,
                Err(e) => {
                    log::error!("da provider error: {e}");
                    continue;
                }
            };

            match da_client.get_mode() {
                DaMode::Validity => {
                    // Check the SHARP status of last_proved + 1
                    // Write the publish state diff of last_proved + 1
                    log::info!("validity da mode not implemented");
                }
                DaMode::Sovereign => match madara_backend.da().state_diff(&notification.hash) {
                    Ok(state_diff) => {
                        if let Err(e) = da_client.publish_state_diff(state_diff).await {
                            log::error!("DA PUBLISH ERROR: {}", e);
                        }
                    }
                    Err(e) => log::error!("could not pull state diff: {e}"),
                },
                DaMode::Volition => log::info!("volition da mode not implemented"),
            }
        }
    }
}
