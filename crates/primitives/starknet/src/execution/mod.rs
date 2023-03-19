//! Starknet execution functionality.

use alloc::sync::Arc;
use alloc::vec;

use blockifier::execution::entry_point::CallEntryPoint as StarknetCallEntryPoint;
use frame_support::BoundedVec;
use sp_core::{ConstU32, H256};
use starknet_api::api_core::{ClassHash, ContractAddress as StarknetContractAddress, EntryPointSelector};
use starknet_api::hash::StarkFelt;
use starknet_api::state::EntryPointType;
use starknet_api::transaction::Calldata;

/// The address of a contract.
pub type ContractAddress = [u8; 32];

type MaxCalldataSize = ConstU32<4294967295>;
type ContractClassHash = [u8; 32];
/// TODO: define enum
type StarknetEntryPointType = u8;

/// Representation of a Starknet transaction.
#[derive(Clone, Debug, PartialEq, Eq, codec::Encode, codec::Decode, scale_info::TypeInfo, codec::MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct CallEntryPoint {
    /// The class hash
    pub class_hash: ContractClassHash,
    /// The entrypoint type
    pub entrypoint_type: StarknetEntryPointType,
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

impl CallEntryPoint {
    /// Creates a new instance of a call entrypoint.
    pub fn new(class_hash: ContractClassHash, entrypoint_type: StarknetEntryPointType) -> Self {
        Self { class_hash, entrypoint_type, ..Self::default() }
    }

    /// Convert to Starknet CallEntryPoint
    pub fn to_starknet_call_entry_point(&self) -> StarknetCallEntryPoint {
        StarknetCallEntryPoint {
            class_hash: Some(ClassHash(StarkFelt::new(self.class_hash).unwrap())),
            /// TODO: Change this to use self.entrypoint_type
            entry_point_type: EntryPointType::External,
            entry_point_selector: EntryPointSelector(StarkFelt::new(self.entrypoint_selector.unwrap_or_default().0).unwrap()),
            calldata: Calldata(Arc::new(
                self.calldata.clone().into_inner().iter().map(|x| StarkFelt::new(*(*x).as_fixed_bytes()).unwrap()).collect(),
            )),
            storage_address: StarknetContractAddress::try_from(StarkFelt::new(self.storage_address).unwrap()).unwrap(),
            caller_address: StarknetContractAddress::try_from(StarkFelt::new(self.caller_address).unwrap()).unwrap(),
        }
    }
}
impl Default for CallEntryPoint {
    fn default() -> Self {
        Self {
            class_hash: ContractClassHash::default(),
            entrypoint_type: 0,
            entrypoint_selector: Some(H256::default()),
            calldata: BoundedVec::try_from(vec![H256::zero(); 32]).unwrap(),
            storage_address: ContractAddress::default(),
            caller_address: ContractAddress::default(),
        }
    }
}
