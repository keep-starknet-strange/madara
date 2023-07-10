use mp_starknet::transaction::types::{Transaction, TxType, MaxArraySize, TransactionReceiptWrapper, EventWrapper};
use pathfinder_lib::state::block_hash::{TransactionCommitmentFinalHashType, calculate_transaction_commitment, calculate_event_commitment};
use sp_core::{U256, ConstU32};
use mp_starknet::execution::types::{ Felt252Wrapper, ContractAddressWrapper, CallEntryPointWrapper, ContractClassWrapper, MaxCalldataSize, EntryPointTypeWrapper, ClassHashWrapper };
use mp_starknet::block::{Block, Header, BlockTransactions, MaxTransactions};
use reqwest::header::{HeaderMap, CONTENT_TYPE};
use serde_json::json;
use sp_core::bounded_vec::BoundedVec;
use starknet_gateway_types::reply::transaction::{DeployAccountTransaction, InvokeTransaction, L1HandlerTransaction};
use starknet_gateway_types::reply::{MaybePendingBlock, transaction as EnumTransaction, transaction::DeclareTransaction};
use transactions::{deploy_account_tx_to_starknet_tx, declare_tx_to_starknet_tx, invoke_tx_to_starknet_tx, l1handler_tx_to_starknet_tx};
use std::ops::Bound;
use std::sync::{mpsc, Arc, Mutex};
use std::collections::VecDeque;
use std::{thread};
use log::info;
use pathfinder_common::{BlockId, BlockNumber};
use starknet_gateway_client::{Client, GatewayApi};
use tokio::time;

// Your block queue type
pub type BlockQueue = Arc<Mutex<VecDeque<Block>>>;

// Function to create a new block queue
pub fn create_block_queue() -> BlockQueue {
    Arc::new(Mutex::new(VecDeque::new()))
}

pub fn process_blocks(queue: BlockQueue) -> mpsc::Sender<Block> {
    let (sender, receiver) = mpsc::channel();
    let thread_queue = Arc::clone(&queue);

    thread::spawn(move || {
        while let Ok(block) = receiver.recv() {
            let mut queue_lock = thread_queue.lock().unwrap();
            queue_lock.push_back(block);
        }
    });

    sender
}

async fn call_rpc() -> Result<reqwest::StatusCode, reqwest::Error> {
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());

    let url = "http://localhost:9944/";
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
pub async fn fetch_block(queue: BlockQueue) {
    let client: Client = Client::mainnet();
    let mut i = 0u64;

    loop {
        let result = client.block(BlockId::Number(BlockNumber::new_or_panic(i).into())).await;
        match result {
            Ok(maybe_pending_block) => {
                let starknet_block = from_gateway_to_starknet_block(maybe_pending_block);
                // Lock the mutex, push to the queue, and then immediately unlock
                {
                    let mut queue_guard = queue.lock().unwrap();
                    queue_guard.push_back(starknet_block);
                } // MutexGuard is dropped here
                match call_rpc().await {
                    Ok(status) => {
                        if status.is_success() {
                            info!("[DEOXYS] Block #{} synced correctly", i);
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
    use std::env;
    use env_logger::Env;
    use pathfinder_common::{BlockId, BlockNumber};
    use starknet_gateway_client::{Client, GatewayApi};

    // This async test verifies the process_blocks function
    #[tokio::test]
    async fn test_process_block() {
        env::set_var("RUST_LOG", "info");
        env_logger::init_from_env(Env::default().default_filter_or("info"));

        let queue = create_block_queue();
        let sender = process_blocks(queue);
        info!("Block processing thread started");

        let client: Client = Client::mainnet();

        for i in 3000..3005u64 {
            let result = client.block(BlockId::Number(BlockNumber::new_or_panic(i).into())).await;
            match result {
                Ok(maybe_pending_block) => {
                    let starknet_block = from_gateway_to_starknet_block(maybe_pending_block);
                    info!("Created block #{}", i);
                    sender.send(starknet_block).unwrap();
                    info!("Sent block #{} for processing", i);
                },
                Err(error) => {
                    eprintln!("Error retrieving block: {:?}", error);
                }
            }
        }

        info!("Waiting for blocks to be processed");
        std::thread::sleep(std::time::Duration::from_secs(5));
        info!("Test completed");
    }

    // This async test verifies the fetch_block function
    #[tokio::test]
    async fn test_fetch_block() {
        use std::sync::{Arc, Mutex};
        use std::collections::VecDeque;
        use std::env;
        use env_logger::Env;

        env::set_var("RUST_LOG", "info");
        env_logger::init_from_env(Env::default().default_filter_or("info"));

        let queue: Arc<Mutex<VecDeque<Block>>> = Arc::new(Mutex::new(VecDeque::new()));

        fetch_block(queue).await;
    }

}
