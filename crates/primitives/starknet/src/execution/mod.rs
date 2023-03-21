//! Starknet execution functionality.

use alloc::sync::Arc;
use alloc::vec;

use blockifier::execution::entry_point::CallEntryPoint as StarknetCallEntryPoint;
use frame_support::BoundedVec;
use sp_core::{ConstU32, H256};
use starknet_api::api_core::{ClassHash, ContractAddress as StarknetContractAddress, EntryPointSelector};
use starknet_api::hash::StarkFelt;
use starknet_api::state::EntryPointType as StarknetEntryPointType;
use starknet_api::transaction::Calldata;

/// The address of a contract.
pub type ContractAddress = [u8; 32];

type MaxCalldataSize = ConstU32<4294967295>;
type ContractClassHash = [u8; 32];

/// Enum that represents all the entrypoints types.
#[derive(Clone, Debug, PartialEq, Eq, codec::Encode, codec::Decode, scale_info::TypeInfo, codec::MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum EntryPointType {
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
pub struct CallEntryPoint {
    /// The class hash
    pub class_hash: Option<ContractClassHash>,
    /// The entrypoint type
    pub entrypoint_type: EntryPointType,
    /// The entrypoint selector
    /// An invoke transaction without an entry point selector invokes the 'execute' function.
    pub entrypoint_selector: Option<H256>,
    /// The Calldata
    pub calldata: BoundedVec<H256, MaxCalldataSize>,
    /// The storage address
    pub storage_address: ContractAddress,
    /// The caller address
    pub caller_address: ContractAddress,
}
impl EntryPointType {
    /// Convert Kaioshin entrypoint type to Starknet entrypoint type.
    pub fn to_starknet(&self) -> StarknetEntryPointType {
        match self {
            Self::Constructor => StarknetEntryPointType::Constructor,
            Self::External => StarknetEntryPointType::External,
            Self::L1Handler => StarknetEntryPointType::L1Handler,
        }
    }
}

impl CallEntryPoint {
    /// Creates a new instance of a call entrypoint.
    pub fn new(
        class_hash: Option<ContractClassHash>,
        entrypoint_type: EntryPointType,
        entrypoint_selector: Option<H256>,
        calldata: BoundedVec<H256, MaxCalldataSize>,
        storage_address: ContractAddress,
        caller_address: ContractAddress,
    ) -> Self {
        Self { class_hash, entrypoint_type, entrypoint_selector, calldata, storage_address, caller_address }
    }

    /// Convert to Starknet CallEntryPoint
    pub fn to_starknet_call_entry_point(&self) -> StarknetCallEntryPoint {
        let class_hash = self.class_hash.map(|class_hash| ClassHash(StarkFelt::new(class_hash).unwrap()));
        StarknetCallEntryPoint {
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
            storage_address: StarknetContractAddress::try_from(StarkFelt::new(self.storage_address).unwrap()).unwrap(),
            caller_address: StarknetContractAddress::try_from(StarkFelt::new(self.caller_address).unwrap()).unwrap(),
        }
    }
}
impl Default for CallEntryPoint {
    fn default() -> Self {
        Self {
            class_hash: Some(ContractClassHash::default()),
            entrypoint_type: EntryPointType::External,
            entrypoint_selector: Some(H256::default()),
            calldata: BoundedVec::try_from(vec![H256::zero(); 32]).unwrap(),
            storage_address: ContractAddress::default(),
            caller_address: ContractAddress::default(),
        }
    }
}
