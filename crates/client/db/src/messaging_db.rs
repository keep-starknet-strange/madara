use std::marker::PhantomData;
use std::sync::Arc;

// Substrate
use parity_scale_codec::{Decode, Encode};
use sp_database::Database;
use sp_runtime::traits::Block as BlockT;

use crate::error::DbError;
use crate::DbHash;

pub struct MessagingDb<B: BlockT> {
    pub(crate) db: Arc<dyn Database<DbHash>>,
    pub(crate) _marker: PhantomData<B>,
}

impl<B: BlockT> MessagingDb<B> {
    pub fn last_synced_l1_block(&self) -> Result<u64, DbError> {
        match self.db.get(crate::columns::MESSAGING, crate::static_keys::LAST_SYNCED_L1_BLOCK) {
            Some(raw) => Ok(u64::decode(&mut &raw[..])?),
            None => Ok(0),
        }
    }

    pub fn update_last_synced_l1_block(&self, l1_block_number: &u64) -> Result<(), DbError> {
        let mut transaction = sp_database::Transaction::new();

        transaction.set(crate::columns::MESSAGING, crate::static_keys::LAST_SYNCED_L1_BLOCK, &l1_block_number.encode());

        self.db.commit(transaction)?;

        Ok(())
    }
}
