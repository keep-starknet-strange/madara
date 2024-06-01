use std::num::NonZeroU128;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{format_err, Result};
use ethers::utils::__serde_json::json;
use futures::lock::Mutex;
use mc_eth_client::config::EthereumClientConfig;
use mp_starknet_inherent::L1GasPrices;
use tokio::time::sleep;

use crate::types::{EthRpcResponse, FeeHistory};

const DEFAULT_GAS_PRICE_POLL_MS: u64 = 10_000;

pub async fn run_worker(config: Arc<EthereumClientConfig>, gas_price: Arc<Mutex<L1GasPrices>>, infinite_loop: bool) {
    let rpc_endpoint = config.provider.rpc_endpoint().clone();
    let client = reqwest::Client::new();
    let poll_time = config.provider.gas_price_poll_ms().unwrap_or(DEFAULT_GAS_PRICE_POLL_MS);

    loop {
        match update_gas_price(rpc_endpoint.clone(), &client, gas_price.clone()).await {
            Ok(_) => log::trace!("Updated gas prices"),
            Err(e) => log::error!("Failed to update gas prices: {:?}", e),
        }

        let gas_price = gas_price.lock().await;
        let last_update_timestamp = gas_price.last_update_timestamp;
        drop(gas_price);
        let current_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Failed to get current timestamp")
            .as_millis();

        if current_timestamp - last_update_timestamp > 10 * poll_time as u128 {
            panic!(
                "Gas prices have not been updated for {} ms. Last update was at {}",
                current_timestamp - last_update_timestamp,
                last_update_timestamp
            );
        }

        if !infinite_loop {
            break;
        }

        sleep(Duration::from_millis(poll_time)).await;
    }
}

async fn update_gas_price(
    rpc_endpoint: String,
    client: &reqwest::Client,
    gas_price: Arc<Mutex<L1GasPrices>>,
) -> Result<()> {
    let fee_history: EthRpcResponse<FeeHistory> = client
        .post(rpc_endpoint.clone())
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "eth_feeHistory",
            "params": [300, "latest", []],
            "id": 83
        }))
        .send()
        .await?
        .json()
        .await?;

    // The RPC responds with 301 elements for some reason. It's also just safer to manually
    // take the last 300. We choose 300 to get average gas caprice for last one hour (300 * 12 sec block
    // time).
    let (_, blob_fee_history_one_hour) = fee_history
        .result
        .base_fee_per_blob_gas
        .split_at(fee_history.result.base_fee_per_blob_gas.len().max(300) - 300);

    let avg_blob_base_fee =
        blob_fee_history_one_hour.iter().map(|hex_str| u128::from_str_radix(&hex_str[2..], 16).unwrap()).sum::<u128>()
            / blob_fee_history_one_hour.len() as u128;

    let eth_gas_price = u128::from_str_radix(
        fee_history
            .result
            .base_fee_per_gas
            .last()
            .ok_or(format_err!("Failed to get last element of `base_fee_per_gas`"))?
            .trim_start_matches("0x"),
        16,
    )?;

    log::info!("avg_blob_base_fee : {:?}", avg_blob_base_fee);
    log::info!("eth_gas_price : {:?}", eth_gas_price);

    // TODO: fetch this from the oracle
    let eth_strk_price = 2425;

    let mut gas_price = gas_price.lock().await;
    gas_price.eth_l1_gas_price =
        NonZeroU128::new(eth_gas_price).ok_or(format_err!("Failed to convert `eth_gas_price` to NonZeroU128"))?;
    gas_price.eth_l1_data_gas_price = NonZeroU128::new(avg_blob_base_fee + 1)
        .ok_or(format_err!("Failed to convert `eth_l1_data_gas_price` to NonZeroU128"))?;
    gas_price.strk_l1_gas_price = NonZeroU128::new(eth_gas_price.saturating_mul(eth_strk_price))
        .ok_or(format_err!("Failed to convert `strk_l1_gas_price` to NonZeroU128"))?;
    gas_price.strk_l1_data_gas_price = NonZeroU128::new((avg_blob_base_fee + 1).saturating_mul(eth_strk_price))
        .ok_or(format_err!("Failed to convert `strk_l1_data_gas_price` to NonZeroU128"))?;
    gas_price.last_update_timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_millis();
    // explicitly dropping gas price here to avoid long waits when fetching the value
    // on the inherent side which would increase block time
    drop(gas_price);

    Ok(())
}

#[tokio::test]
async fn test_l1_gas_price() {}
