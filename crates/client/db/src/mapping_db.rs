use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

// Substrate
use scale_codec::{Decode, Encode};
use sp_core::H256;
use sp_database::Database;
use sp_runtime::traits::Block as BlockT;

use crate::DbHash;

#[derive(Clone, Debug, Eq, PartialEq, Encode, Decode)]
pub struct TransactionMetadata<Block: BlockT> {
    pub block_hash: Block::Hash,
    pub starknet_block_hash: H256,
    pub starknet_index: u32,
}

#[derive(Debug)]
pub struct MappingCommitment<Block: BlockT> {
    pub block_hash: Block::Hash,
    pub starknet_block_hash: H256,
    pub starknet_transaction_hashes: Vec<H256>,
}

pub struct MappingDb<Block: BlockT> {
    pub(crate) db: Arc<dyn Database<DbHash>>,
    pub(crate) write_lock: Arc<Mutex<()>>,
    pub(crate) _marker: PhantomData<Block>,
}

impl<Block: BlockT> MappingDb<Block> {
    pub fn is_synced(&self, block_hash: &Block::Hash) -> Result<bool, String> {
        match self.db.get(crate::columns::SYNCED_MAPPING, &block_hash.encode()) {
            Some(raw) => Ok(bool::decode(&mut &raw[..]).map_err(|e| format!("{:?}", e))?),
            None => Ok(false),
        }
    }

    pub fn block_hash(&self, starknet_block_hash: &H256) -> Result<Option<Vec<Block::Hash>>, String> {
        match self.db.get(crate::columns::BLOCK_MAPPING, &starknet_block_hash.encode()) {
            Some(raw) => Ok(Some(Vec::<Block::Hash>::decode(&mut &raw[..]).map_err(|e| format!("{:?}", e))?)),
            None => Ok(None),
        }
    }

    pub fn transaction_metadata(
        &self,
        starknet_transaction_hash: &H256,
    ) -> Result<Vec<TransactionMetadata<Block>>, String> {
        match self.db.get(crate::columns::TRANSACTION_MAPPING, &starknet_transaction_hash.encode()) {
            Some(raw) => Ok(Vec::<TransactionMetadata<Block>>::decode(&mut &raw[..]).map_err(|e| format!("{:?}", e))?),
            None => Ok(Vec::new()),
        }
    }

    pub fn write_none(&self, block_hash: Block::Hash) -> Result<(), String> {
        let _lock = self.write_lock.lock();

        let mut transaction = sp_database::Transaction::new();

        transaction.set(crate::columns::SYNCED_MAPPING, &block_hash.encode(), &true.encode());

        self.db.commit(transaction).map_err(|e| format!("{:?}", e))?;

        Ok(())
    }

    pub fn write_hashes(&self, commitment: MappingCommitment<Block>) -> Result<(), String> {
        let _lock = self.write_lock.lock();

        let mut transaction = sp_database::Transaction::new();

        let substrate_hashes = match self.block_hash(&commitment.starknet_block_hash) {
            Ok(Some(mut data)) => {
                data.push(commitment.block_hash);
                log::warn!(
                    target: "fc-db",
                    "Possible equivocation at starknet block hash {} {:?}",
                    &commitment.starknet_block_hash,
                    &data
                );
                data
            }
            _ => vec![commitment.block_hash],
        };

        transaction.set(
            crate::columns::BLOCK_MAPPING,
            &commitment.starknet_block_hash.encode(),
            &substrate_hashes.encode(),
        );

        for (i, starknet_transaction_hash) in commitment.starknet_transaction_hashes.into_iter().enumerate() {
            let mut metadata = self.transaction_metadata(&starknet_transaction_hash)?;
            metadata.push(TransactionMetadata::<Block> {
                block_hash: commitment.block_hash,
                starknet_block_hash: commitment.starknet_block_hash,
                starknet_index: i as u32,
            });
            transaction.set(
                crate::columns::TRANSACTION_MAPPING,
                &starknet_transaction_hash.encode(),
                &metadata.encode(),
            );
        }

        transaction.set(crate::columns::SYNCED_MAPPING, &commitment.block_hash.encode(), &true.encode());

        self.db.commit(transaction).map_err(|e| format!("{:?}", e))?;

        Ok(())
    }
}
