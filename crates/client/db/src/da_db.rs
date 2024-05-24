use std::sync::Arc;

// Substrate
use parity_scale_codec::{Decode, Encode};
use sp_database::Database;
// Starknet
use starknet_api::block::BlockHash;
use starknet_api::hash::StarkFelt;

use crate::{DbError, DbHash};

// The fact db stores DA facts that need to be written to L1
pub struct DaDb {
    pub(crate) db: Arc<dyn Database<DbHash>>,
}

// TODO: purge old cairo job keys
impl DaDb {
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
