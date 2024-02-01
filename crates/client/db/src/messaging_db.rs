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

#[derive(Encode, Decode)]
pub struct LastSyncedEventBlock {
    pub block_number: u64,
    pub event_index: u64,
}

impl LastSyncedEventBlock {
    pub fn new(block_number: u64, event_index: u64) -> Self {
        LastSyncedEventBlock { block_number, event_index }
    }
}

impl<B: BlockT> MessagingDb<B> {
    pub fn last_synced_l1_block_with_event(&self) -> Result<LastSyncedEventBlock, DbError> {
        match self.db.get(crate::columns::MESSAGING, crate::static_keys::LAST_SYNCED_L1_EVENT_BLOCK) {
            Some(raw) => Ok(LastSyncedEventBlock::decode(&mut &raw[..])?),
            None => Ok(LastSyncedEventBlock::new(0, 0)),
        }
    }

    pub fn update_last_synced_l1_block_with_event(
        &self,
        last_synced_event_block: &LastSyncedEventBlock,
    ) -> Result<(), DbError> {
        let mut transaction = sp_database::Transaction::new();

        transaction.set(
            crate::columns::MESSAGING,
            crate::static_keys::LAST_SYNCED_L1_EVENT_BLOCK,
            &last_synced_event_block.encode(),
        );

        self.db.commit(transaction)?;

        Ok(())
    }
}
