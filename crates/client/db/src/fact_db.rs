use std::marker::PhantomData;
use std::sync::Arc;

// Substrate
use scale_codec::{Decode, Encode};
use sp_core::H256;
use sp_database::Database;
use sp_runtime::traits::Block as BlockT;
use uuid::Uuid;

use crate::DbHash;

#[derive(Debug)]
pub enum CairoJobStatus {
    Unknown,
    NotCreated,
    InProgress,
    Processed,
    Onchain,
    Invalid,
    Failed,
}

impl CairoJobStatus {
    fn as_str(&self) -> &'static str {
        match self {
            CairoJobStatus::Unknown => "UNKNOWN",
            CairoJobStatus::NotCreated => "NOT_CREATED",
            CairoJobStatus::InProgress => "IN_PROGRESS",
            CairoJobStatus::Processed => "PROCESSED",
            CairoJobStatus::Onchain => "ONCHAIN",
            CairoJobStatus::Invalid => "INVALID",
            CairoJobStatus::Failed => "FAILED",
        }
    }
}

// The fact db stores DA facts that need to be written to L1
pub struct FactDb<B: BlockT> {
    pub(crate) db: Arc<dyn Database<DbHash>>,
    pub(crate) _marker: PhantomData<B>,
}

// TODO: business logic for last proven and purge
impl<B: BlockT> FactDb<B> {
    pub fn block_facts(&self, block_hash: &B::Hash) -> Result<Vec<H256>, String> {
        match self.db.get(crate::columns::FACT, &block_hash.encode()) {
            Some(raw) => Ok(Vec::<H256>::decode(&mut &raw[..]).map_err(|e| format!("{:?}", e))?),
            None => Ok(Vec::new()),
        }
    }

    pub fn store_block_facts(&self, block_hash: &B::Hash, facts: Vec<H256>) -> Result<(), String> {
        let mut transaction = sp_database::Transaction::new();

        transaction.set(crate::columns::FACT, &block_hash.encode(), &facts.encode());

        self.db.commit(transaction).map_err(|e| format!("{:?}", e))?;

        Ok(())
    }

    pub fn cairo_job(&self, block_hash: &B::Hash) -> Result<Uuid, String> {
        match self.db.get(crate::columns::FACT, &block_hash.encode()) {
            Some(raw) => Ok(Uuid::from_slice(&raw[..]).map_err(|e| format!("{:?}", e))?),
            None => Err(String::from("can't locate cairo job")),
        }
    }

    pub fn update_cairo_job(&self, block_hash: &B::Hash, job_id: Uuid) -> Result<(), String> {
        let mut transaction = sp_database::Transaction::new();

        transaction.set(crate::columns::FACT, &block_hash.encode(), &job_id.into_bytes());

        self.db.commit(transaction).map_err(|e| format!("{:?}", e))?;

        Ok(())
    }

    pub fn last_proved_block(&self) -> Result<B::Hash, String> {
        match self.db.get(crate::columns::FACT, crate::static_keys::LAST_PROVED_BLOCK) {
            Some(raw) => Ok(B::Hash::decode(&mut &raw[..]).map_err(|e| format!("{:?}", e))?),
            None => Err(String::from("can't locate last proved block")),
        }
    }

    pub fn update_last_proved_block(&self, block_hash: &B::Hash) -> Result<(), String> {
        let mut transaction = sp_database::Transaction::new();

        transaction.set(crate::columns::FACT, crate::static_keys::LAST_PROVED_BLOCK, &block_hash.encode());

        self.db.commit(transaction).map_err(|e| format!("{:?}", e))?;

        Ok(())
    }
}
