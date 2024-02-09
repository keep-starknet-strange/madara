use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

// Substrate
use parity_scale_codec::{Decode, Encode};
use sp_database::Database;
use sp_runtime::traits::Block as BlockT;
use starknet_api::hash::StarkHash;

use crate::{DbError, DbHash};

/// The mapping to write in db
#[derive(Debug)]
pub struct MappingCommitment<B: BlockT> {
    pub block_hash: B::Hash,
    pub starknet_block_hash: StarkHash,
    pub starknet_transaction_hashes: Vec<StarkHash>,
}

/// Allow interaction with the mapping db
pub struct MappingDb<B: BlockT> {
    db: Arc<dyn Database<DbHash>>,
    write_lock: Arc<Mutex<()>>,
    /// Whether more information should be cached in the database.
    cache_more_things: bool,
    _marker: PhantomData<B>,
}

impl<B: BlockT> MappingDb<B> {
    /// Creates a new instance of the mapping database.
    pub fn new(db: Arc<dyn Database<DbHash>>, cache_more_things: bool) -> Self {
        Self { db, write_lock: Arc::new(Mutex::new(())), cache_more_things, _marker: PhantomData }
    }

    /// Check if the given block hash has already been processed
    pub fn is_synced(&self, block_hash: &B::Hash) -> Result<bool, DbError> {
        match self.db.get(crate::columns::SYNCED_MAPPING, &block_hash.encode()) {
            Some(raw) => Ok(bool::decode(&mut &raw[..])?),
            None => Ok(false),
        }
    }

    /// Return the hash of the Substrate block wrapping the Starknet block with given hash
    ///
    /// Under some circumstances it can return multiples blocks hashes, meaning that the result has
    /// to be checked against the actual blockchain state in order to find the good one.
    pub fn block_hash(&self, starknet_block_hash: StarkHash) -> Result<Option<Vec<B::Hash>>, DbError> {
        match self.db.get(crate::columns::BLOCK_MAPPING, &starknet_block_hash.encode()) {
            Some(raw) => Ok(Some(Vec::<B::Hash>::decode(&mut &raw[..])?)),
            None => Ok(None),
        }
    }

    /// Register that a Substrate block has been seen, without it containing a Starknet one
    pub fn write_none(&self, block_hash: B::Hash) -> Result<(), DbError> {
        let _lock = self.write_lock.lock();

        let mut transaction = sp_database::Transaction::new();

        transaction.set(crate::columns::SYNCED_MAPPING, &block_hash.encode(), &true.encode());

        self.db.commit(transaction)?;

        Ok(())
    }

    /// Register that a Substate block has been seen and map it to the Statknet block it contains
    pub fn write_hashes(&self, commitment: MappingCommitment<B>) -> Result<(), DbError> {
        let _lock = self.write_lock.lock();

        let mut transaction = sp_database::Transaction::new();

        let substrate_hashes = match self.block_hash(commitment.starknet_block_hash) {
            Ok(Some(mut data)) => {
                data.push(commitment.block_hash);
                log::warn!(
                    target: "fc-db",
                    "Possible equivocation at starknet block hash {} {:?}",
                    &commitment.starknet_block_hash,
                    &data
                );
                data
            }
            _ => vec![commitment.block_hash],
        };

        transaction.set(
            crate::columns::BLOCK_MAPPING,
            &commitment.starknet_block_hash.encode(),
            &substrate_hashes.encode(),
        );

        transaction.set(crate::columns::SYNCED_MAPPING, &commitment.block_hash.encode(), &true.encode());

        for transaction_hash in commitment.starknet_transaction_hashes.iter() {
            transaction.set(
                crate::columns::TRANSACTION_MAPPING,
                &transaction_hash.encode(),
                &commitment.block_hash.encode(),
            );
        }

        if self.cache_more_things {
            transaction.set(
                crate::columns::STARKNET_TRANSACTION_HASHES_CACHE,
                &commitment.starknet_block_hash.encode(),
                &commitment.starknet_transaction_hashes.encode(),
            );
        }

        self.db.commit(transaction)?;

        Ok(())
    }

    /// Retrieves the substrate block hash
    /// associated with the given transaction hash, if any.
    ///
    /// # Arguments
    ///
    /// * `transaction_hash` - the transaction hash to search for. H256 is used here because it's a
    ///   native type of substrate, and we are sure it's SCALE encoding is optimized and will not
    ///   change.
    pub fn block_hash_from_transaction_hash(&self, transaction_hash: StarkHash) -> Result<Option<B::Hash>, DbError> {
        match self.db.get(crate::columns::TRANSACTION_MAPPING, &transaction_hash.encode()) {
            Some(raw) => Ok(Some(<B::Hash>::decode(&mut &raw[..])?)),
            None => Ok(None),
        }
    }

    /// Returns the list of transaction hashes for the given block hash.
    ///
    /// # Arguments
    ///
    /// * `starknet_hash` - the hash of the starknet block to search for.
    ///
    /// # Returns
    ///
    /// The list of transaction hashes.
    ///
    /// This function may return `None` for two separate reasons:
    ///
    /// - The cache is disabled.
    /// - The provided `starknet_hash` is not present in the cache.
    pub fn cached_transaction_hashes_from_block_hash(
        &self,
        starknet_block_hash: StarkHash,
    ) -> Result<Option<Vec<StarkHash>>, DbError> {
        if !self.cache_more_things {
            // The cache is not enabled, no need to even touch the database.
            return Ok(None);
        }

        match self.db.get(crate::columns::STARKNET_TRANSACTION_HASHES_CACHE, &starknet_block_hash.encode()) {
            Some(raw) => Ok(Some(Vec::<StarkHash>::decode(&mut &raw[..])?)),
            None => Ok(None),
        }
    }
}
