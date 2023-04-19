//! Utility to read the Starknet block form the Substrate block digest
//!
//! Following the wrapper block model, each one of the madara block contains a starknet block.
//! This block is not only stored in the chain storage, but also pushed inton the wrapper block
//! itself.
//!
//! We expect the starknet pallet to push a log into the substrate digest in it's `on_finalize`
//! hook. This log must contain the whole new starknet block.
//!
//! In the current state of this crate, only one single log must be pushed to the digest each block,
//! and it should contain the starknet block. Pushing more log will make it impossible for this set
//! of reader functions to operate properly.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::large_enum_variant)]
#![deny(unused_crate_dependencies)]

mod error;
#[cfg(test)]
mod tests;

pub use error::FindLogError;
use mp_starknet::block::Block as StarknetBlock;
use scale_codec::{Decode, Encode};
use sp_runtime::generic::{Digest, OpaqueDigestItemId};
use sp_runtime::ConsensusEngineId;

pub const MADARA_ENGINE_ID: ConsensusEngineId = [b'm', b'a', b'd', b'a'];

/// A Madara log
///
/// Right now we only expect Madara to log the Starknet block,
/// but other usecases may appears later on.
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub enum Log {
    #[codec(index = 0)]
    Block(StarknetBlock),
}

/// Return the wrapped [StarknetBlock] contained in a given [Digest]
pub fn find_starknet_block(digest: &Digest) -> Result<StarknetBlock, FindLogError> {
    find_log(digest).map(|log| match log {
        Log::Block(b) => b,
    })
}

/// Return the Madara [Log] contained in a given [Digest]
pub fn find_log(digest: &Digest) -> Result<Log, FindLogError> {
    _find_log(digest, OpaqueDigestItemId::Consensus(&MADARA_ENGINE_ID))
}

/// Ensure there is a single valid Madara [Log] in a given [Digest]
///
/// It can be used to check if the wrapper block does contains the wrapped block
/// without reading the wrapped block itself
pub fn ensure_log(digest: &Digest) -> Result<(), FindLogError> {
    find_log(digest).map(|_log| ())
}

fn _find_log<Log: Decode>(digest: &Digest, digest_item_id: OpaqueDigestItemId) -> Result<Log, FindLogError> {
    let mut found = None;

    for log in digest.logs() {
        let log = log.try_to::<Log>(digest_item_id);
        match (log, found.is_some()) {
            (Some(_), true) => return Err(FindLogError::MultipleLogs),
            (Some(log), false) => found = Some(log),
            (None, _) => (),
        }
    }

    found.ok_or(FindLogError::NotLog)
}
