use mp_transactions::execution::ExecutionConfig;
use sp_core::Get;

use crate::Config;

/// Builder pattern for [`ExecutionConfig`]. Combines the
/// execution configuration from the runtime with the possible
/// flags for each transaction mode (e.g. normal, estimate fee, simulate, ...).
pub struct RuntimeExecutionConfigBuilder(ExecutionConfig);

impl Default for RuntimeExecutionConfigBuilder {
    fn default() -> Self {
        Self(ExecutionConfig {
            is_query: false,
            disable_transaction_fee: false,
            disable_fee_charge: false,
            disable_nonce_validation: false,
            disable_validation: false,
        })
    }
}

impl RuntimeExecutionConfigBuilder {
    pub fn with_query_mode(mut self) -> Self {
        self.0.is_query = true;
        self
    }
    pub fn with_transaction_fee_disabled(mut self) -> Self {
        self.0.disable_transaction_fee = true;
        self
    }
    pub fn with_fee_charge_disabled(mut self) -> Self {
        self.0.disable_fee_charge = true;
        self
    }
    pub fn with_nonce_validation_disabled(mut self) -> Self {
        self.0.disable_nonce_validation = true;
        self
    }
    pub fn with_validation_disabled(mut self) -> Self {
        self.0.disable_validation = true;
        self
    }

    /// Builds the [`ExecutionConfig`] from the current set
    /// of configuration flags and the runtime configuration.
    #[must_use]
    pub fn build<T: Config>(self) -> ExecutionConfig {
        let mut execution_config = self.0;
        execution_config.disable_transaction_fee |= T::DisableTransactionFee::get();
        execution_config.disable_nonce_validation |= T::DisableNonceValidation::get();
        execution_config
    }
}
