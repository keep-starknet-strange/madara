use serde::{Deserialize, Serialize};

use super::{BlockNumber, FieldElement};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct BlockHashAndNumber {
    pub block_hash: FieldElement,
    pub block_number: BlockNumber,
}
