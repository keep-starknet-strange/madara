/// Maximum number of filter keys that can be passed to the `get_events` RPC.
pub const MAX_EVENTS_KEYS: usize = 100;
/// Maximum number of events that can be fetched in a single chunk for the `get_events` RPC.
pub const MAX_EVENTS_CHUNK_SIZE: usize = 1000;

/// Path to the genesis assets file
pub const GENESIS_ASSETS_PATH: &[&str] = &["configs", "genesis-assets", "genesis.json"];
