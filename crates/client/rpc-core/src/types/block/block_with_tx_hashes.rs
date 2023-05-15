use serde::{Deserialize, Serialize};

use super::{BlockStatus, FieldElement};

/// The resulting block information with transaction hashes
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MaybePendingBlockWithTxHashes {
    Block(BlockWithTxHashes),
    PendingBlock(PendingBlockWithTxHashes),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockWithTxHashes {
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
    /// The hashes of the transactions included in this block
    pub transactions: Vec<FieldElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingBlockWithTxHashes {
    /// The hashes of the transactions included in this block
    pub transactions: Vec<FieldElement>,
    /// The time in which the block was created, encoded in Unix time
    pub timestamp: u64,
    /// The Starknet identity of the sequencer submitting this block
    pub sequencer_address: FieldElement,
    /// The hash of this block's parent
    pub parent_hash: FieldElement,
}
