// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0
// This file is part of Frontier.
//
// Copyright (c) 2020-2022 Parity Technologies (UK) Ltd.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use std::path::Path;
use std::sync::Arc;

use sp_runtime::traits::Block as BlockT;

use crate::{Database, DatabaseSettings, DatabaseSource, DbHash};

pub fn open_database<Block: BlockT>(config: &DatabaseSettings) -> Result<Arc<dyn Database<DbHash>>, String> {
    let db: Arc<dyn Database<DbHash>> = match &config.source {
        DatabaseSource::ParityDb { path } => open_parity_db::<Block>(path)?,
        DatabaseSource::RocksDb { path, .. } => open_kvdb_rocksdb::<Block>(path, true)?,
        DatabaseSource::Auto { paritydb_path, rocksdb_path, .. } => {
            match open_kvdb_rocksdb::<Block>(rocksdb_path, false) {
                Ok(db) => db,
                Err(_) => open_parity_db::<Block>(paritydb_path)?,
            }
        }
        _ => return Err("Missing feature flags `parity-db`".to_string()),
    };
    Ok(db)
}

#[cfg(feature = "kvdb-rocksdb")]
fn open_kvdb_rocksdb<Block: BlockT>(path: &Path, create: bool) -> Result<Arc<dyn Database<DbHash>>, String> {
    let mut db_config = kvdb_rocksdb::DatabaseConfig::with_columns(crate::columns::NUM_COLUMNS);
    db_config.create_if_missing = create;

    let db = kvdb_rocksdb::Database::open(&db_config, path).map_err(|err| format!("{}", err))?;
    return Ok(sp_database::as_database(db));
}

#[cfg(not(feature = "kvdb-rocksdb"))]
fn open_kvdb_rocksdb<Block: BlockT>(
    _client: Arc<C>,
    _path: &Path,
    _create: bool,
    _source: &DatabaseSource,
) -> Result<Arc<dyn Database<DbHash>>, String> {
    Err("Missing feature flags `kvdb-rocksdb`".to_string())
}

#[cfg(feature = "parity-db")]
fn open_parity_db<Block: BlockT>(path: &Path) -> Result<Arc<dyn Database<DbHash>>, String> {
    let mut config = parity_db::Options::with_columns(path, crate::columns::NUM_COLUMNS as u8);
    config.columns[crate::columns::BLOCK_MAPPING as usize].btree_index = true;

    let db = parity_db::Db::open_or_create(&config).map_err(|err| format!("{}", err))?;
    Ok(Arc::new(crate::parity_db_adapter::DbAdapter(db)))
}

#[cfg(not(feature = "parity-db"))]
fn open_parity_db<Block: BlockT>(_path: &Path) -> Result<Arc<dyn Database<DbHash>>, String> {
    Err("Missing feature flags `parity-db`".to_string())
}
