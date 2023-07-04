/// Maximum number of filter keys that can be passed to the `get_events` RPC.
pub const MAX_EVENTS_KEYS: usize = 100;
/// Maximum number of events that can be fetched in a single chunk for the `get_events` RPC.
pub const MAX_EVENTS_CHUNK_SIZE: usize = 1000;
/// Maximum number of keys that can be used to query a storage proof using `getProof` RPC.
pub const MAX_STORAGE_PROOF_KEYS_BY_QUERY: usize = 100;
