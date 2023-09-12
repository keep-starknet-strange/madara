use mp_starknet::sequencer_address;
use mp_starknet::transaction::types::{Transaction, TxType, TransactionReceiptWrapper, EventWrapper};
use pathfinder_lib::state::block_hash::{TransactionCommitmentFinalHashType, calculate_transaction_commitment, calculate_event_commitment};
use sp_core::{U256, ConstU32};
use mp_starknet::execution::types::{ Felt252Wrapper, ContractAddressWrapper };
use mp_starknet::block::{Block, Header, MaxTransactions};
use reqwest::header::{HeaderMap, CONTENT_TYPE};
use serde_json::json;
use sp_core::bounded_vec::BoundedVec;
use starknet_api::core::ChainId;
use starknet_client::RetryConfig;
use starknet_client::reader::{StarknetFeederGatewayClient, StarknetReader};
use starknet_gateway_types::reply::{MaybePendingBlock, transaction as EnumTransaction};
use transactions::{deploy_account_tx_to_starknet_tx, declare_tx_to_starknet_tx, invoke_tx_to_starknet_tx, l1handler_tx_to_starknet_tx};
use std::sync::{mpsc, Arc, Mutex};
use std::collections::VecDeque;
use std::thread;
use log::info;
use pathfinder_common::{BlockId};
// use crate::test_utils::retry::get_test_config;
use tokio::time;

use mockito::mock;
use starknet_api::block::BlockNumber;
use std::env;
use std::fs::read_to_string;
use std::path::Path;
use std::string::String;
use starknet_client;
use std::path::PathBuf;
use validator::Validate;

pub fn read_resource_file(path_in_resource_dir: &str) -> String {
    let path = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
        .join(path_in_resource_dir);
    println!("this is path: {:?}", path);
    return read_to_string(path.to_str().unwrap()).unwrap();
}

const NODE_VERSION: &str = "NODE VERSION";
const BLOCK_NUMBER_QUERY: &str = "blockNumber";

mod transactions;
// Your block queue type
pub type BlockQueue = Arc<Mutex<VecDeque<Block>>>;

// Function to create a new block queue
pub fn create_block_queue() -> BlockQueue {
    Arc::new(Mutex::new(VecDeque::new()))
}

// This function converts a block received from the gateway into a StarkNet block
pub fn get_header(block: starknet_client::reader::Block) -> Header  {
    let parent_block_hash = Felt252Wrapper::try_from(block.parent_block_hash.0.bytes());
    let block_number = block.block_number.0;
    let global_state_root = Felt252Wrapper::try_from(block.state_root.0.bytes());
    let sequencer_address = ContractAddressWrapper::default();
    let block_timestamp = block.timestamp.0;
    let transaction_count = block.transactions.len() as u128;
    let transaction_commitment = Felt252Wrapper::default();
    let event_count = block.transaction_receipts.len() as u128;
    let event_commitment = Felt252Wrapper::default();   
    let protocol_version = Some(0u8);
    let extra_data: U256 = Felt252Wrapper::try_from(block.block_hash.0.bytes()).unwrap().into();
    let starknet_header = Header::new(
        parent_block_hash.unwrap(),
        block_number.into(),
        global_state_root.unwrap(),
        sequencer_address,
        block_timestamp,
        transaction_count,
        transaction_commitment,
        event_count,
        event_commitment,
        protocol_version.unwrap(),
        Some(extra_data),
    );
    starknet_header
}

// This function converts a block received from the gateway into a StarkNet block
pub fn from_gateway_to_starknet_block(block: starknet_client::reader::Block) -> Block {
    let transactions_vec: BoundedVec<Transaction, MaxTransactions> = BoundedVec::new();
    let transaction_receipts_vec: BoundedVec<TransactionReceiptWrapper, MaxTransactions> = BoundedVec::new();
    Block::new(
        get_header(block.clone()),
        transactions_vec,
        transaction_receipts_vec
    )
}


