//! Starknet execution functionality.

use alloc::vec;

use frame_support::BoundedVec;
use sp_core::{ConstU32, H256};

type MaxCalldataSize = ConstU32<32>;
pub type ContractAddress = [u8; 32];
type ContractClassHash = [u8; 32];
type EntryPointType = u8;

/// Representation of a Starknet transaction.
#[derive(Clone, Debug, PartialEq, Eq, codec::Encode, codec::Decode, scale_info::TypeInfo, codec::MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct CallEntryPoint {
    /// The class hash
    pub class_hash: ContractClassHash,
    /// The entrypoint type
    pub entrypoint_type: EntryPointType,
    /// The entrypoint selector
    /// An invoke transaction without an entry point selector invokes the 'execute' function.
    pub entrypoint_selector: Option<H256>,
    /// The Calldata
    pub calldata: BoundedVec<u8, MaxCalldataSize>,
    /// The storage address
    pub storage_address: ContractAddress,
    /// The caller address
    pub caller_address: ContractAddress,
}

impl CallEntryPoint {
    /// Creates a new instance of a call entrypoint.
    pub fn new(class_hash: ContractClassHash, entrypoint_type: EntryPointType) -> Self {
        Self { class_hash, entrypoint_type, ..Self::default() }
    }
}
impl Default for CallEntryPoint {
    fn default() -> Self {
        Self {
            class_hash: ContractClassHash::default(),
            entrypoint_type: 0,
            entrypoint_selector: None,
            calldata: BoundedVec::try_from(vec![0; 32]).unwrap(),
            storage_address: ContractAddress::default(),
            caller_address: ContractAddress::default(),
        }
    }
}
