use serde::{Deserialize, Serialize};
use starknet_providers::jsonrpc::models::SyncStatus;

/// Boolean or SyncStatus
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Syncing {
    #[serde(rename = "sync_status")]
    False(bool),
    #[serde(rename = "sync_status")]
    SyncStatus(SyncStatus),
}
