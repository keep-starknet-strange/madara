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

mod error;
pub use error::DbError;

mod mapping_db;
pub use mapping_db::MappingCommitment;
use sierra_classes_db::SierraClassesDb;
use starknet_api::hash::StarkHash;
mod da_db;
mod db_opening_utils;
mod messaging_db;
mod sierra_classes_db;
pub use messaging_db::LastSyncedEventBlock;
mod l1_handler_tx_fee;
mod meta_db;

use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use da_db::DaDb;
use l1_handler_tx_fee::L1HandlerTxFeeDb;
use mapping_db::MappingDb;
use messaging_db::MessagingDb;
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
    /// Total number of columns.
    // ===== /!\ ===================================================================================
    // MUST BE INCREMENTED WHEN A NEW COLUMN IN ADDED
    // ===== /!\ ===================================================================================
    pub const NUM_COLUMNS: u32 = 9;

    pub const META: u32 = 0;
    pub const BLOCK_MAPPING: u32 = 1;
    pub const TRANSACTION_MAPPING: u32 = 2;
    pub const SYNCED_MAPPING: u32 = 3;
    pub const DA: u32 = 4;

    /// This column is used to map starknet block hashes to a list of transaction hashes that are
    /// contained in the block.
    ///
    /// This column should only be accessed if the `--cache` flag is enabled.
    pub const STARKNET_TRANSACTION_HASHES_CACHE: u32 = 5;

    /// This column contains last synchronized L1 block.
    pub const MESSAGING: u32 = 6;

    /// This column contains the Sierra contract classes
    pub const SIERRA_CONTRACT_CLASSES: u32 = 7;

    /// This column stores the fee paid on l1 for L1Handler transactions
    pub const L1_HANDLER_PAID_FEE: u32 = 8;
}

pub mod static_keys {
    pub const CURRENT_SYNCING_TIPS: &[u8] = b"CURRENT_SYNCING_TIPS";
    pub const LAST_PROVED_BLOCK: &[u8] = b"LAST_PROVED_BLOCK";
    pub const LAST_SYNCED_L1_EVENT_BLOCK: &[u8] = b"LAST_SYNCED_L1_EVENT_BLOCK";
}

/// The Madara client database backend
///
/// Contains two distinct databases: `meta` and `mapping`.
/// `mapping` is used to map Starknet blocks to Substrate ones.
/// `meta` is used to store data about the current state of the chain
pub struct Backend<B: BlockT> {
    meta: Arc<MetaDb<B>>,
    mapping: Arc<MappingDb<B>>,
    da: Arc<DaDb>,
    messaging: Arc<MessagingDb>,
    sierra_classes: Arc<SierraClassesDb>,
    l1_handler_paid_fee: Arc<L1HandlerTxFeeDb>,
}

/// Returns the Starknet database directory.
pub fn starknet_database_dir(db_config_dir: &Path, db_path: &str) -> PathBuf {
    db_config_dir.join("starknet").join(db_path)
}

impl<B: BlockT> Backend<B> {
    /// Open the database
    ///
    /// The database will be created at db_config_dir.join(<db_type_name>)
    pub fn open(database: &DatabaseSource, db_config_dir: &Path, cache_more_things: bool) -> Result<Self, String> {
        Self::new(
            &DatabaseSettings {
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
            },
            cache_more_things,
        )
    }

    fn new(config: &DatabaseSettings, cache_more_things: bool) -> Result<Self, String> {
        let db = db_opening_utils::open_database(config)?;

        Ok(Self {
            mapping: Arc::new(MappingDb::new(db.clone(), cache_more_things)),
            meta: Arc::new(MetaDb { db: db.clone(), _marker: PhantomData }),
            da: Arc::new(DaDb { db: db.clone() }),
            messaging: Arc::new(MessagingDb { db: db.clone() }),
            sierra_classes: Arc::new(SierraClassesDb { db: db.clone() }),
            l1_handler_paid_fee: Arc::new(L1HandlerTxFeeDb { db: db.clone() }),
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

    /// Return the da database manager
    pub fn da(&self) -> &Arc<DaDb> {
        &self.da
    }

    /// Return the da database manager
    pub fn messaging(&self) -> &Arc<MessagingDb> {
        &self.messaging
    }

    /// Return the sierra classes database manager
    pub fn sierra_classes(&self) -> &Arc<SierraClassesDb> {
        &self.sierra_classes
    }

    /// Return l1 handler tx paid fee database manager
    pub fn l1_handler_paid_fee(&self) -> &Arc<L1HandlerTxFeeDb> {
        &self.l1_handler_paid_fee
    }

    /// In the future, we will compute the block global state root asynchronously in the client,
    /// using the Starknet-Bonzai-trie.
    /// That what replaces it for now :)
    pub fn temporary_global_state_root_getter(&self) -> StarkHash {
        Default::default()
    }
}
