use serde::{Deserialize, Serialize};

// In order to mix tagged and untagged {de}serialization for BlockId (see starknet RPC standard)
// in the same object, we need this kind of workaround with intermediate types
use super::{BlockHash, BlockNumber, BlockTag};

#[derive(Serialize, Deserialize, Clone)]
enum BlockIdTaggedVariants {
    #[serde(rename = "block_hash")]
    BlockHash(BlockHash),
    #[serde(rename = "block_number")]
    BlockNumber(BlockNumber),
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
enum BlockIdUntagged {
    Tagged(BlockIdTaggedVariants),
    BlockTag(BlockTag),
}

/// A block hash, number or tag
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(from = "BlockIdUntagged")]
#[serde(into = "BlockIdUntagged")]
pub enum BlockId {
    BlockHash(BlockHash),
    BlockNumber(BlockNumber),
    BlockTag(BlockTag),
}

impl From<BlockIdUntagged> for BlockId {
    fn from(value: BlockIdUntagged) -> Self {
        match value {
            BlockIdUntagged::Tagged(v) => match v {
                BlockIdTaggedVariants::BlockHash(h) => Self::BlockHash(h),
                BlockIdTaggedVariants::BlockNumber(n) => Self::BlockNumber(n),
            },
            BlockIdUntagged::BlockTag(t) => Self::BlockTag(t),
        }
    }
}

impl From<BlockId> for BlockIdUntagged {
    fn from(value: BlockId) -> Self {
        match value {
            BlockId::BlockHash(h) => Self::Tagged(BlockIdTaggedVariants::BlockHash(h)),
            BlockId::BlockNumber(n) => Self::Tagged(BlockIdTaggedVariants::BlockNumber(n)),
            BlockId::BlockTag(t) => Self::BlockTag(t),
        }
    }
}
