use serde::{Deserialize, Serialize};

use super::FieldElement;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct SyncStatus {
    pub starting_block_hash: FieldElement,
    pub starting_block_num: u64,
    pub current_block_hash: FieldElement,
    pub current_block_num: u64,
    pub highest_block_hash: FieldElement,
    pub highest_block_num: u64,
}

/// Boolean or SyncStatus
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Syncing {
    #[serde(rename = "sync_status")]
    False(bool),
    #[serde(rename = "sync_status")]
    SyncStatus(SyncStatus),
}