// pub fn process_blocks(queue: BlockQueue) -> mpsc::Sender<Block> {
//     let (sender, receiver) = mpsc::channel();
//     let thread_queue = Arc::clone(&queue);

//     thread::spawn(move || {
//         while let Ok(block) = receiver.recv() {
//             let mut queue_lock = thread_queue.lock().unwrap();
//             queue_lock.push_back(block);
//         }
//     });

//     sender
// }

async fn call_rpc(rpc_port: u16) -> Result<reqwest::StatusCode, reqwest::Error> {
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());

    let url = format!("http://localhost:{}/", rpc_port);
    let payload = json!({
        "id": 1,
        "jsonrpc": "2.0",
        "method": "engine_createBlock",
        "params": [true, true, null]
    });

    let response = client.post(url)
        .headers(headers.clone())
        .json(&payload)
        .send().await?;

    if response.status().is_success() {
        println!("RPC call succeeded.");
    } else {
        println!("RPC call failed with status: {}", response.status());
    }

    Ok(response.status())
}

// Fetching blocks from gateway
// pub async fn fetch_block(queue: BlockQueue, rpc_port: u16) {
//     let client: Client = Client::mainnet();
//     let mut i = 0u64;

//     loop {
//         let result = client.block(BlockId::Number(BlockNumber::new_or_panic(i).into())).await;
//         match result {
//             Ok(maybe_pending_block) => {
//                 let starknet_block = from_gateway_to_starknet_block(maybe_pending_block);
//                 // Lock the mutex, push to the queue, and then immediately unlock
//                 {
//                     let mut queue_guard = queue.lock().unwrap();
//                     queue_guard.push_back(starknet_block);
//                 } // MutexGuard is dropped here
//                 match call_rpc(rpc_port).await {
//                     Ok(status) => {
//                         if status.is_success() {
//                             info!("[👽] Block #{} synced correctly", i);
//                             i += 1;
//                         }
//                     },
//                     Err(e) => {
//                         eprintln!("Error processing RPC call: {:?}", e);
//                         // You could also add a delay here if needed
//                     }
//                 }

//             },
//             Err(error) => {
//                 eprintln!("Error retrieving block: {:?}", error);
//                 time::sleep(time::Duration::from_secs(2)).await;
//             }
//         }
//     }
// }

const DEFAULT_CONFIG_FILE: &str = "config/execution_config/default_config.json";

pub struct ExecutionConfig {
    pub config_file_name: PathBuf,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        ExecutionConfig { config_file_name: PathBuf::from(DEFAULT_CONFIG_FILE) }
    }
}

pub struct RpcConfig {
    // #[validate(custom = "validate_ascii")]
    pub chain_id: ChainId,
    pub server_address: String,
    pub max_events_chunk_size: usize,
    pub max_events_keys: usize,
    pub collect_metrics: bool,
    pub starknet_url: String,
    pub starknet_gateway_retry_config: RetryConfig,
    pub execution_config: ExecutionConfig,
}

impl Default for RpcConfig {
    fn default() -> Self {
        RpcConfig {
            chain_id: ChainId("SN_MAIN".to_string()),
            server_address: String::from("0.0.0.0:9944"),
            max_events_chunk_size: 1000,
            max_events_keys: 100,
            collect_metrics: false,
            starknet_url: String::from("https://alpha-mainnet.starknet.io/"),
            starknet_gateway_retry_config: RetryConfig {
                retry_base_millis: 50,
                retry_max_delay_millis: 1000,
                max_retries: 5,
            },
            execution_config: ExecutionConfig::default(),
        }
    }
}

