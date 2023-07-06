mod sharp_utils;

use std::collections::HashMap;
use std::marker::PhantomData;
use std::str::FromStr;
use std::sync::Arc;

use ethers::prelude::abigen;
use ethers::providers::{Http, Provider};
use ethers::types::{Address, U256};
use futures::StreamExt;
use lazy_static::lazy_static;
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
                    log::info!("NONCE: {:?}", prefix);
                    log::info!("KEY: {:?}", key);
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
    pub async fn update_state(client: Arc<C>, madara_backend: Arc<mc_db::Backend<B>>) {
        let mut notification_st = client.import_notification_stream();

        while let Some(notification) = notification_st.next().await {
            // Query last proven block
            // let res = madara_backend.da().last_proved_block().unwrap();
            starknet_last_proven_block().await;
            // log::info!("Last proved block: {}", res);

            // Check the associated job status
            if let Ok(job_resp) = sharp_utils::get_status(sharp_utils::TEST_JOB_ID) {
                // TODO: use fact db enum type
                if let Some(status) = job_resp.status {
                    if status == "ONCHAIN" {
                        // Fetch DA Facts for block
                        let _res = madara_backend.da().state_diff(&notification.hash).unwrap();
                    }
                }
            }
        }
    }
}

// async fn publish_data(sender_id: &[u8], state_diff: Vec<String>) {
//     abigen!(
//         STARKNET,
//         r#"[
//             function updateState(uint256[] calldata programOutput, uint256 onchainDataHash,
// uint256 onchainDataSize) external         ]"#,
//     );

// const RPC_URL: &str = "https://eth-mainnet.g.alchemy.com/v2/<TODO: config>";
// pub const STARKNET_MAINNET_CC_ADDRESS: &str = "0xc662c410C0ECf747543f5bA90660f6ABeBD9C8c4";
// pub const STARKNET_GOERLI_CC_ADDRESS: &str = "0xde29d060D45901Fb19ED6C6e959EB22d8626708e";

// let provider = Provider::<Http>::try_from(RPC_URL).unwrap();
// let client = Arc::new(provider);

// let address: Address = STARKNET_MAINNET_CC_ADDRESS.parse().unwrap();
// let signer = Arc::new(SignerMiddleware::new(provider,
// from_wallet.with_chain_id(anvil.chain_id()))); let contract = STARKNET::new(address, client,
// signer);

// let tx = contract.update_state(state_diff, U256::default(), U256::default());
// let pending_tx = tx.send().await.unwrap();
// let _minted_tx = pending_tx.await.unwrap();
// log::info!("State Update: {pending_tx:?}");
// }

pub async fn starknet_last_proven_block() {
    abigen!(
        STARKNET,
        r#"[
            function stateBlockNumber() external view returns (int256)
        ]"#,
    );

    const RPC_URL: &str = "https://eth-mainnet.g.alchemy.com/v2/<TODO: config>";
    pub const STARKNET_MAINNET_CC_ADDRESS: &str = "0xc662c410C0ECf747543f5bA90660f6ABeBD9C8c4";
    // pub const STARKNET_GOERLI_CC_ADDRESS: &str = "0xde29d060D45901Fb19ED6C6e959EB22d8626708e";

    let provider = Provider::<Http>::try_from(RPC_URL).unwrap();
    let client = Arc::new(provider);

    let address: Address = STARKNET_MAINNET_CC_ADDRESS.parse().unwrap();
    let contract = STARKNET::new(address, client);
    if let Ok(state_block_number) = contract.state_block_number().call().await {
        log::info!("State Block Number {state_block_number:?}");
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

// #[test]
// fn decode_raw_change_set() {
//     let mut nonces: HashMap<&[u8], &[u8]> = HashMap::new();
//     let mut storage_diffs: HashMap<&[u8], StorageWrites> = HashMap::new();
//     let test_val: &[u8] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
//     nonces.push(())
// }
