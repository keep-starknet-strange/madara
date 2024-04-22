//! Starknet block primitives.
mod header;

use blockifier::transaction::transaction_execution::Transaction;
pub use header::Header;
use mp_felt::Felt252Wrapper;
use starknet_api::transaction::TransactionHash;

/// Block Transactions
pub type BlockTransactions = Vec<Transaction>;

/// Block tag.
///
/// A tag specifying a dynamic reference to a block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "parity-scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
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

#[derive(Debug, thiserror::Error)]
pub enum NewBlockError {
    #[error("header's field `transaction_count` does not matched the len of the `transactions` field")]
    InvalidTxCount,
}

/// Starknet block definition.
#[derive(Clone, Debug)]
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
    pub fn try_new(header: Header, transactions: BlockTransactions) -> Result<Self, NewBlockError> {
        if header.transaction_count as usize != transactions.len() {
            Err(NewBlockError::InvalidTxCount)
        } else {
            Ok(Self { header, transactions })
        }
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
    pub fn transactions_hashes(&self) -> impl '_ + Iterator<Item = TransactionHash> {
        self.transactions.iter().map(|tx| match tx {
            Transaction::AccountTransaction(ac) => match ac {
                blockifier::transaction::account_transaction::AccountTransaction::Declare(tx) => tx.tx_hash,
                blockifier::transaction::account_transaction::AccountTransaction::DeployAccount(tx) => tx.tx_hash,
                blockifier::transaction::account_transaction::AccountTransaction::Invoke(tx) => tx.tx_hash,
            },
            Transaction::L1HandlerTransaction(lhc) => lhc.tx_hash,
        })
    }
}

#[cfg(test)]
mod tests;
