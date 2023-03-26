//! Starknet execution functionality.

use starknet_api::stdlib::collections::HashMap;

use alloc::sync::Arc;
use alloc::vec;

use blockifier::execution::entry_point::CallEntryPoint;
use frame_support::BoundedVec;
use sp_core::{ConstU32, H256, U256};
use starknet_api::api_core::{ClassHash, ContractAddress, EntryPointSelector};
use starknet_api::hash::StarkFelt;
use starknet_api::state::{EntryPointType, EntryPoint, EntryPointOffset, Program, ContractClassAbiEntry, ContractClass};
use starknet_api::transaction::Calldata;

/// The address of a contract.
pub type ContractAddressWrapper = [u8; 32];

/// Maximum vector sizes.
type MaxCalldataSize = ConstU32<4294967295>;
type MaxAbiSize = ConstU32<4294967295>;
type MaxEntryPoints = ConstU32<4294967295>;

/// Wrapper type for class hash field.
pub type ClassHashWrapper = [u8; 32];

/// Contract Class
#[derive(Clone, Debug, PartialEq, Eq, codec::Encode, codec::Decode, scale_info::TypeInfo, codec::MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct ContractClassWrapper {
	/// Contract class id.
	pub program: Program,
	/// Contract class name.
	pub abi: BoundedVec<ContractClassAbiEntry, MaxAbiSize>,
	/// Contract class code.
	pub entry_points_by_type: HashMap<EntryPointTypeWrapper, BoundedVec<EntryPoint, MaxEntryPoints>>,
}

impl ContractClassWrapper {
	/// Creates a new instance of a contract class.
	pub fn new(program: Program, abi: BoundedVec<ContractClassAbiEntry, MaxAbiSize>, entry_points_by_type: HashMap<EntryPointTypeWrapper, BoundedVec<EntryPoint, MaxEntryPoints>>) -> Self {
		Self { program, abi, entry_points_by_type }
	}

	/// Convert to starknet contract class.
	pub fn to_starknet_contract_class(&self) -> ContractClass {
		ContractClass {
			program: self.program.clone(),
			abi: Some(self.abi.clone().to_vec()),
			entry_points_by_type: self.entry_points_by_type.iter().map(|(k, v)| (k.to_starknet(), v.to_vec())).collect(),
		}
	}
}

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

// pub enum ContractClassAbiEntryWrapper {
// 	/// An event abi entry.
//     Event(EventAbiEntry),
//     /// A function abi entry.
//     Function(FunctionAbiEntryWithType),
//     /// A struct abi entry.
//     Struct(StructAbiEntry),
// }

/// Representation of a Starknet Entry Point.
#[derive(Clone, Debug, PartialEq, Eq, codec::Encode, codec::Decode, scale_info::TypeInfo, codec::MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct EntryPointWrapper {
    /// The entrypoint selector
	pub entrypoint_selector: H256,
	/// The entrypoint offset
	pub entrypoint_offset: U256,
}

impl EntryPointWrapper {
	/// Creates a new instance of an entrypoint.
	pub fn new(entrypoint_selector: H256, entrypoint_offset: U256) -> Self {
		Self { entrypoint_selector, entrypoint_offset }
	}

	/// Convert to Starknet EntryPoint
	pub fn to_starknet_entry_point(&self) -> EntryPoint {
		EntryPoint {
			selector: EntryPointSelector(
				StarkFelt::new(self.entrypoint_selector.0).unwrap(),
			),
			offset: EntryPointOffset(self.entrypoint_offset.as_usize()),
		}
	}
}

/// Representation of a Starknet Call Entry Point.
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
