use mp_simulations::SimulationFlags;
use mp_transactions::execution::ExecutionConfig;
use sp_core::Get;

use crate::Config;

/// Builder pattern for [`ExecutionConfig`]. Combines the
/// execution configuration from the runtime with the possible
/// flags for each transaction mode (e.g. normal, estimate fee, simulate, ...).
pub struct RuntimeExecutionConfigBuilder(ExecutionConfig);

impl RuntimeExecutionConfigBuilder {
    pub fn new<T: Config>() -> Self {
        Self(ExecutionConfig {
            is_query: false,
            disable_fee_charge: false,
            disable_validation: false,
            disable_nonce_validation: T::DisableNonceValidation::get(),
            disable_transaction_fee: T::DisableTransactionFee::get(),
            offset_version: false,
        })
    }
    #[must_use]
    pub fn with_query_mode(mut self) -> Self {
        self.0.is_query = true;
        self
    }
    #[must_use]
    pub fn with_simulation_mode(mut self, simulation_flags: &SimulationFlags) -> Self {
        self.0.disable_fee_charge = simulation_flags.skip_fee_charge;
        self.0.disable_validation = simulation_flags.skip_validate;
        self
    }
    #[must_use]
    pub fn with_disable_nonce_validation(mut self) -> Self {
        self.0.disable_nonce_validation = true;
        self
    }
    #[must_use]
    pub fn with_offset_version(mut self) -> Self {
        self.0.offset_version = true;
        self
    }

    pub fn build(self) -> ExecutionConfig {
        self.0
    }
}
