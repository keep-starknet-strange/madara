/// Maximum number of filter keys that can be passed to the `get_events` RPC.
pub const MAX_EVENTS_KEYS: usize = 100;
/// Maximum number of events that can be fetched in a single chunk for the `get_events` RPC.
pub const MAX_EVENTS_CHUNK_SIZE: usize = 1000;

/// Path to the `genesis.json` file
pub const GENESIS_FILE: &str = "configs/genesis-assets/genesis.json";

/// Pk for the relevant predeployed accounts defined in the genesis state.
pub const ARGENT_PK: &str = "0x00c1cf1490de1352865301bb8705143f3ef938f97fdf892f1090dcb5ac7bcd1d";
