use std::num::NonZeroU128;
use std::ops::DerefMut;
use std::sync::Arc;

use ethers::providers::Middleware;
use futures::lock::Mutex;
use mc_eth_client::config::EthereumClientConfig;
use mp_starknet_inherent::L1GasPrices;

pub async fn run_worker(gas_price: Arc<Mutex<L1GasPrices>>) {
    let mut gas_price = gas_price.lock().await;
    gas_price.eth_l1_gas_price = NonZeroU128::new(12).unwrap();
    gas_price.eth_l1_data_gas_price = NonZeroU128::new(13).unwrap();
    gas_price.strk_l1_gas_price = NonZeroU128::new(14).unwrap();
    gas_price.strk_l1_data_gas_price = NonZeroU128::new(15).unwrap();
}
