//! A database backend storing data about madara chain
//!
//! # Usefulness
//! Starknet RPC methods use Starknet block hash as arguments to access on-chain values.
//! Because the Starknet blocks are wrapped inside the Substrate ones, we have no simple way to
//! index the chain storage using this hash.
//! Rather than iterating over all the Substrate blocks in order to find the one wrapping the
//! requested Starknet one, we maintain a StarknetBlockHash to SubstrateBlock hash mapping.
//!
//! # Databases supported
//! `paritydb` and `rocksdb` are both supported, behind the `kvdb-rocksd` and `parity-db` feature
//! flags. Support for custom databases is possible but not supported yet.

mod mapping_db;
pub use mapping_db::MappingCommitment;
mod db_opening_utils;
mod meta_db;

use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use mapping_db::MappingDb;
use meta_db::MetaDb;
use sc_client_db::DatabaseSource;
use sp_database::Database;
use sp_runtime::traits::Block as BlockT;

const DB_HASH_LEN: usize = 32;
/// Hash type that this backend uses for the database.
pub type DbHash = [u8; DB_HASH_LEN];

struct DatabaseSettings {
    /// Where to find the database.
    pub source: DatabaseSource,
}

pub(crate) mod columns {
    pub const NUM_COLUMNS: u32 = 4;

    pub const META: u32 = 0;
    pub const BLOCK_MAPPING: u32 = 1;
    // pub const TRANSACTION_MAPPING: u32 = 2;
    pub const SYNCED_MAPPING: u32 = 3;
}

pub mod static_keys {
    pub const CURRENT_SYNCING_TIPS: &[u8] = b"CURRENT_SYNCING_TIPS";
}

/// The Madara client database backend
///
/// Contains two distinct databases: `meta` and `mapping`.
/// `mapping` is used to map Starknet blocks to Substrate ones.
/// `meta` is used to store data about the current state of the chain
pub struct Backend<B: BlockT> {
    meta: Arc<MetaDb<B>>,
    mapping: Arc<MappingDb<B>>,
}

/// Returns the Starknet database directory.
pub fn starknet_database_dir(db_config_dir: &Path, db_path: &str) -> PathBuf {
    db_config_dir.join("starknet").join(db_path)
}

impl<B: BlockT> Backend<B> {
    /// Open the database
    ///
    /// The database will be created at db_config_dir.join(<db_type_name>)
    pub fn open(database: &DatabaseSource, db_config_dir: &Path) -> Result<Self, String> {
        Self::new(&DatabaseSettings {
            source: match database {
                DatabaseSource::RocksDb { .. } => {
                    DatabaseSource::RocksDb { path: starknet_database_dir(db_config_dir, "rockdb"), cache_size: 0 }
                }
                DatabaseSource::ParityDb { .. } => {
                    DatabaseSource::ParityDb { path: starknet_database_dir(db_config_dir, "paritydb") }
                }
                DatabaseSource::Auto { .. } => DatabaseSource::Auto {
                    rocksdb_path: starknet_database_dir(db_config_dir, "rockdb"),
                    paritydb_path: starknet_database_dir(db_config_dir, "paritydb"),
                    cache_size: 0,
                },
                _ => return Err("Supported db sources: `rocksdb` | `paritydb` | `auto`".to_string()),
            },
        })
    }

    fn new(config: &DatabaseSettings) -> Result<Self, String> {
        let db = db_opening_utils::open_database(config)?;

        Ok(Self {
            mapping: Arc::new(MappingDb { db: db.clone(), write_lock: Arc::new(Mutex::new(())), _marker: PhantomData }),
            meta: Arc::new(MetaDb { db: db.clone(), _marker: PhantomData }),
        })
    }

    /// Return the mapping database manager
    pub fn mapping(&self) -> &Arc<MappingDb<B>> {
        &self.mapping
    }

    /// Return the meta database manager
    pub fn meta(&self) -> &Arc<MetaDb<B>> {
        &self.meta
    }
}
