//! Read-only access to Madara's RocksDB database

use rocksdb::{DBWithThreadMode, MultiThreaded, Options};
use std::path::{Path, PathBuf};
use thiserror::Error;

pub mod blocks;
pub mod contracts;
mod queries;
pub mod raw;
pub mod state_diff;
pub mod transactions;

pub use blocks::*;
pub use contracts::*;
pub use queries::*;
pub use raw::*;
pub use state_diff::*;
pub use transactions::*;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("RocksDB error: {0}")]
    RocksDb(#[from] rocksdb::Error),
    #[error("Database path does not exist: {0}")]
    PathNotFound(PathBuf),
    #[error("Deserialization error: {0}")]
    Deserialize(String),
}

type DB = DBWithThreadMode<MultiThreaded>;

/// Read-only handle to a Madara RocksDB database
pub struct DbReader {
    db: DB,
    path: PathBuf,
}

impl DbReader {
    /// Open a RocksDB database in read-only mode
    pub fn open(path: impl AsRef<Path>) -> Result<Self, DbError> {
        let path = path.as_ref().to_path_buf();

        if !path.exists() {
            return Err(DbError::PathNotFound(path));
        }

        let mut opts = Options::default();
        opts.set_max_open_files(100);

        // List existing column families
        let cf_names = DB::list_cf(&opts, &path).unwrap_or_default();

        // Open in read-only mode with all existing column families
        let db = DB::open_cf_for_read_only(&opts, &path, cf_names.iter(), false)?;

        Ok(Self { db, path })
    }

    /// Get the database path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// List all column family names
    pub fn column_families(&self) -> Vec<String> {
        let opts = Options::default();
        DB::list_cf(&opts, &self.path).unwrap_or_default()
    }

    /// Get raw access to the underlying RocksDB handle
    pub fn raw_db(&self) -> &DB {
        &self.db
    }
}
