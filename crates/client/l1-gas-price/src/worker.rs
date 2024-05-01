use std::sync::atomic::AtomicU128;
use std::sync::Arc;

use ethers::prelude::{Http, Provider};
use ethers::providers::Middleware;
use mc_eth_client::config::EthereumClientConfig;

pub async fn run_worker(config: EthereumClientConfig, global_gas_price: Arc<AtomicU128>) {
    let provider: Provider<Http> = config.provider.try_into().unwrap();
    let gas_price = provider.get_gas_price().await.unwrap();
    let x: u128 = gas_price.try_into().unwrap();
    global_gas_price.store(x, std::sync::atomic::Ordering::Relaxed);
}
