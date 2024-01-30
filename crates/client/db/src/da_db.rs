use std::marker::PhantomData;
use std::sync::Arc;

use ethers::types::U256;
// Substrate
use parity_scale_codec::{Decode, Encode};
use sp_database::Database;
use sp_runtime::traits::Block as BlockT;
// Starknet
use starknet_api::block::BlockHash;
use starknet_api::hash::StarkFelt;
use uuid::Uuid;

use crate::{DbError, DbHash};

// The fact db stores DA facts that need to be written to L1
pub struct DaDb<B: BlockT> {
    pub(crate) db: Arc<dyn Database<DbHash>>,
    pub(crate) _marker: PhantomData<B>,
}

// TODO: purge old cairo job keys
impl<B: BlockT> DaDb<B> {
    pub fn state_diff(&self, block_hash: &BlockHash) -> Result<Option<bytes::Bytes>, DbError> {
        match self.db.get(crate::columns::DA, block_hash.0.bytes()) {
            Some(raw) => Ok(Some(raw.into())),
            None => Ok(None),
        }
    }

    pub fn store_state_diff(&self, block_hash: &BlockHash, diff: bytes::Bytes) -> Result<(), DbError> {
        let mut transaction = sp_database::Transaction::new();

        transaction.set(crate::columns::DA, block_hash.0.bytes(), &diff.encode());

        self.db.commit(transaction)?;

        Ok(())
    }

    pub fn cairo_job(&self, block_hash: &BlockHash) -> Result<Option<Uuid>, DbError> {
        match self.db.get(crate::columns::DA, block_hash.0.bytes()) {
            Some(raw) => Ok(Some(Uuid::from_slice(&raw[..])?)),
            None => Ok(None),
        }
    }

    pub fn update_cairo_job(&self, block_hash: &BlockHash, job_id: Uuid) -> Result<(), DbError> {
        let mut transaction = sp_database::Transaction::new();

        transaction.set(crate::columns::DA, block_hash.0.bytes(), &job_id.into_bytes());

        self.db.commit(transaction)?;

        Ok(())
    }

    pub fn last_proved_block(&self) -> Result<BlockHash, DbError> {
        match self.db.get(crate::columns::DA, crate::static_keys::LAST_PROVED_BLOCK) {
            Some(raw) => {
                let felt = StarkFelt::decode(&mut &raw[..])?;
                Ok(BlockHash(felt))
            }
            None => Err(DbError::ValueNotInitialized(
                crate::columns::DA,
                // Safe coze `LAST_PROVED_BLOCK` is utf8
                unsafe { std::str::from_utf8_unchecked(crate::static_keys::LAST_PROVED_BLOCK) }.to_string(),
            )),
        }
    }

    pub fn update_last_proved_block(&self, block_hash: &BlockHash) -> Result<(), DbError> {
        let mut transaction = sp_database::Transaction::new();

        transaction.set(crate::columns::DA, crate::static_keys::LAST_PROVED_BLOCK, &block_hash.0.encode());

        self.db.commit(transaction)?;

        Ok(())
    }
}
