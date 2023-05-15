mod block_hash_and_number;
mod block_id;
mod block_status;
mod block_tag;
mod block_with_tx_hashes;
mod block_with_txs;

pub use block_hash_and_number::BlockHashAndNumber;
pub use block_id::BlockId;
pub use block_status::BlockStatus;
pub use block_tag::BlockTag;
pub use block_with_tx_hashes::*;
pub use block_with_txs::*;

use super::FieldElement;

pub type BlockHash = FieldElement;
pub type BlockNumber = u64;
