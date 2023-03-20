use frame_support::BoundedVec;
use sp_core::{ConstU32, H256, U256};

use crate::execution::{CallEntryPoint, ContractAddress};

/// Max size of the event array.
pub type MaxArraySize = ConstU32<4294967295>;

/// Different tx types.
pub enum TxType {
    /// Regular invoke transaction.
    InvokeTx,
    /// Message sent from ethereum.
    L1HandlerTx,
}
/// Representation of a Starknet transaction.
#[derive(Clone, Debug, PartialEq, Eq, codec::Encode, codec::Decode, scale_info::TypeInfo, codec::MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct Transaction {
    /// The version of the transaction.
    pub version: U256,
    /// Transaction hash.
    pub hash: H256,
    /// Signature.
    pub signature: BoundedVec<H256, MaxArraySize>,
    /// Events.
    pub events: BoundedVec<Event, MaxArraySize>,
    /// Sender Address
    pub sender_address: ContractAddress,
    /// Nonce
    pub nonce: U256,
    /// Call entrypoint
    pub call_entrypoint: CallEntryPoint,
    /// Selector
    pub selector: H256,
}

/// Representation of a Starknet event.
#[derive(Clone, Debug, PartialEq, Eq, codec::Encode, codec::Decode, scale_info::TypeInfo, codec::MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct Event {
    /// The keys (topics) of the event.
    pub keys: BoundedVec<H256, MaxArraySize>,
    /// The data of the event.
    pub data: BoundedVec<H256, MaxArraySize>,
    /// The address that emited the event
    pub from_address: H256,
}
