use mp_starknet::transaction::types::{Transaction, TxType, TransactionReceiptWrapper, EventWrapper};
use pathfinder_lib::state::block_hash::{TransactionCommitmentFinalHashType, calculate_transaction_commitment, calculate_event_commitment};
use sp_core::{U256, ConstU32};
use mp_starknet::execution::types::{ Felt252Wrapper, ContractAddressWrapper };
use mp_starknet::block::{Block, Header, MaxTransactions};
use reqwest::header::{HeaderMap, CONTENT_TYPE};
use serde_json::json;
use sp_core::bounded_vec::BoundedVec;
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
pub type BlockQueue = Arc<Mutex<VecDeque<starknet_client::reader::Block>>>;

// Function to create a new block queue
pub fn create_block_queue() -> BlockQueue {
    Arc::new(Mutex::new(VecDeque::new()))
}

// This function converts a block received from the gateway into a StarkNet block

// pub fn from_gateway_to_starknet_block(_block: Option<Block>) -> Block {
//     match _block {
//         Some(block) => {  // <-- You should match against Some(block) to destructure the Option
//             let parent_block_hash = Felt252Wrapper::try_from(block.header().parent_block_hash.0);
//             let block_number = block.header().block_number;  // <-- Access fields of `Header` struct using `header` 
//             let global_state_root = Felt252Wrapper::try_from(block.header().global_state_root.0);
//             // println!("this is sequence address : {:?}", block.sequencer_address);
//             let sequencer_address = if let Some(sequencer_address) = block.header().sequencer_address {
//                 ContractAddressWrapper::try_from(sequencer_address.0.as_be_bytes())
//             } else { 
//                 Ok(ContractAddressWrapper::ZERO)
//             };
//             let block_timestamp =  block.header().timestamp.get();
//             let transaction_count = block.transactions.len() as u128;
//             let event_count = block.transaction_receipts
//                 .iter()
//                 .map(|receipt| receipt.events.len() as u128)
//                 .sum();
//             let transaction_final_hash_type =
//                 TransactionCommitmentFinalHashType::for_version(&block.starknet_version);
//             let transaction_commitment_from_block =
//                 calculate_transaction_commitment(&block.transactions, transaction_final_hash_type.unwrap());
//             let event_commitment_from_block = calculate_event_commitment(&block.transaction_receipts);

//             let transaction_commitment = Felt252Wrapper::try_from(transaction_commitment_from_block.unwrap().0.as_be_bytes());
//             let event_commitment = Felt252Wrapper::try_from(event_commitment_from_block.unwrap().0.as_be_bytes());
//             let protocol_version = Some(0u8);

//             let extra_data: U256 = Felt252Wrapper::try_from(block.block_hash.0.as_be_bytes()).unwrap().into();

//             let starknet_header = Header::new(
//                 parent_block_hash.unwrap(),
//                 block_number.into(),
//                 global_state_root.unwrap(),
//                 sequencer_address.unwrap(),
//                 block_timestamp,
//                 transaction_count,
//                 transaction_commitment.unwrap(),
//                 event_count,
//                 event_commitment.unwrap(),
//                 protocol_version.unwrap(),
//                 Some(extra_data),
//             );

