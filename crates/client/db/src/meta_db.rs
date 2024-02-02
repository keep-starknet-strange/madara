use std::marker::PhantomData;
use std::sync::Arc;

// Substrate
use parity_scale_codec::{Decode, Encode};
use sp_database::Database;
use sp_runtime::traits::Block as BlockT;

use crate::{DbError, DbHash};

/// Allow interaction with the meta db
///
/// The meta db store the tips of the synced chain.
/// In case of forks, there can be multiple tips.
pub struct MetaDb<B: BlockT> {
    pub(crate) db: Arc<dyn Database<DbHash>>,
    pub(crate) _marker: PhantomData<B>,
}

impl<B: BlockT> MetaDb<B> {
    /// Retrieve the current tips of the synced chain
    pub fn current_syncing_tips(&self) -> Result<Vec<B::Hash>, DbError> {
        match self.db.get(crate::columns::META, crate::static_keys::CURRENT_SYNCING_TIPS) {
            Some(raw) => Ok(Vec::<B::Hash>::decode(&mut &raw[..])?),
            None => Ok(Vec::new()),
        }
    }

    /// Store the current tips of the synced chain
    pub fn write_current_syncing_tips(&self, tips: Vec<B::Hash>) -> Result<(), DbError> {
        let mut transaction = sp_database::Transaction::new();

        transaction.set(crate::columns::META, crate::static_keys::CURRENT_SYNCING_TIPS, &tips.encode());

        self.db.commit(transaction)?;

        Ok(())
    }
}
