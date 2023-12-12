use mp_simulations::SimulationFlag;
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
        })
    }
    #[must_use]
    pub fn with_query_mode(mut self) -> Self {
        self.0.is_query = true;
        self
    }
    #[must_use]
    pub fn with_simulation_mode(mut self, simulation_flags: &[SimulationFlag]) -> Self {
        for sim in simulation_flags {
            match sim {
                SimulationFlag::SkipFeeCharge => {
                    self.0.disable_fee_charge = true;
                }
                SimulationFlag::SkipValidate => {
                    self.0.disable_validation = true;
                }
            }
            if self.0.disable_fee_charge && self.0.disable_validation {
                break;
            }
        }
        self
    }

    pub fn build(self) -> ExecutionConfig {
        self.0
    }
}
