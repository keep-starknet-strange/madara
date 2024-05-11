use std::collections::HashMap;
use std::num::NonZeroU128;
use std::ops::{DerefMut, Mul};
use std::sync::Arc;
use std::time::Duration;

use ethers::prelude::{Http, Middleware, Provider};
use ethers::utils::__serde_json::json;
use futures::lock::Mutex;
use futures::stream::iter;
use mc_eth_client::config::EthereumClientConfig;
use mp_starknet_inherent::L1GasPrices;
use tokio::time::sleep;

use crate::types::{EthRpcResponse, FeeHistory};

const DEFAULT_GAS_PRICE_POLL_MS: u64 = 10_000;

pub async fn run_worker(config: Arc<EthereumClientConfig>, gas_price: Arc<Mutex<L1GasPrices>>, infinite_loop: bool) {
    let rpc_endpoint = config.provider.rpc_endpoint().clone();
    let provider: Provider<Http> = config.provider.clone().try_into().unwrap();
    let client = reqwest::Client::new();
    let poll_time = config.provider.gas_price_poll_ms().unwrap_or(DEFAULT_GAS_PRICE_POLL_MS);

    loop {
        let eth_gas_price = provider.get_gas_price().await.unwrap().try_into().unwrap();

        let fee_history: EthRpcResponse<FeeHistory> = client
            .post(rpc_endpoint.clone())
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "eth_feeHistory",
                "params": [300, "latest", []],
                "id": 83
            }))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        // The RPC responds with 301 elements for some reason. It's also just safer to manually
        // take the last 300. We choose 300 to get average gas caprice for last one hour (300 * 12 sec block
        // time).
        let (_, blob_fee_history_one_hour) =
            fee_history.result.base_fee_per_blob_gas.split_at(fee_history.result.base_fee_per_blob_gas.len() - 300);

        let avg_blob_base_fee =
            blob_fee_history_one_hour.iter().sum::<u128>() / blob_fee_history_one_hour.len() as u128;

        // TODO: fetch this from the oracle
        let eth_strk_price = 2425;

        let mut gas_price = gas_price.lock().await;
        gas_price.eth_l1_gas_price = NonZeroU128::new(eth_gas_price).unwrap();
        gas_price.eth_l1_data_gas_price = NonZeroU128::new(avg_blob_base_fee).unwrap();
        gas_price.strk_l1_gas_price = NonZeroU128::new(eth_gas_price.saturating_mul(eth_strk_price)).unwrap();
        gas_price.strk_l1_data_gas_price = NonZeroU128::new(avg_blob_base_fee.saturating_mul(eth_strk_price)).unwrap();
        // explicitly dropping gas price here to avoid long waits when fetching the value
        // on the inherent side which would increase block time
        drop(gas_price);

        if !infinite_loop {
            break;
        }

        sleep(Duration::from_millis(poll_time)).await;
    }
}
