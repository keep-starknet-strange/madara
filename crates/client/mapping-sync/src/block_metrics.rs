use prometheus_endpoint::prometheus::{Counter, Gauge};
use prometheus_endpoint::{register, PrometheusError, Registry};

#[derive(Clone, Debug)]
pub struct BlockMetrics {
    pub block_height: Gauge,
    pub transaction_count: Counter,
    pub event_count: Counter,
    pub l1_gas_price_wei: Gauge,
    pub l1_gas_price_strk: Gauge,
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
            l1_gas_price_wei: register(Gauge::new("madara_l1_gas_price", "Gauge for madara l1 gas price")?, registry)?,
            l1_gas_price_strk: register(
                Gauge::new("madara_l1_gas_price_strk", "Gauge for madara l1 gas price in strk")?,
                registry,
            )?,
        })
    }
}
