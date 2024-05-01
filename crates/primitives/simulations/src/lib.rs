use blockifier::state::errors::StateError;
use blockifier::transaction::errors::TransactionExecutionError;
use blockifier::transaction::objects::TransactionExecutionInfo;
use starknet_core::types::{SimulationFlag, SimulationFlagForEstimateFee};

// TODO: This is a placeholder
// https://github.com/starkware-libs/starknet-specs/blob/master/api/starknet_api_openrpc.json#L3919
// The official rpc expect use to return the trace up to the point of failure.
// Figuring out how to get that is a problem for later
#[derive(Debug, Clone)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
pub enum Error {
    ContractNotFound,
    TransactionExecutionFailed(String),
    MissingL1GasUsage,
    FailedToCreateATransactionalStorageExecution,
    StateDiff,
}

impl From<TransactionExecutionError> for Error {
    fn from(e: TransactionExecutionError) -> Error {
        Error::TransactionExecutionFailed(e.to_string())
    }
}

impl From<StateError> for Error {
    fn from(_e: StateError) -> Error {
        Error::StateDiff
    }
}

pub type TransactionSimulationResult = Result<TransactionExecutionInfo, Error>;

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

        flags_out
    }
}

impl core::default::Default for SimulationFlags {
    fn default() -> Self {
        Self { validate: true, charge_fee: true }
    }
}
