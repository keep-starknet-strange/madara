use mp_felt::Felt252Wrapper;
use reqwest::StatusCode;
use sp_core::U256;
use mp_block::{Block, Header };
use reqwest::header::{HeaderMap, CONTENT_TYPE, self, HeaderValue};
use serde_json::{json, Value};
use sp_core::bounded_vec::BoundedVec;
use starknet_api::block::BlockNumber;
use starknet_api::api_core::{ChainId, PatriciaKey};
use starknet_api::transaction::Event;
use starknet_client::RetryConfig;
use starknet_client::reader::{StarknetFeederGatewayClient, StarknetReader};
use starknet_core::chain_id;
use starknet_ff::FieldElement;
use std::sync::{ Arc, Mutex};
use std::collections::VecDeque;
use log::info;
use tokio::time;
use std::env;
use std::fs::read_to_string;
use std::path::Path;
use std::string::String;
use starknet_client;
use std::path::PathBuf;
use crate::transactions::{declare_tx_to_starknet_tx, deploy_account_tx_to_starknet_tx, invoke_tx_to_starknet_tx, l1handler_tx_to_starknet_tx, deploy_tx_to_starknet_tx, convert_to_contract_class};

pub fn read_resource_file(path_in_resource_dir: &str) -> String {
    let path = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
        .join(path_in_resource_dir);
    return read_to_string(path.to_str().unwrap()).unwrap();
}

const NODE_VERSION: &str = "NODE VERSION";

mod transactions;
// Your block queue type
pub type BlockQueue = Arc<Mutex<VecDeque<Block>>>;

// Function to create a new block queue
pub fn create_block_queue() -> BlockQueue {
    Arc::new(Mutex::new(VecDeque::new()))
}


