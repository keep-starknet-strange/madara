use blockifier::state::cached_state::CommitmentStateDiff;
use blockifier::state::errors::StateError;
use blockifier::transaction::errors::TransactionExecutionError;
use blockifier::transaction::objects::TransactionExecutionInfo;
use starknet_core::types::{SimulationFlag, SimulationFlagForEstimateFee};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
pub enum SimulationError {
    ContractNotFound,
    TransactionExecutionFailed(String),
    MissingL1GasUsage,
    StateDiff,
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

impl From<StateError> for SimulationError {
    fn from(_e: StateError) -> SimulationError {
        SimulationError::StateDiff
    }
}

pub type ReExecutionResult = Result<Vec<(TransactionExecutionInfo, Option<CommitmentStateDiff>)>, SimulationError>;
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
