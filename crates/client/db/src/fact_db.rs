use std::marker::PhantomData;
use std::sync::Arc;

// Substrate
use scale_codec::{Decode, Encode};
use sp_database::Database;
use sp_runtime::traits::Block as BlockT;

use crate::DbHash;

// The fact db stores DA facts that need to be written to L1
pub struct FactDb<B: BlockT> {
    pub(crate) db: Arc<dyn Database<DbHash>>,
    pub(crate) _marker: PhantomData<B>,
}

impl<B: BlockT> FactDb<B> {
    pub fn block_facts(&self, block_hash: &B::Hash) -> Result<Vec<B::Hash>, String> {
        match self.db.get(crate::columns::FACT, &block_hash.encode()) {
            Some(raw) => Ok(Vec::<B::Hash>::decode(&mut &raw[..]).map_err(|e| format!("{:?}", e))?),
            None => Ok(Vec::new()),
        }
    }

    pub fn block_pie(&self, block_hash: &B::Hash) -> Result<Vec<B::Hash>, String> {
        match self.db.get(crate::columns::FACT, &block_hash.encode()) {
            Some(raw) => Ok(Vec::<B::Hash>::decode(&mut &raw[..]).map_err(|e| format!("{:?}", e))?),
            None => Ok(Vec::new()),
        }
    }

    pub fn store_block_facts(&self, block_hash: B::Hash, facts: Vec<B::Hash>) -> Result<(), String> {
        let mut transaction = sp_database::Transaction::new();

        transaction.set(crate::columns::FACT, &block_hash.encode(), &facts.encode());

        self.db.commit(transaction).map_err(|e| format!("{:?}", e))?;

        Ok(())
    }

    pub fn store_block_pie(&self, block_hash: B::Hash, facts: Vec<B::Hash>) -> Result<(), String> {
        let mut transaction = sp_database::Transaction::new();

        transaction.set(crate::columns::FACT, &block_hash.encode(), &facts.encode());

        self.db.commit(transaction).map_err(|e| format!("{:?}", e))?;

        Ok(())
    }
}