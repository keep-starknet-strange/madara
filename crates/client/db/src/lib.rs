// This is *heavely* inspired by https://github.com/paritytech/frontier/blob/master/client/db/src/lib.rs

mod mapping_db;
pub use mapping_db::MappingCommitment;
mod meta_db;
mod parity_db_adapter;
mod utils;

use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use mapping_db::MappingDb;
use meta_db::MetaDb;
// Substrate
use sc_client_db::DatabaseSource;
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
    pub const NUM_COLUMNS: u32 = 4;

    pub const META: u32 = 0;
    pub const BLOCK_MAPPING: u32 = 1;
    pub const TRANSACTION_MAPPING: u32 = 2;
    pub const SYNCED_MAPPING: u32 = 3;
}

pub mod static_keys {
    pub const CURRENT_SYNCING_TIPS: &[u8] = b"CURRENT_SYNCING_TIPS";
}

pub struct Backend<Block: BlockT> {
    meta: Arc<MetaDb<Block>>,
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
            meta: Arc::new(MetaDb { db: db.clone(), _marker: PhantomData }),
        })
    }

    pub fn mapping(&self) -> &Arc<MappingDb<Block>> {
        &self.mapping
    }

    pub fn meta(&self) -> &Arc<MetaDb<Block>> {
        &self.meta
    }
}
