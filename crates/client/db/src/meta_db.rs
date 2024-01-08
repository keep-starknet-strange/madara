use std::marker::PhantomData;
use std::sync::Arc;

// Substrate
use scale_codec::{Decode, Encode};
use sp_core::{H256, U256};
use sp_database::Database;
use sp_runtime::traits::Block as BlockT;

use crate::DbHash;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct L1L2BlockMapping {
    pub l1_block_hash: H256,
    pub l1_block_number: u64,
    pub l2_block_hash: U256,
    pub l2_block_number: u64,
}

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
    pub fn current_syncing_tips(&self) -> Result<Vec<B::Hash>, String> {
        match self.db.get(crate::columns::META, crate::static_keys::CURRENT_SYNCING_TIPS) {
            Some(raw) => Ok(Vec::<B::Hash>::decode(&mut &raw[..]).map_err(|e| format!("{:?}", e))?),
            None => Ok(Vec::new()),
        }
    }

    /// Store the current tips of the synced chain
    pub fn write_current_syncing_tips(&self, tips: Vec<B::Hash>) -> Result<(), String> {
        let mut transaction = sp_database::Transaction::new();

        transaction.set(crate::columns::META, crate::static_keys::CURRENT_SYNCING_TIPS, &tips.encode());

        self.db.commit(transaction).map_err(|e| format!("{:?}", e))?;

        Ok(())
    }

    pub fn write_last_l1_l2_mapping(&self, mapping: &L1L2BlockMapping) -> Result<(), String> {
        let mut transaction = sp_database::Transaction::new();

        transaction.set(crate::columns::META, crate::static_keys::LAST_L1_L1_HEADER_MAPPING, &mapping.encode());
        self.db.commit(transaction).map_err(|e| format!("{:?}", e))?;

        Ok(())
    }

    pub fn last_l1_l2_mapping(&self) -> Result<L1L2BlockMapping, String> {
        match self.db.get(crate::columns::META, crate::static_keys::LAST_L1_L1_HEADER_MAPPING) {
            Some(data) => L1L2BlockMapping::decode(&mut &data[..]).map_err(|e| e.to_string()),
            None => Err("last l1 l2 block mapping not found".to_string()),
        }
    }
}
