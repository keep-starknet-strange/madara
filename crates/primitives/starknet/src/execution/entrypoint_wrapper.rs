use blockifier::execution::errors::EntryPointExecutionError;
use sp_core::ConstU32;
use starknet_api::api_core::EntryPointSelector;
use starknet_api::deprecated_contract_class::{EntryPoint, EntryPointOffset, EntryPointType};
use starknet_api::hash::StarkFelt;
use starknet_api::StarknetApiError;
#[cfg(feature = "std")]
use starknet_core::types::LegacyContractEntryPoint;
use thiserror_no_std::Error;

/// Max number of entrypoints.
pub type MaxEntryPoints = ConstU32<4294967295>;

/// Wrapper type for transaction execution result.
pub type EntryPointExecutionResultWrapper<T> = Result<T, EntryPointExecutionErrorWrapper>;

/// Enum that represents all the entrypoints types.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
    PartialOrd,
    Ord,
    Hash,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum EntryPointTypeWrapper {
    /// Constructor.
    Constructor,
    /// External.
    External,
    /// L1 Handler.
    L1Handler,
}

// Traits implementation.
impl From<EntryPointType> for EntryPointTypeWrapper {
    fn from(entry_point_type: EntryPointType) -> Self {
        match entry_point_type {
            EntryPointType::Constructor => EntryPointTypeWrapper::Constructor,
            EntryPointType::External => EntryPointTypeWrapper::External,
            EntryPointType::L1Handler => EntryPointTypeWrapper::L1Handler,
        }
    }
}

impl From<EntryPointTypeWrapper> for EntryPointType {
    fn from(entrypoint: EntryPointTypeWrapper) -> Self {
        match entrypoint {
            EntryPointTypeWrapper::Constructor => EntryPointType::Constructor,
            EntryPointTypeWrapper::External => EntryPointType::External,
            EntryPointTypeWrapper::L1Handler => EntryPointType::L1Handler,
        }
    }
}

/// Representation of a Starknet Entry Point.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
    PartialOrd,
    Ord,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct EntryPointWrapper {
    /// The entrypoint offset
    pub offset: u128,
    /// The entrypoint selector
    pub selector: [u8; 32],
}

// Regular implementation.
impl EntryPointWrapper {
    /// Creates a new instance of an entrypoint.
    pub fn new(selector: [u8; 32], offset: u128) -> Self {
        Self { selector, offset }
    }
}

// Traits implementation.

impl From<EntryPoint> for EntryPointWrapper {
    fn from(entry_point: EntryPoint) -> Self {
        Self { selector: entry_point.selector.0.0, offset: entry_point.offset.0 as u128 }
    }
}

impl From<EntryPointWrapper> for EntryPoint {
    fn from(entry_point: EntryPointWrapper) -> Self {
        Self {
            selector: EntryPointSelector(StarkFelt(entry_point.selector)),
            offset: EntryPointOffset(entry_point.offset as usize),
        }
    }
}

#[cfg(feature = "std")]
impl From<LegacyContractEntryPoint> for EntryPointWrapper {
    fn from(value: LegacyContractEntryPoint) -> Self {
        let selector = value.selector.to_bytes_be();
        let offset = value.offset.into();
        Self { selector, offset }
    }
}

/// Wrapper type for transaction execution error.
#[derive(Debug, Error)]
pub enum EntryPointExecutionErrorWrapper {
    /// Transaction execution error.
    #[error(transparent)]
    EntryPointExecution(#[from] EntryPointExecutionError),
    /// Starknet API error.
    #[error(transparent)]
    StarknetApi(#[from] StarknetApiError),
    /// Block context serialization error.
    #[error("Block context serialization error")]
    BlockContextSerializationError,
}
