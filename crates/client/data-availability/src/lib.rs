mod ethereum;
mod sharp_utils;

use std::collections::HashMap;
use std::marker::PhantomData;
use std::str::FromStr;
use std::sync::Arc;

use ethers::types::U256;
use futures::StreamExt;
use lazy_static::lazy_static;
use mp_starknet::sequencer_address::DEFAULT_SEQUENCER_ADDRESS;
use mp_starknet::storage::{
    PALLET_STARKNET, STARKNET_CONTRACT_CLASS, STARKNET_CONTRACT_CLASS_HASH, STARKNET_NONCE, STARKNET_STORAGE,
};
use sc_client_api::client::BlockchainEvents;
use sp_api::ProvideRuntimeApi;
use sp_io::hashing::twox_128;
use sp_runtime::traits::Block as BlockT;
use uuid::Uuid;

lazy_static! {
    static ref SN_NONCE_PREFIX: Vec<u8> = [twox_128(PALLET_STARKNET), twox_128(STARKNET_NONCE)].concat();
    static ref SN_CONTRACT_CLASS_HASH_PREFIX: Vec<u8> =
        [twox_128(PALLET_STARKNET), twox_128(STARKNET_CONTRACT_CLASS_HASH)].concat();
    static ref SN_CONTRACT_CLASS_PREFIX: Vec<u8> =
        [twox_128(PALLET_STARKNET), twox_128(STARKNET_CONTRACT_CLASS)].concat();
    static ref SN_STORAGE_PREFIX: Vec<u8> = [twox_128(PALLET_STARKNET), twox_128(STARKNET_STORAGE)].concat();
}

pub type StorageWrites<'a> = Vec<(&'a [u8], &'a [u8])>;
pub struct DataAvailabilityWorker<B, C>(PhantomData<(B, C)>);

impl<B, C> DataAvailabilityWorker<B, C>
where
    B: BlockT,
    C: ProvideRuntimeApi<B>,
    C: BlockchainEvents<B> + 'static,
{
    pub async fn prove_current_block(client: Arc<C>, madara_backend: Arc<mc_db::Backend<B>>) {
        let mut storage_event_st = client.storage_changes_notification_stream(None, None).unwrap();

        while let Some(storage_event) = storage_event_st.next().await {
            // TODO:
            // - old_declared_contracts, declared_classes, replaced_classes
            // - test deployed contracts
            let mut _deployed_contracts: Vec<String> = Vec::new();
            let mut nonces: HashMap<&[u8], &[u8]> = HashMap::new();
            let mut storage_diffs: HashMap<&[u8], StorageWrites> = HashMap::new();

            // Locate and encode the storage change
            for event in storage_event.changes.iter() {
                let mut prefix = event.1.0.as_slice();
                let mut key: &[u8] = &[];
                if prefix.len() > 32 {
                    let raw_split = prefix.split_at(32);
                    prefix = raw_split.0;
                    key = raw_split.1;
                }

                if prefix == *SN_NONCE_PREFIX {
                    if let Some(data) = event.2 {
                        nonces.insert(key, data.0.as_slice());
                    }
                }

                if prefix == *SN_STORAGE_PREFIX {
                    if let Some(data) = event.2 {
                        // first 32 bytes = contract address, second 32 bytes = storage variable
                        let write_split = key.split_at(32);

                        storage_diffs
                            .entry(write_split.0)
                            .and_modify(|v| v.push((write_split.1, data.0.as_slice())))
                            .or_insert(vec![(write_split.1, data.0.as_slice())]);
                    }
                }
            }

            let state_diff = pre_0_11_0_state_diff(storage_diffs, nonces);

            if let Err(db_err) = madara_backend.da().store_state_diff(&storage_event.block, state_diff) {
                log::error!("db err: {db_err}");
            };

            // Submit the StarkNet OS PIE
            /*if let Ok(job_resp) = sharp_utils::submit_pie(sharp_utils::TEST_CAIRO_PIE_BASE64) {
                log::info!("Job Submitted: {}", job_resp.cairo_job_key);
                // Store the cairo job key
                let _res = madara_backend
                    .da()
                    .update_cairo_job(&storage_event.block, Uuid::from_str(sharp_utils::TEST_JOB_ID).unwrap());
            }*/
        }
    }
}

impl<B, C> DataAvailabilityWorker<B, C>
where
    B: BlockT,
    C: ProvideRuntimeApi<B>,
    C: BlockchainEvents<B> + 'static,
{
    pub async fn update_state(client: Arc<C>, madara_backend: Arc<mc_db::Backend<B>>, l1_node: String) {
        let mut notification_st = client.import_notification_stream();

        while let Some(notification) = notification_st.next().await {
            // Query last proven block
            /*  if let Ok(last_block) = ethereum::last_proven_block(&l1_node).await {
                match madara_backend.da().last_proved_block() {
                    Ok(last_local_block) => log::info!("Last onchain: {last_block}, Last Local: {last_local_block}"),
                    Err(e) => log::debug!("could not pull last local block: {e}"),
                };
            }*/

            // Check the associated job status
            // remove sharp status checking, we publish anyway

            match madara_backend.da().state_diff(&notification.hash) {
                Ok(state_diff) => {
                    // publish state diff to Layer 1
                    //TODO :celestia
                    ethereum::publish_data(&l1_node, &DEFAULT_SEQUENCER_ADDRESS, state_diff).await;

                    // save last proven block
                    /*if let Err(db_err) = madara_backend.da().update_last_proved_block(&notification.hash) {
                        log::debug!("could not save last proved block: {db_err}");
                    };*/
                }
                Err(e) => log::debug!("could not pull state diff: {e}"),
            }
            
            /*if let Ok(job_resp) = sharp_utils::get_status(sharp_utils::TEST_JOB_ID) {
                if let Some(status) = job_resp.status {
                    
                }
            }*/
        }
    }
}

// encode calldata:
// - https://docs.starknet.io/documentation/architecture_and_concepts/Data_Availability/on-chain-data/#pre_v0.11.0_example
pub fn pre_0_11_0_state_diff(storage_diffs: HashMap<&[u8], StorageWrites>, nonces: HashMap<&[u8], &[u8]>) -> Vec<U256> {
    let mut state_diff: Vec<U256> = Vec::new();

    state_diff.push(U256::from(storage_diffs.len()));

    for (addr, writes) in storage_diffs {
        state_diff.push(U256::from_big_endian(addr));
        state_diff.push(U256::from(writes.len()));
        for write in writes {
            state_diff.push(U256::from_big_endian(write.0));
            state_diff.push(U256::from_big_endian(write.1));
        }
    }

    for (addr, nonce) in nonces {
        state_diff.push(U256::from_big_endian(addr));
        state_diff.push(U256::from_big_endian(nonce));
    }
    state_diff
}
