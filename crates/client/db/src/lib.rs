// This is *heavely* inspired by https://github.com/paritytech/frontier/blob/master/client/db/src/lib.rs

mod parity_db_adapter;
mod utils;

use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

// Substrate
use sc_client_db::DatabaseSource;
use scale_codec::{Decode, Encode};
use sp_core::H256;
use sp_database::Database;
use sp_runtime::traits::Block as BlockT;

const DB_HASH_LEN: usize = 32;
/// Hash type that this backend uses for the database.
pub type DbHash = [u8; DB_HASH_LEN];

/// Database settings.
pub struct DatabaseSettings {
    /// Where to find the database.
    pub source: DatabaseSource,
}

pub(crate) mod columns {
    pub const NUM_COLUMNS: u32 = 3;

    pub const BLOCK_MAPPING: u32 = 0;
    pub const TRANSACTION_MAPPING: u32 = 1;
    pub const SYNCED_MAPPING: u32 = 2;
}

pub struct Backend<Block: BlockT> {
    mapping: Arc<MappingDb<Block>>,
}

/// Returns the frontier database directory.
pub fn frontier_database_dir(db_config_dir: &Path, db_path: &str) -> PathBuf {
    db_config_dir.join("starknet").join(db_path)
}

impl<Block: BlockT> Backend<Block> {
    pub fn open(database: &DatabaseSource, db_config_dir: &Path) -> Result<Self, String> {
        Self::new(&DatabaseSettings {
            source: match database {
                DatabaseSource::RocksDb { .. } => {
                    DatabaseSource::RocksDb { path: frontier_database_dir(db_config_dir, "db"), cache_size: 0 }
                }
                DatabaseSource::ParityDb { .. } => {
                    DatabaseSource::ParityDb { path: frontier_database_dir(db_config_dir, "paritydb") }
                }
                DatabaseSource::Auto { .. } => DatabaseSource::Auto {
                    rocksdb_path: frontier_database_dir(db_config_dir, "db"),
                    paritydb_path: frontier_database_dir(db_config_dir, "paritydb"),
                    cache_size: 0,
                },
                _ => return Err("Supported db sources: `rocksdb` | `paritydb` | `auto`".to_string()),
            },
        })
    }

    pub fn new(config: &DatabaseSettings) -> Result<Self, String> {
        let db = utils::open_database::<Block>(config)?;

        Ok(Self {
            mapping: Arc::new(MappingDb { db: db.clone(), write_lock: Arc::new(Mutex::new(())), _marker: PhantomData }),
        })
    }

    pub fn mapping(&self) -> &Arc<MappingDb<Block>> {
        &self.mapping
    }
}

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
    db: Arc<dyn Database<DbHash>>,
    write_lock: Arc<Mutex<()>>,
    _marker: PhantomData<Block>,
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
