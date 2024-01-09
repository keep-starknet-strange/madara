//! Starknet block primitives.
#![cfg_attr(not(feature = "std"), no_std)]

#[doc(hidden)]
pub extern crate alloc;

mod header;

use alloc::vec::Vec;

pub use header::*;
use mp_felt::Felt252Wrapper;
use mp_hashers::HasherT;
use mp_transactions::compute_hash::ComputeTransactionHash;
use mp_transactions::Transaction;

/// Block Transactions
pub type BlockTransactions = Vec<Transaction>;

/// Block tag.
///
/// A tag specifying a dynamic reference to a block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum BlockTag {
    #[cfg_attr(feature = "serde", serde(rename = "latest"))]
    Latest,
    #[cfg_attr(feature = "serde", serde(rename = "pending"))]
    Pending,
}

/// Block Id
/// Block hash, number or tag
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub enum BlockId {
    Hash(Felt252Wrapper),
    Number(u64),
    Tag(BlockTag),
}

/// Starknet block definition.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
pub struct Block {
    /// The block header.
    header: Header,
    /// The block transactions.
    transactions: BlockTransactions,
}

impl Block {
    /// Creates a new block.
    ///
    /// # Arguments
    ///
    /// * `header` - The block header.
    /// * `transactions` - The block transactions.
    pub fn new(header: Header, transactions: BlockTransactions) -> Self {
        Self { header, transactions }
    }

    /// Return a reference to the block header
    pub fn header(&self) -> &Header {
        &self.header
    }

    /// Return a reference to all transactions
    pub fn transactions(&self) -> &BlockTransactions {
        &self.transactions
    }

    /// Returns an iterator that iterates over all transaction hashes.
    ///
    /// Those transactions are computed using the given `chain_id`.
    pub fn transactions_hashes<H: HasherT>(
        &self,
        chain_id: Felt252Wrapper,
    ) -> impl '_ + Iterator<Item = Felt252Wrapper> {
        self.transactions.iter().map(move |tx| tx.compute_hash::<H>(chain_id, false))
    }
}

#[cfg(test)]
mod tests;
