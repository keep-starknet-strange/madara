#![cfg_attr(not(feature = "std"), no_std)]

#[doc(hidden)]
pub extern crate alloc;

use alloc::vec::Vec;

use blockifier::transaction::objects::TransactionExecutionInfo;
use starknet_core::types::SimulationFlag;

// TODO: This is a placeholder
// https://github.com/starkware-libs/starknet-specs/blob/master/api/starknet_api_openrpc.json#L3919
// The official rpc expect use to return the trace up to the point of failure.
// Figuring out how to get that is a problem for later
#[derive(Debug)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
pub struct PlaceHolderErrorTypeForFailedStarknetExecution;

pub type TransactionSimulationResult = Result<TransactionExecutionInfo, PlaceHolderErrorTypeForFailedStarknetExecution>;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub struct SimulationFlags {
    pub skip_validate: bool,
    pub skip_fee_charge: bool,
}

impl From<Vec<SimulationFlag>> for SimulationFlags {
    fn from(flags: Vec<SimulationFlag>) -> Self {
        let mut skip_validate = false;
        let mut skip_fee_charge = false;

        for flag in flags {
            match flag {
                SimulationFlag::SkipValidate => skip_validate = true,
                SimulationFlag::SkipFeeCharge => skip_fee_charge = true,
            }
            if skip_validate && skip_fee_charge {
                break;
            }
        }

        Self { skip_validate, skip_fee_charge }
    }
}
