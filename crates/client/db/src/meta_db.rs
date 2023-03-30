use std::marker::PhantomData;
use std::sync::Arc;

// Substrate
use scale_codec::{Decode, Encode};
use sp_database::Database;
use sp_runtime::traits::Block as BlockT;

use crate::DbHash;

pub struct MetaDb<Block: BlockT> {
    pub(crate) db: Arc<dyn Database<DbHash>>,
    pub(crate) _marker: PhantomData<Block>,
}

impl<Block: BlockT> MetaDb<Block> {
    pub fn current_syncing_tips(&self) -> Result<Vec<Block::Hash>, String> {
        match self.db.get(crate::columns::META, crate::static_keys::CURRENT_SYNCING_TIPS) {
            Some(raw) => Ok(Vec::<Block::Hash>::decode(&mut &raw[..]).map_err(|e| format!("{:?}", e))?),
            None => Ok(Vec::new()),
        }
    }

    pub fn write_current_syncing_tips(&self, tips: Vec<Block::Hash>) -> Result<(), String> {
        let mut transaction = sp_database::Transaction::new();

        transaction.set(crate::columns::META, crate::static_keys::CURRENT_SYNCING_TIPS, &tips.encode());

        self.db.commit(transaction).map_err(|e| format!("{:?}", e))?;

        Ok(())
    }
}
