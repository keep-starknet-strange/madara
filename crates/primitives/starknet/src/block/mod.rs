//! StarkNet block primitives.

mod header;
use frame_support::BoundedVec;
pub use header::*;
use sp_core::{ConstU32, H256};

use crate::transaction::types::Transaction;

/// Serializer
pub mod serialize;

/// Block transactions max size
pub type MaxTransactions = ConstU32<4294967295>; // TODO: add real value

/// Block Transactions
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    scale_codec::Encode,
    scale_codec::Decode,
    scale_info::TypeInfo,
    scale_codec::MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum BlockTransactions {
    /// Only hashes
    Hashes(BoundedVec<H256, MaxTransactions>),
    /// Full transactions
    Full(BoundedVec<Transaction, MaxTransactions>),
}

impl Default for BlockTransactions {
    fn default() -> Self {
        Self::Hashes(BoundedVec::default())
    }
}

/// Starknet block definition.
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
pub struct Block {
    /// The block header.
    header: Header,
    /// The block transactions.
    transactions: BlockTransactions,
}

impl Block {
    /// Creates a new block.
    pub fn new(header: Header, transactions: BlockTransactions) -> Self {
        Self { header, transactions }
    }

    /// Return a reference to the block header
    pub fn header(&self) -> &Header {
        &self.header
    }
}
