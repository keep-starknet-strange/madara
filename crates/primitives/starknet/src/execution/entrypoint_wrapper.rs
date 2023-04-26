use blockifier::execution::errors::EntryPointExecutionError;
use sp_core::{ConstU32, H256};
use starknet_api::api_core::EntryPointSelector;
use starknet_api::deprecated_contract_class::{EntryPoint, EntryPointOffset, EntryPointType};
use starknet_api::hash::StarkFelt;
use starknet_api::StarknetApiError;

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

// Regular implementation.
impl EntryPointTypeWrapper {
    /// Convert Madara entrypoint type to Starknet entrypoint type.
    pub fn to_starknet(&self) -> EntryPointType {
        match self {
            Self::Constructor => EntryPointType::Constructor,
            Self::External => EntryPointType::External,
            Self::L1Handler => EntryPointType::L1Handler,
        }
    }
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
    /// The entrypoint selector
    pub entrypoint_selector: H256,
    /// The entrypoint offset
    pub entrypoint_offset: u128,
}

// Regular implementation.
impl EntryPointWrapper {
    /// Creates a new instance of an entrypoint.
    pub fn new(entrypoint_selector: H256, entrypoint_offset: u128) -> Self {
        Self { entrypoint_selector, entrypoint_offset }
    }
}

// Traits implementation.

impl From<EntryPoint> for EntryPointWrapper {
    fn from(entry_point: EntryPoint) -> Self {
        Self {
            entrypoint_selector: H256::from_slice(entry_point.selector.0.bytes()),
            entrypoint_offset: entry_point.offset.0 as u128,
        }
    }
}

impl From<EntryPointWrapper> for EntryPoint {
    fn from(entry_point: EntryPointWrapper) -> Self {
        Self {
            selector: EntryPointSelector(StarkFelt(entry_point.entrypoint_selector.to_fixed_bytes())),
            offset: EntryPointOffset(entry_point.entrypoint_offset as usize),
        }
    }
}

/// Wrapper type for transaction execution error.
#[derive(Debug)]
pub enum EntryPointExecutionErrorWrapper {
    /// Transaction execution error.
    EntryPointExecution(EntryPointExecutionError),
    /// Starknet API error.
    StarknetApi(StarknetApiError),
    /// Block context serialization error.
    BlockContextSerializationError,
}
