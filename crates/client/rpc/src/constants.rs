/// Maximum number of filter keys that can be passed to the `get_events` RPC.
pub const MAX_EVENTS_KEYS: usize = 100;
/// Maximum number of events that can be fetched in a single chunk for the `get_events` RPC.
pub const MAX_EVENTS_CHUNK_SIZE: usize = 1000;

/// Class hashes whose corresponding contracts classes are "NoValidateAccount".
pub const NO_VALIDATE_ACCOUNT_CLASS_HASHES: [&str; 2] = [
    "0x0279d77db761fba82e0054125a6fdb5f6baa6286fa3fb73450cc44d193c2d37f",
    "0x35ccefcf9d5656da623468e27e682271cd327af196785df99e7fee1436b6276",
];
/// Predeployed accounts adresses defined in the genesis state.
pub const PREDEPLOYED_ACCOUNTS_ADDRESSES: [&str; 4] = ["0x1", "0x2", "0x3", "0x4"];

/// Pk for the relevant predeployed accounts defined in the genesis state.
pub const ARGENT_PK: &str = "0x00c1cf1490de1352865301bb8705143f3ef938f97fdf892f1090dcb5ac7bcd1d";
/// Fee token contract address defined in the genesis state.
pub const FEE_TOKEN_ADDRESS: &str = "0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7";
