use prometheus_endpoint::{register, Histogram, HistogramOpts, PrometheusError, Registry};

#[derive(Clone, Debug)]
pub struct DaMetrics {
    pub state_updates: Histogram,
    pub state_proofs: Histogram,
}

impl DaMetrics {
    pub fn register(registry: &Registry) -> Result<Self, PrometheusError> {
        Ok(Self {
            state_updates: register(
                Histogram::with_opts(HistogramOpts::new(
                    "madara_da_state_updates",
                    "Histogram of time taken for state updates",
                ))?,
                registry,
            )?,
            state_proofs: register(
                Histogram::with_opts(HistogramOpts::new(
                    "madara_da_state_proofs",
                    "Histogram of time taken for state proofs",
                ))?,
                registry,
            )?,
        })
    }
}
