use mp_starknet::transaction::types::Transaction;
use serde::{Deserialize, Serialize};

use super::{BlockStatus, FieldElement};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MaybePendingBlockWithTxs {
    Block(BlockWithTxs),
    PendingBlock(PendingBlockWithTxs),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockWithTxs {
    pub status: BlockStatus,
    pub block_hash: FieldElement,
    /// The hash of this block's parent
    pub parent_hash: FieldElement,
    /// The block number (its height)
    pub block_number: u64,
    /// The new global state root
    pub new_root: FieldElement,
    /// The time in which the block was created, encoded in Unix time
    pub timestamp: u64,
    /// The Starknet identity of the sequencer submitting this block
    pub sequencer_address: FieldElement,
    /// The transactions in this block
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingBlockWithTxs {
    /// The transactions in this block
    pub transactions: Vec<Transaction>,
    /// The time in which the block was created, encoded in Unix time
    pub timestamp: u64,
    /// The Starknet identity of the sequencer submitting this block
    pub sequencer_address: FieldElement,
    /// The hash of this block's parent
    pub parent_hash: FieldElement,
}
