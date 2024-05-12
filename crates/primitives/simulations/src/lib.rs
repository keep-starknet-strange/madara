use blockifier::state::errors::StateError;
use blockifier::transaction::errors::{TransactionExecutionError, TransactionFeeError};
use blockifier::transaction::objects::{FeeType, TransactionExecutionInfo};
use starknet_core::types::{PriceUnit, SimulationFlag, SimulationFlagForEstimateFee};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
pub enum SimulationError {
    ContractNotFound,
    TransactionExecutionFailed(String),
    MissingL1GasUsage,
    StateDiff,
    EstimateFeeFailed(String),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
pub enum InternalSubstrateError {
    FailedToCreateATransactionalStorageExecution,
}

impl From<TransactionExecutionError> for SimulationError {
    fn from(e: TransactionExecutionError) -> SimulationError {
        SimulationError::TransactionExecutionFailed(e.to_string())
    }
}

impl From<TransactionFeeError> for SimulationError {
    fn from(e: TransactionFeeError) -> SimulationError {
        SimulationError::EstimateFeeFailed(e.to_string())
    }
}

impl From<StateError> for SimulationError {
    fn from(_e: StateError) -> SimulationError {
        SimulationError::StateDiff
    }
}

pub type TransactionSimulationResult = Result<TransactionExecutionInfo, SimulationError>;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub struct SimulationFlags {
    pub validate: bool,
    pub charge_fee: bool,
}

impl From<Vec<SimulationFlag>> for SimulationFlags {
    fn from(flags: Vec<SimulationFlag>) -> Self {
        let mut flags_out = Self::default();

        for flag in flags {
            match flag {
                SimulationFlag::SkipValidate => flags_out.validate = false,
                SimulationFlag::SkipFeeCharge => flags_out.charge_fee = false,
            }
            if !flags_out.validate && !flags_out.charge_fee {
                break;
            }
        }

        flags_out
    }
}

impl From<Vec<SimulationFlagForEstimateFee>> for SimulationFlags {
    fn from(flags: Vec<SimulationFlagForEstimateFee>) -> Self {
        let mut flags_out = Self::default();

        for flag in flags {
            match flag {
                SimulationFlagForEstimateFee::SkipValidate => flags_out.validate = false,
            }
            if !flags_out.validate {
                break;
            }
        }

        // estimate_fee does not charge fees or do any balance checks
        flags_out.charge_fee = false;

        flags_out
    }
}

impl core::default::Default for SimulationFlags {
    fn default() -> Self {
        Self { validate: true, charge_fee: true }
    }
}

// We can use `FeeEstimate` from starknet-rs once we upgrade to 0.13.1
#[derive(Debug)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub struct FeeEstimate {
    pub gas_consumed: u128,
    pub gas_price: u128,
    pub data_gas_consumed: u128,
    pub data_gas_price: u128,
    pub overall_fee: u128,
    pub fee_type: FeeType,
}

impl From<&FeeEstimate> for starknet_core::types::FeeEstimate {
    fn from(fee_estimate: &FeeEstimate) -> Self {
        Self {
            gas_price: fee_estimate.gas_price.into(),
            // this is a rough estimate because in reality the gas is split into data gas
            // and execution gas. however, since we're not on 0.13.1 yet, we're using this
            gas_consumed: fee_estimate.overall_fee.saturating_div(fee_estimate.gas_price).into(),
            overall_fee: fee_estimate.overall_fee.into(),
            unit: match fee_estimate.fee_type {
                FeeType::Strk => PriceUnit::Fri,
                FeeType::Eth => PriceUnit::Wei,
            },
        }
    }
}