// This function converts a block received from the gateway into a StarkNet block
pub fn get_header(block: starknet_client::reader::Block, transactions: mp_block::BlockTransactions, events:) -> mp_block::Header  {
    let parent_block_hash = Felt252Wrapper::try_from(block.parent_block_hash.0.bytes());
    let block_number = block.block_number.0;
    let global_state_root = Felt252Wrapper::try_from(block.state_root.0.bytes());
    let status = match block.status {
        starknet_client::reader::objects::block::BlockStatus::Pending => starknet_core::types::BlockStatus::Pending,
        starknet_client::reader::objects::block::BlockStatus::AcceptedOnL2 => starknet_core::types::BlockStatus::AcceptedOnL2,
        starknet_client::reader::objects::block::BlockStatus::AcceptedOnL1 => starknet_core::types::BlockStatus::AcceptedOnL1,
        starknet_client::reader::objects::block::BlockStatus::Reverted => starknet_core::types::BlockStatus::Rejected,
        starknet_client::reader::objects::block::BlockStatus::Aborted => starknet_core::types::BlockStatus::Rejected,
    };
    let sequencer_address = Felt252Wrapper(FieldElement::from(*PatriciaKey::key(&block.sequencer_address.0)));
    let block_timestamp = block.timestamp.0;
    let transaction_count = block.transactions.len() as u128;
    let (transaction_commitment, event_commitment) =
            mp_commitments::calculate_commitments::<T::SystemHash>(&transactions, &events, Felt252Wrapper("SN_MAIN"));
    let event_count: u128 = block.transaction_receipts
                .iter()
                .map(|receipt| receipt.events.len() as u128)
                .sum();
    let protocol_version = Some(0u8);
    let extra_data: U256 = Felt252Wrapper::try_from(block.block_hash.0.bytes()).unwrap().into();
    let starknet_header = mp_block::Header::new(
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

pub async fn get_txs(block: starknet_client::reader::Block) -> mp_block::BlockTransactions {
    let mut transactions_vec: mp_block::BlockTransactions;
        for transaction in &block.transactions {
            match transaction {
                starknet_client::reader::objects::transaction::Transaction::Declare(declare_transaction) => {
                    // convert declare_transaction to starknet transaction
                    let tx = declare_tx_to_starknet_tx(declare_transaction.clone());
                    transactions_vec.push(tx.unwrap());
                },
                starknet_client::reader::objects::transaction::Transaction::DeployAccount(deploy_account_transaction) => {
                    // convert declare_transaction to starknet transaction
                    let tx = deploy_account_tx_to_starknet_tx(deploy_account_transaction.clone().into());
                    transactions_vec.push(tx.unwrap());
                },
                starknet_client::reader::objects::transaction::Transaction::Deploy(deploy_transaction) => {
                    // convert declare_transaction to starknet transaction
                    let tx = deploy_tx_to_starknet_tx(deploy_transaction.clone()).await;
                    transactions_vec.push(tx.unwrap());
                },
                starknet_client::reader::objects::transaction::Transaction::Invoke(invoke_transaction) => {
                    // convert invoke_transaction to starknet transaction
                    let tx = invoke_tx_to_starknet_tx(invoke_transaction.clone());
                    transactions_vec.push(tx.unwrap());
                },
                starknet_client::reader::objects::transaction::Transaction::L1Handler(l1handler_transaction) => {
                    // convert declare_transaction to starknet transaction
                    let tx = l1handler_tx_to_starknet_tx(l1handler_transaction.clone().into());
                    transactions_vec.push(tx.unwrap());
                },
            }
        }

    transactions_vec
}

fn convert_event_to_wrapper(event: &Event) -> EventWrapper {
    let mut keys_vec: BoundedVec<Felt252Wrapper, MaxArraySize> = BoundedVec::new();
    for key in &event.content.keys {
        keys_vec.try_push(Felt252Wrapper(key.0.into())).unwrap();
    }

    let mut data_vec: BoundedVec<Felt252Wrapper, MaxArraySize> = BoundedVec::new();
    for data_item in &event.content.data.0 {
        data_vec.try_push(Felt252Wrapper::from(FieldElement::from(*data_item))).unwrap();
    }

    let from_address_wrapper: ContractAddressWrapper = Felt252Wrapper::from(*PatriciaKey::key(&event.from_address.0));
    
    EventWrapper {
        keys: keys_vec,
        data: data_vec,
        from_address: from_address_wrapper,
    }
}

pub fn get_txs_receipts(block: starknet_client::reader::Block, transactions: BoundedVec<Transaction, MaxTransactions>) -> BoundedVec<TransactionReceiptWrapper, MaxTransactions> {
    let mut transaction_receipts_vec: BoundedVec<TransactionReceiptWrapper, MaxTransactions> = BoundedVec::new();

    for (transaction, transaction_receipt) in transactions.iter().zip(&block.transaction_receipts) {
        let mut events_vec: BoundedVec<EventWrapper, MaxArraySize> = BoundedVec::new();
        for event in &transaction_receipt.events {
            let event_wrapper = convert_event_to_wrapper(event);
            events_vec.try_push(event_wrapper).unwrap();
        }

        let transaction_receipt_wrapper = TransactionReceiptWrapper {
            transaction_hash: Felt252Wrapper(transaction_receipt.transaction_hash.0.into()),
            actual_fee: Felt252Wrapper::from(transaction_receipt.actual_fee.0),
            tx_type: transaction.tx_type.clone(),
            events: events_vec,
        };

        transaction_receipts_vec.try_push(transaction_receipt_wrapper).unwrap();
    }

    transaction_receipts_vec
}

// This function converts a block received from the gateway into a StarkNet block
pub async fn from_gateway_to_starknet_block(block: starknet_client::reader::Block) -> mp_block::Block {
    let transactions_vec: mp_block::BlockTransactions = get_txs(block.clone()).await;
    let transaction_receipts_vec: mp::BlockTransactions = get_txs_receipts(block.clone(), transactions_vec.clone());
    let header = get_header(block.clone(), transactions_vec.clone(), transaction_receipts_vec.clone().envents);
    mp_block::Block::new(
        header,
        transactions_vec,
    )
}

async fn create_block(rpc_port: u16) -> Result<StatusCode, reqwest::Error> {
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "application/json ".parse().unwrap());

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

    Ok(response.status())
}

async fn get_last_synced_block(rpc_port: u16) -> Result<Option<u64>, reqwest::Error> {
    let client = reqwest::Client::new();
    let headers = {
        let mut h = HeaderMap::new();
        h.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        h
    };

    let url = format!("http://localhost:{}/", rpc_port);
    let payload = json!({
        "id": 1,
        "jsonrpc": "2.0",
        "method": "chain_getBlock",
        "params": []
    });

    let response = client.post(&url)
        .headers(headers)
        .json(&payload)
        .send().await?;

    let body: Value = response.json().await?;
    
    body["result"]["block"]["header"]["number"]
        .as_str()
        .and_then(|number_hex| u64::from_str_radix(&number_hex[2..], 16).ok())
        .map_or(Ok(None), |number_decimal| Ok(Some(number_decimal)))
}

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
        None,
        NODE_VERSION,
        retry_config
    ).unwrap();

    let mut i = get_last_synced_block(rpc_port).await.unwrap().unwrap() + 1;
    loop {
        let block = starknet_client.block(BlockNumber(i)).await;
        match block {
            Ok(block) => {
                let starknet_block = from_gateway_to_starknet_block(block.unwrap()).await;
                {
                    let mut queue_guard: std::sync::MutexGuard<'_, VecDeque<Block>> = queue.lock().unwrap();
                    queue_guard.push_back(starknet_block);
                }
                match create_block(rpc_port).await {
                    Ok(status) => {
                        if status.is_success() {
                            info!("[👽] Block #{} synced correctly", i);
                            i += 1;
                        }
                    },
                    Err(e) => {
                        eprintln!("Error processing RPC call: {:?}", e);
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
    use std::sync::Mutex;
    use std::collections::VecDeque;
    use mockall::mock;
    use mockito::mock;

    // Mocking StarknetFeederGatewayClient for testing
    mock! {
        StarknetFeederGatewayClient {
            fn new(url: &str, option: Option<&str>, version: &str, retry_config: RetryConfig) -> Self;
            async fn block(&self, block_number: BlockNumber) -> Result<Option<starknet_client::reader::Block>, starknet_client::Error>;
        }
    }

    #[test]
    fn test_read_resource_file() {
        // This test can check if the function properly reads files from a resource directory.
        // For simplicity, you can skip the actual file reading and just check if the path formation is correct.
    }

    #[test]
    fn test_get_header() {
        // Provide a mock starknet_client::reader::Block and BoundedVec<Transaction, MaxTransactions>
        // Then, call get_header with the mock data and check if the resulting Header is as expected.
    }

    #[test]
    fn test_get_txs() {
        // Provide a mock starknet_client::reader::Block
        // Call get_txs with the mock data and verify if the resulting BoundedVec<Transaction, MaxTransactions> is correct.
    }

    #[tokio::test]
    async fn test_from_gateway_to_starknet_block() {
        // Provide a mock starknet_client::reader::Block
        // Call from_gateway_to_starknet_block and verify the resulting Block.
    }

    #[tokio::test]
    async fn test_create_block() {
        // This test can check if an RPC call is made properly.
        // It can also verify if the function handles different response statuses correctly.
    }

    #[tokio::test]
    async fn test_fetch_block_success() {
        // Mock the StarknetFeederGatewayClient to return a successful block.
        // Run fetch_block and ensure the block is correctly added to the queue and RPC call is made.
    }

    #[tokio::test]
    async fn test_fetch_block_error() {
        // Mock the StarknetFeederGatewayClient to return an error.
        // Run fetch_block and ensure the error is handled correctly.
    }

    // ... More tests for other functions and scenarios ...
}
