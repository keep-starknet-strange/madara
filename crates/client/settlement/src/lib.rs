pub mod errors;
pub mod ethereum;
mod sync_state;

use std::marker::PhantomData;
use std::time::Duration;

use async_trait::async_trait;
use mp_snos_output::StarknetOsOutput;
use serde::{Deserialize, Serialize};
use sp_runtime::traits::Block;
use starknet_api::hash::{StarkFelt, StarkHash};

use crate::errors::{Error, Result};

pub struct SettlementWorker<B, H, SC>(PhantomData<(B, H, SC)>);

#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum SettlementLayer {
    /// Use Ethereum core contract
    Ethereum,
}

#[async_trait]
pub trait SettlementProvider<B: Block>: Send + Sync {
    async fn is_initialized(&self) -> Result<bool, B>;
    async fn get_chain_spec(&self) -> Result<StarknetSpec, B>;
    async fn get_state(&self) -> Result<StarknetState, B>;
    async fn update_state(&self, program_output: StarknetOsOutput) -> Result<(), B>;
}

/// Starknet chain identity, contains OS config & program hashes
///
/// How to calculate program hash:
///     1. Install Cairo https://docs.cairo-lang.org/quickstart.html
///     2. Get latest Starknet OS sources (e.g. https://github.com/keep-starknet-strange/snos)
///     3. Run `cairo-hash-program --program <path-to-the-cairo-build-artifact>.json`
///
/// How to calculate config hash:
///     1. Get Starknet chain ID, which is a string reinterpreted as big number
///     2. Get Starknet fee token address
///     3. Calculate Pedersen hash of [CONFIG_HASH_VERSION; chain_id; fee_token_address]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StarknetSpec {
    /// Starknet OS config hash
    pub config_hash: StarkHash,
    /// Starknet OS program hash
    pub program_hash: StarkHash,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StarknetState {
    /// The state commitment after last settled block.
    pub state_root: StarkHash,
    /// The number (height) of last settled block.
    pub block_number: StarkFelt,
}

pub trait RetryStrategy<B: Block>: Send + Sync {
    fn can_retry(&self, error: &Error<B>) -> Option<Duration>;
}
