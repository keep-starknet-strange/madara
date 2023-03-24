//! Starknet execution functionality.

use alloc::sync::Arc;
use alloc::vec;

use blockifier::execution::entry_point::CallEntryPoint;
use frame_support::BoundedVec;
use sp_core::{ConstU32, H256};
use starknet_api::api_core::{ClassHash, ContractAddress, EntryPointSelector};
use starknet_api::hash::StarkFelt;
use starknet_api::state::EntryPointType;
use starknet_api::transaction::Calldata;

/// The address of a contract.
pub type ContractAddressWrapper = [u8; 32];

type MaxCalldataSize = ConstU32<4294967295>;
/// Wrapper type for class hash field.
pub type ClassHashWrapper = [u8; 32];

/// Enum that represents all the entrypoints types.
#[derive(Clone, Debug, PartialEq, Eq, codec::Encode, codec::Decode, scale_info::TypeInfo, codec::MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum EntryPointTypeWrapper {
    /// Constructor.
    Constructor,
    /// External.
    External,
    /// L1 Handler.
    L1Handler,
}

/// Representation of a Starknet transaction.
#[derive(Clone, Debug, PartialEq, Eq, codec::Encode, codec::Decode, scale_info::TypeInfo, codec::MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct CallEntryPointWrapper {
    /// The class hash
    pub class_hash: Option<ClassHashWrapper>,
    /// The entrypoint type
    pub entrypoint_type: EntryPointTypeWrapper,
    /// The entrypoint selector
    /// An invoke transaction without an entry point selector invokes the 'execute' function.
    pub entrypoint_selector: Option<H256>,
    /// The Calldata
    pub calldata: BoundedVec<H256, MaxCalldataSize>,
    /// The storage address
    pub storage_address: ContractAddressWrapper,
    /// The caller address
    pub caller_address: ContractAddressWrapper,
}
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

impl CallEntryPointWrapper {
    /// Creates a new instance of a call entrypoint.
    pub fn new(
        class_hash: Option<ClassHashWrapper>,
        entrypoint_type: EntryPointTypeWrapper,
        entrypoint_selector: Option<H256>,
        calldata: BoundedVec<H256, MaxCalldataSize>,
        storage_address: ContractAddressWrapper,
        caller_address: ContractAddressWrapper,
    ) -> Self {
        Self { class_hash, entrypoint_type, entrypoint_selector, calldata, storage_address, caller_address }
    }

    /// Convert to Starknet CallEntryPoint
    pub fn to_starknet_call_entry_point(&self) -> CallEntryPoint {
        let class_hash = self.class_hash.map(|class_hash| ClassHash(StarkFelt::new(class_hash).unwrap()));
        CallEntryPoint {
            class_hash,
            entry_point_type: self.entrypoint_type.to_starknet(),
            entry_point_selector: EntryPointSelector(
                StarkFelt::new(self.entrypoint_selector.unwrap_or_default().0).unwrap(),
            ),
            calldata: Calldata(Arc::new(
                self.calldata
                    .clone()
                    .into_inner()
                    .iter()
                    .map(|x| StarkFelt::new(*(*x).as_fixed_bytes()).unwrap())
                    .collect(),
            )),
            storage_address: ContractAddress::try_from(StarkFelt::new(self.storage_address).unwrap()).unwrap(),
            caller_address: ContractAddress::try_from(StarkFelt::new(self.caller_address).unwrap()).unwrap(),
        }
    }
}
impl Default for CallEntryPointWrapper {
    fn default() -> Self {
        Self {
            class_hash: Some(ClassHashWrapper::default()),
            entrypoint_type: EntryPointTypeWrapper::External,
            entrypoint_selector: Some(H256::default()),
            calldata: BoundedVec::try_from(vec![H256::zero(); 32]).unwrap(),
            storage_address: ContractAddressWrapper::default(),
            caller_address: ContractAddressWrapper::default(),
        }
    }
}
