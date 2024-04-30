use prometheus_endpoint::prometheus::{Counter, Gauge};
use prometheus_endpoint::{register, PrometheusError, Registry};

#[derive(Clone, Debug)]
pub struct BlockMetrics {
    pub block_height: Gauge,
    pub transaction_count: Counter,
    pub event_count: Counter,
    pub eth_l1_gas_price_wei: Gauge,
    pub strk_l1_gas_price_fri: Gauge,
    pub eth_l1_data_gas_price_wei: Gauge,
    pub strk_l1_data_gas_price_fri: Gauge,
}

impl BlockMetrics {
    pub fn register(registry: &Registry) -> Result<Self, PrometheusError> {
        Ok(Self {
            block_height: register(Gauge::new("madara_block_height", "Gauge for madara block height")?, registry)?,
            transaction_count: register(
                Counter::new("madara_transaction_count", "Counter for madara transaction count")?,
                registry,
            )?,
            event_count: register(Counter::new("madara_event_count", "Counter for madara event count")?, registry)?,
            eth_l1_gas_price_wei: register(
                Gauge::new("madara_l1_gas_price_eth", "Gauge for madara l1 gas price in eth wei")?,
                registry,
            )?,
            strk_l1_gas_price_fri: register(
                Gauge::new("madara_l1_gas_price_strk", "Gauge for madara l1 gas price in strk fri")?,
                registry,
            )?,
            eth_l1_data_gas_price_wei: register(
                Gauge::new("madara_l1_data_gas_price_eth", "Gauge for madara l1 data gas price in eth wei")?,
                registry,
            )?,
            strk_l1_data_gas_price_fri: register(
                Gauge::new("madara_l1_data_gas_price_strk", "Gauge for madara l1 data gas price in strk fri")?,
                registry,
            )?,
        })
    }
}