pub async fn fetch_block(queue: BlockQueue, rpc_port: u16) {
    let rpc_config = RpcConfig::default();

    let retry_config = RetryConfig {
        retry_base_millis: 30,
        retry_max_delay_millis: 30000,
        max_retries: 10,
    };

    let starknet_client = StarknetFeederGatewayClient::new(
        &rpc_config.starknet_url,
        None, // This assumes the second parameter remains as None, adjust if otherwise.
        NODE_VERSION,
        retry_config
    ).unwrap();
    let mut i = 0u64;
    // If this raw_block is only for mocking purposes, consider removing it.
    let raw_block = read_resource_file("/Users/antiyro/Documents/Projet/Kasar/deoxys/crates/client/deoxys/src/block.json");
    loop {
        // No mock creation here, directly fetch the block from the Starknet client
        let block = starknet_client.block(BlockNumber(i)).await;
        println!("{:?}", block);
        match block {
            Ok(block) => {
                let starknet_block = from_gateway_to_starknet_block(block.unwrap());
                println!("maybe_pending_block: {:?}", starknet_block);
                // Lock the mutex, push to the queue, and then immediately unlock
                {
                    let mut queue_guard: std::sync::MutexGuard<'_, VecDeque<Block>> = queue.lock().unwrap();
                    queue_guard.push_back(starknet_block);
                } // MutexGuard is dropped here
                match call_rpc(rpc_port).await {
                    Ok(status) => {
                        if status.is_success() {
                            info!("[👽] Block #{} synced correctly", i);
                            i += 1;
                        }
                    },
                    Err(e) => {
                        eprintln!("Error processing RPC call: {:?}", e);
                        // You could also add a delay here if needed
                    }
                }
            },
            Err(error) => {
                eprintln!("Error retrieving block: {:?}", error);
                time::sleep(time::Duration::from_secs(2)).await;
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use log::info;
    use tokio;
    use std::env;
    use env_logger::Env;
    use pathfinder_common::{BlockId, BlockNumber};
    // use starknet_gateway_client::{Client, GatewayApi};

    // This async test verifies the from_gateway_to_starknet_block function
    // #[tokio::test]
    // async fn test_from_gateway_to_starknet_block() {
    //     let client: Client = Client::mainnet();
    //     // let result = client.block(BlockId::Latest).await;
    //     let result = client.block(BlockId::Number(BlockNumber::new_or_panic(10u64).into())).await;

    //     match result {
    //         Ok(maybe_pending_block) => {
    //             let starknet_block = from_gateway_to_starknet_block(maybe_pending_block);
    //             println!("Block retrieved: {:?}", starknet_block);
    //         },
    //         Err(error) => {
    //             eprintln!("Error retrieving block: {:?}", error);
    //         }
    //     }
    // }

    // This async test verifies the process_blocks function
    #[tokio::test]
    async fn test_process_block() {
        let _m = mockito::mock("GET", "/feeder_gateway/get_block?BLOCK_NUMBER_QUERY=0")
            .with_status(200)
            .with_body(&read_resource_file("src/block.json"))
            .create();

        // Define the queue and port
        let queue = create_block_queue();
        let rpc_port = 9944; // Replace with the desired port

        fetch_block(queue, rpc_port).await;

        _m.assert();
    }

    // // This async test verifies the fetch_block function
    // #[tokio::test]
    // async fn test_fetch_block() {
    //     use std::sync::{Arc, Mutex};
    //     use std::collections::VecDeque;
    //     use std::env;
    //     use env_logger::Env;

    //     env::set_var("RUST_LOG", "info");
    //     env_logger::init_from_env(Env::default().default_filter_or("info"));

    //     let queue: Arc<Mutex<VecDeque<Block>>> = Arc::new(Mutex::new(VecDeque::new()));
    //     let rpc_port = 9944;
    //     fetch_block_v2(queue, rpc_port).await;
    // }

    // use super::*;
    // use mockito;
    // use tokio::runtime::Runtime;

    // fn read_resource_file(_filename: &str) -> String {
    //     // Just a stub. You'd ideally return the actual file's contents here.
    //     r#"{"key": "value"}"#.to_string()
    // }

    #[tokio::test]
    async fn test_fetch_block_v2() {
    
    // Define the queue and port
        let queue: Arc<Mutex<VecDeque<starknet_client::reader::Block>>> = create_block_queue();
        let rpc_port = 9944; // Replace with the desired port

        fetch_block(queue, rpc_port).await;
    }


}
