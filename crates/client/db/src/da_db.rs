use std::sync::Arc;

use parity_scale_codec::{Decode, Encode};
use sp_database::Database;
use starknet_api::block::BlockHash;

use crate::{DbError, DbHash};

pub struct DaDb {
    pub(crate) db: Arc<dyn Database<DbHash>>,
}

impl DaDb {
    pub fn last_proved_block(&self) -> Result<Option<BlockHash>, DbError> {
        let block_hash = self
            .db
            .get(crate::columns::DA, crate::static_keys::LAST_PROVED_BLOCK)
            .map(|raw| BlockHash::decode(&mut &raw[..]))
            .transpose()?;

        Ok(block_hash)
    }

    pub fn update_last_proved_block(&self, block_hash: &BlockHash) -> Result<(), DbError> {
        let mut transaction = sp_database::Transaction::new();

        transaction.set(crate::columns::DA, crate::static_keys::LAST_PROVED_BLOCK, &block_hash.0.encode());

        self.db.commit(transaction)?;

        Ok(())
    }
}