//             // Missing attributs for DeployAccountTransaction : sender_address, call_entrypoint, contract_class,
//             // Missing attributs for Declare : version, call_entrypoint, contract_class, contract_address_salt,
//             // Missing attributs for L1Handler : signature, contract_class, contract_class_salt, max_fee
//             // Missing attributs for Invoke : version, call_entrypoint, contract_class, contract_address_salt
//             let mut transactions_vec: BoundedVec<Transaction, MaxTransactions> = BoundedVec::new();
//             for transaction in &block.transactions {
//                 match transaction {
//                     EnumTransaction::Transaction::Declare(declare_transaction) => {
//                         // convert declare_transaction to starknet transaction
//                         let tx = declare_tx_to_starknet_tx(declare_transaction.clone());
//                         transactions_vec.try_push(tx).unwrap();
//                     },
//                     EnumTransaction::Transaction::DeployAccount(deploy_account_transaction) => {
//                         // Do something with deploy_account_transaction
//                         let tx = deploy_account_tx_to_starknet_tx(deploy_account_transaction.clone());
//                         transactions_vec.try_push(tx).unwrap();
//                     },
//                     EnumTransaction::Transaction::Invoke(invoke_transaction) => {
//                         // Do something with invoke_transaction
//                         //println!("this is tx: ");
//                         let tx = invoke_tx_to_starknet_tx(invoke_transaction.clone());
//                         transactions_vec.try_push(tx).unwrap();
//                     },
//                     EnumTransaction::Transaction::L1Handler(l1handler_transaction) => {
//                         // Do something with l1handler_transaction
//                         // let tx = tx_to_starknet_tx(declare_transaction);
//                         // transactionsVec.push(tx).unwrap()
//                         let tx = l1handler_tx_to_starknet_tx(l1handler_transaction.clone());
//                         transactions_vec.try_push(tx).unwrap();
//                     },
//                     EnumTransaction::Transaction::Deploy(_deploy_transaction) => {
//                         //
//                     },
//                 }
//             }

//             let mut transaction_receipts: BoundedVec<TransactionReceiptWrapper, MaxTransactions> = BoundedVec::new();
//             let events_receipt: BoundedVec<EventWrapper, ConstU32<10000>> = BoundedVec::new();
//             for receipt in &block.transaction_receipts {
//                 let tx_receipt = TransactionReceiptWrapper {
//                     transaction_hash: Felt252Wrapper::try_from(receipt.transaction_hash.0.as_be_bytes()).unwrap(),
//                     actual_fee: Felt252Wrapper::try_from(receipt.actual_fee.unwrap().0.as_be_bytes()).unwrap(),
//                     tx_type: TxType::Declare, // !TODO not definitive

//                     // match &block.transactions.get(receipt.transaction_index.get().into()) {
//                     //     Some(EnumTransaction::DeclareTransaction) => TxType::Declare,
//                     //     Some(EnumTransaction::DeployAccountTransaction) => TxType::DeployAccount,
//                     //     Some(EnumTransaction::InvokeTransaction) => TxType::Invoke,
//                     //     Some(EnumTransaction::DeployTransaction) => TxType::DeployAccount,
//                     //     Some(EnumTransaction::L1HandlerTransaction) => TxType::L1Handler,
//                     //     None => todo!(),
//                     // },

//                     //block_number: block.block_number.get(),
//                     //block_hash: Felt252Wrapper::try_from(block.block_hash.0.as_be_bytes()).unwrap(),
//                     events: events_receipt.clone(),
//                 };
//                 match transaction_receipts.try_push(tx_receipt) {
//                     Ok(_) => (),
//                     Err(_) => {
//                         println!("Error: transaction_receipts is full");
//                         break;
//                     },
//                 }
//             }

//             Block::new(
//                 starknet_header,
//                 transactions_vec,
//                 transaction_receipts
//             )
//         },
//         MaybePendingBlock::Pending(_pending_blockk) => {todo!()},
//     }
// }


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

pub async fn fetch_block(queue: BlockQueue, rpc_port: u16) {
    let retry_config = RetryConfig {
        retry_base_millis: 30,
        retry_max_delay_millis: 30000,
        max_retries: 10,
    };

    let starknet_client = StarknetFeederGatewayClient::new(
        &mockito::server_url(), // Adjust this as needed
        None,
        NODE_VERSION,
        retry_config
    )
    .unwrap();

    let mut i = 0u64;

    loop {
        let block_query = format!("/feeder_gateway/get_block?{BLOCK_NUMBER_QUERY}={}", i);
        let result = starknet_client.block(BlockNumber(i)).await;
        match result {
            Ok(maybe_pending_block) => {
                let starknet_block = maybe_pending_block.unwrap();
                // Lock the mutex, push to the queue, and then immediately unlock
                {
                    let mut queue_guard: std::sync::MutexGuard<'_, VecDeque<starknet_client::reader::Block>> = queue.lock().unwrap();
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
            .with_body(&read_resource_file("./block.json"))
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


}
