//! StarkNet block primitives.

mod header;
use alloc::vec::Vec;

use frame_support::BoundedVec;
pub use header::*;
use sp_core::ConstU32;

use crate::execution::types::Felt252Wrapper;
use crate::transaction::types::{Transaction, TransactionReceiptWrapper};

/// Block transactions max size
// TODO: add real value (#250)
pub type MaxTransactions = ConstU32<4294967295>;

/// Block Transactions
pub type BlockTransactions = BoundedVec<Transaction, MaxTransactions>;

/// Block transaction receipts.
pub type BlockTransactionReceipts = BoundedVec<TransactionReceiptWrapper, MaxTransactions>;

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
#[cfg_attr(feature = "std", derive(serde::Deserialize))]
pub struct Block {
    /// The block header.
    header: Header,
    /// The block transactions.
    transactions: BlockTransactions,
    /// The block transaction receipts.
    transaction_receipts: BlockTransactionReceipts,
}

impl Block {
    /// Creates a new block.
    ///
    /// # Arguments
    ///
    /// * `header` - The block header.
    /// * `transactions` - The block transactions.
    pub fn new(
        header: Header,
        transactions: BlockTransactions,
        transaction_receipts: BlockTransactionReceipts,
    ) -> Self {
        Self { header, transactions, transaction_receipts }
    }

    /// Return a reference to the block header
    pub fn header(&self) -> &Header {
        &self.header
    }

    /// Return a reference to all transactions
    pub fn transactions(&self) -> &BlockTransactions {
        &self.transactions
    }

    /// Returns a reference to all transaction receipts.
    pub fn transaction_receipts(&self) -> &BlockTransactionReceipts {
        &self.transaction_receipts
    }

    /// Return a reference to all transaction hashes
    pub fn transactions_hashes(&self) -> Vec<Felt252Wrapper> {
        let transactions = &self.transactions;
        transactions.into_iter().map(|tx| tx.hash).collect()
    }
}
