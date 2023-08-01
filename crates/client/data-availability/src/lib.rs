mod ethereum;
mod sharp_utils;

use std::marker::PhantomData;
use std::str::FromStr;
use std::sync::Arc;
use std::path::Path;
use std::fs::File;
use std::io::{BufReader, Read};

use cairo_vm::cairo_run::{cairo_run, CairoRunConfig};
use cairo_vm::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::BuiltinHintProcessor;
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
            // Locate and encode the storage change
            for _event in storage_event.changes.iter() {
                // TODO:
                // - Encode the storage events for the block in a manner the Starknet OS understands and can take as input
            }

            // Run the StarkNet OS + Submit PIE(blocked):
            // - https://github.com/lambdaclass/cairo-vm/issues/1305
            let mut hint_processor = BuiltinHintProcessor::new_empty();
            // TODO: add hint processor for OS Hints

            let os_file = File::open(Path::new("cairo-contracts/build/os_compiled.json")).unwrap();
            let mut reader = BufReader::new(os_file);
            let mut buffer = Vec::<u8>::new();
            reader.read_to_end(&mut buffer).unwrap();

            let run_output = cairo_run(
                &buffer,
                &CairoRunConfig {
                    layout: "starknet_with_keccak",
                    ..Default::default()
                },
                &mut hint_processor,
            ).unwrap();
            log::error!("OS RUN: {:?}", run_output.0);

            // Store the DA output from the SN OS
            if let Err(db_err) = madara_backend.da().store_state_diff(&storage_event.block, Vec::new()) {
                log::error!("db err: {db_err}");
            };

            // Submit the StarkNet OS PIE
            if let Ok(job_resp) = sharp_utils::submit_pie(sharp_utils::TEST_CAIRO_PIE_BASE64) {
                log::info!("Job Submitted: {}", job_resp.cairo_job_key);
                // Store the cairo job key
                let _res = madara_backend
                    .da()
                    .update_cairo_job(&storage_event.block, Uuid::from_str(sharp_utils::TEST_JOB_ID).unwrap());
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
    pub async fn update_state(client: Arc<C>, madara_backend: Arc<mc_db::Backend<B>>, l1_node: String) {
        let mut notification_st = client.import_notification_stream();

        while let Some(notification) = notification_st.next().await {
            // Query last proven block
            if let Ok(last_block) = ethereum::last_proven_block(&l1_node).await {
                match madara_backend.da().last_proved_block() {
                    Ok(last_local_block) => log::info!("Last onchain: {last_block}, Last Local: {last_local_block}"),
                    Err(e) => log::debug!("could not pull last local block: {e}"),
                };
            }

            // Check the associated job status
            if let Ok(job_resp) = sharp_utils::get_status(sharp_utils::TEST_JOB_ID) {
                if let Some(status) = job_resp.status {
                    if status == "ONCHAIN" {
                        match madara_backend.da().state_diff(&notification.hash) {
                            Ok(state_diff) => {
                                // publish state diff to Layer 1
                                ethereum::publish_data(&l1_node, &DEFAULT_SEQUENCER_ADDRESS, state_diff).await;

                                // save last proven block
                                if let Err(db_err) = madara_backend.da().update_last_proved_block(&notification.hash) {
                                    log::debug!("could not save last proved block: {db_err}");
                                };
                            }
                            Err(e) => log::debug!("could not pull state diff: {e}"),
                        }
                    }
                }
            }
        }
    }
}
