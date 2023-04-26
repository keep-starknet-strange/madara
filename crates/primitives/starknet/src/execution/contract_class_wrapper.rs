use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::fmt::Display;

use blockifier::execution::contract_class::ContractClass;
use blockifier::execution::execution_utils::{cairo_vm_program_to_sn_api, sn_api_to_cairo_vm_program};
use cairo_vm::types::errors::program_errors::ProgramError;
use frame_support::{BoundedBTreeMap, BoundedVec};
use serde_json::{from_slice, to_string};
use sp_core::ConstU32;
use starknet_api::deprecated_contract_class::{EntryPoint, Program as DeprecatedProgram};
use starknet_api::stdlib::collections::HashMap;
use thiserror_no_std::Error;

use super::entrypoint_wrapper::{EntryPointTypeWrapper, EntryPointWrapper, MaxEntryPoints};
use super::program_wrapper::ProgramWrapper;
#[cfg(feature = "std")]
use super::{deserialize_bounded_btreemap, serialize_bounded_btreemap};

/// Max number of entrypoints types (EXTERNAL/L1_HANDLER/CONSTRUCTOR)
type MaxEntryPointsType = ConstU32<3>;

// TODO: use real value
/// Maximum size of a program
type MaxProgramSize = ConstU32<{ u32::MAX }>;

/// Contract Class type wrapper.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    Default,
    scale_codec::MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct ContractClassWrapper {
    /// Contract class program json.
    pub program: ProgramWrapper,
    /// Contract class entrypoints.
    #[cfg_attr(
        feature = "std",
        serde(deserialize_with = "deserialize_bounded_btreemap", serialize_with = "serialize_bounded_btreemap")
    )]
    pub entry_points_by_type:
        BoundedBTreeMap<EntryPointTypeWrapper, BoundedVec<EntryPointWrapper, MaxEntryPoints>, MaxEntryPointsType>,
}

// Regular implementaiton.
impl ContractClassWrapper {
    /// Creates a new instance of a contract class.
    pub fn new(
        program: ProgramWrapper,
        entry_points_by_type: BoundedBTreeMap<
            EntryPointTypeWrapper,
            BoundedVec<EntryPointWrapper, MaxEntryPoints>,
            MaxEntryPointsType,
        >,
    ) -> Self {
        Self { program, entry_points_by_type }
    }
}

/// Errors in the try_from implementation of [ContractClassWrapper]
#[derive(Debug, Error)]
pub enum ContractClassFromWrapperError {
    /// Program error.
    #[error(transparent)]
    Program(#[from] ProgramError),
    /// Serde error.
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error("something else happend")]
    Other,
}

// Traits implementation.

impl TryFrom<ContractClassWrapper> for ContractClass {
    type Error = ContractClassFromWrapperError;

    fn try_from(wrapper: ContractClassWrapper) -> Result<Self, Self::Error> {
        let mut entrypoints = HashMap::new();
        wrapper.entry_points_by_type.into_iter().for_each(|(key, val)| {
            entrypoints.insert(key.into(), val.iter().map(|entrypoint| EntryPoint::from(entrypoint.clone())).collect());
        });

        Ok(ContractClass {
            program: wrapper.program.try_into().map_err(|_| ContractClassFromWrapperError::Other)?,
            entry_points_by_type: entrypoints,
        })
    }
}

impl TryFrom<ContractClass> for ContractClassWrapper {
    type Error = ContractClassFromWrapperError;

    fn try_from(contract_class: ContractClass) -> Result<Self, Self::Error> {
        let mut entrypoints = BTreeMap::new();
        for (key, val) in contract_class.entry_points_by_type.iter() {
            entrypoints.insert(
                (*key).into(),
                BoundedVec::try_from(val.iter().map(|elt| elt.clone().into()).collect::<Vec<EntryPointWrapper>>())
                    .unwrap(),
            );
        }
        Ok(Self {
            program: contract_class.program.try_into().map_err(|_| ContractClassFromWrapperError::Other)?,
            entry_points_by_type: BoundedBTreeMap::try_from(entrypoints)
                .map_err(|_| ContractClassFromWrapperError::Other)?,
        })
    }
}
