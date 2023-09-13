//! StarkNet block primitives.

mod header;
use alloc::vec::Vec;

pub use header::*;
use sp_core::ConstU32;

use crate::execution::types::Felt252Wrapper;
use crate::traits::hash::HasherT;
use crate::transaction::compute_hash::ComputeTransactionHash;
use crate::transaction::Transaction;

/// Block transactions max size
// TODO: add real value (#250)
pub type MaxTransactions = ConstU32<4294967295>;

/// Maximum number of storage slots per contract
pub type MaxStorageSlots = ConstU32<{ u32::MAX }>;

/// Block Transactions
pub type BlockTransactions = Vec<Transaction>;

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

    /// Return a reference to all transaction hashes
    pub fn transactions_hashes<H: HasherT>(&self, chain_id: Felt252Wrapper) -> Vec<Felt252Wrapper> {
        self.transactions.iter().map(|tx| tx.compute_hash::<H>(chain_id, false)).collect()
    }
}
