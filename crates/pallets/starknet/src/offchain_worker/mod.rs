mod types;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use frame_support::traits::OriginTrait;
use frame_system::pallet_prelude::OriginFor;
use serde_json::from_slice;
use sp_runtime::offchain::http;
use sp_runtime::offchain::storage::StorageValueRef;
pub use types::*;

use crate::message::get_messages_events;
use crate::{Config, Pallet, ETHEREUM_EXECUTION_RPC};

pub const LAST_FINALIZED_BLOCK_QUERY: &str =
    r#"{"jsonrpc": "2.0", "method": "eth_getBlockByNumber", "params": ["finalized", true], "id": 0}"#;

pub const LAST_GAS_PRICE_QUERY: &str = r#"{"jsonrpc": "2.0", "method": "eth_gasPrice", "params": [], "id": 1}"#;

impl<T: Config> Pallet<T> {
    /// Fetches L1 messages and execute them.
    /// This function is called by the offchain worker.
    /// It is executed in a separate thread.
    /// # Returns
    /// The result of the offchain worker execution.
    pub(crate) fn process_l1_messages() -> Result<(), OffchainWorkerError> {
        // Query L1 for the last finalized block.
        let raw_body = query_eth(LAST_FINALIZED_BLOCK_QUERY)?;
        let last_finalized_block: u64 = from_slice::<EthGetBlockByNumberResponse>(&raw_body)
            .map_err(|_| OffchainWorkerError::SerdeError)?
            .try_into()?;

        // Get the last known block from storage.
        let last_known_eth_block = Self::last_known_eth_block().ok_or(OffchainWorkerError::NoLastKnownEthBlock)?;

        // Check if there are new messages to be processed.
        if last_finalized_block > last_known_eth_block {
            // Read the new messages from L1.
            let raw_body = query_eth(&get_messages_events(last_known_eth_block, last_finalized_block))?;
            let res: EthLogs = from_slice(&raw_body).map_err(|_| OffchainWorkerError::SerdeError)?;
            // Iterate over the messages and execute them.
            res.result.iter().try_for_each(|message| {
                // Execute the message.
                Self::consume_l1_message(OriginFor::<T>::none(), message.try_into_transaction()?)
                    .map_err(OffchainWorkerError::ConsumeMessageError)
            })?;
        }
        Ok(())
    }

    /// Fetches L1 gas price and return the result in gwei.
    pub(crate) fn fetch_gas_price() -> Result<u128, OffchainWorkerError> {
        let raw_body = query_eth(LAST_GAS_PRICE_QUERY)?;
        let res: EthGasPriceResponse = from_slice(&raw_body).map_err(|_| OffchainWorkerError::SerdeError)?;
        let gas_price = u128::from_str_radix(&res.result[2..], 16)
            .map_err(|_| OffchainWorkerError::StringConversionError)?
            / 1_000_000_000;
        log::info!("Gas price in gwei: {}", gas_price);
        let currentgas_price = Self::gas_price_l1();
        log::info!("Current Gas price: {}", currentgas_price);

        Ok(gas_price)
    }
}

/// Returns Ethereum RPC URL from Storage
pub fn get_eth_rpc_url() -> Result<String, OffchainWorkerError> {
    let eth_execution_rpc_url = StorageValueRef::persistent(ETHEREUM_EXECUTION_RPC)
        .get::<Vec<u8>>()
        .map_err(|_| OffchainWorkerError::GetStorageFailed)?
        .ok_or(OffchainWorkerError::EthRpcNotSet)?;

    let endpoint: &str =
        core::str::from_utf8(&eth_execution_rpc_url).map_err(|_| OffchainWorkerError::FormatBytesFailed)?;

    if endpoint.is_empty() {
        return Err(OffchainWorkerError::EthRpcNotSet);
    }

    Ok(endpoint.to_string())
}

/// Queries an Eth json rpc node
fn query_eth(request: &str) -> Result<Vec<u8>, OffchainWorkerError> {
    let res = http::Request::post(&get_eth_rpc_url()?, vec![request])
        .add_header("content-type", "application/json")
        .send()
        .map_err(OffchainWorkerError::HttpError)?
        .wait()
        .map_err(OffchainWorkerError::RequestError)?;
    Ok(res.body().collect::<Vec<u8>>())
}
