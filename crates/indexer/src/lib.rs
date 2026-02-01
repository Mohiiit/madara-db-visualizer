//! SQLite indexer for complex queries on Madara DB

use db_reader::DbReader;
use hex;
use rusqlite::{params, Connection};
use std::path::Path;
use thiserror::Error;

/// Current schema version - increment when schema changes
const SCHEMA_VERSION: u32 = 2;

#[derive(Error, Debug)]
pub enum IndexerError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Index status information
#[derive(Debug, Clone)]
pub struct IndexStatus {
    pub indexed_blocks: u64,
    pub latest_block: u64,
    pub is_synced: bool,
    pub total_transactions: u64,
    pub failed_transactions: u64,
    pub total_events: u64,
    pub total_storage_updates: u64,
    pub total_deployed_contracts: u64,
}

/// Transaction record for queries
#[derive(Debug, Clone)]
pub struct IndexedTransaction {
    pub tx_hash: String,
    pub block_number: u64,
    pub tx_index: u64,
    pub tx_type: String,
    pub version: Option<String>,
    pub status: String,
    pub revert_reason: Option<String>,
    pub sender_address: Option<String>,
    pub nonce: Option<String>,
    pub actual_fee: Option<String>,
    pub fee_unit: Option<String>,
    pub max_fee: Option<String>,
    pub calldata_length: Option<i64>,
    pub signature_length: Option<i64>,
}

/// Contract record for queries
#[derive(Debug, Clone)]
pub struct IndexedContract {
    pub address: String,
    pub class_hash: Option<String>,
    pub nonce: Option<u64>,
}

/// Block record for queries
#[derive(Debug, Clone)]
pub struct IndexedBlock {
    pub block_number: u64,
    pub block_hash: String,
    pub parent_hash: String,
    pub state_root: Option<String>,
    pub sequencer_address: Option<String>,
    pub timestamp: Option<i64>,
    pub transaction_count: Option<i64>,
    pub event_count: Option<i64>,
    pub l1_gas_price: Option<String>,
    pub l1_data_gas_price: Option<String>,
}

/// Event record for queries
#[derive(Debug, Clone)]
pub struct IndexedEvent {
    pub id: i64,
    pub tx_hash: String,
    pub block_number: u64,
    pub event_index: i64,
    pub from_address: String,
    pub keys_count: Option<i64>,
    pub data_count: Option<i64>,
    pub key_0: Option<String>,
    pub key_1: Option<String>,
}

/// Storage update record for queries
#[derive(Debug, Clone)]
pub struct StorageUpdate {
    pub id: i64,
    pub block_number: u64,
    pub contract_address: String,
    pub storage_key: String,
    pub storage_value: String,
}

/// Deployed contract record for queries
#[derive(Debug, Clone)]
pub struct IndexedDeployedContract {
    pub id: i64,
    pub block_number: u64,
    pub contract_address: String,
    pub class_hash: String,
}

/// Class record for queries
#[derive(Debug, Clone)]
pub struct IndexedClass {
    pub class_hash: String,
    pub class_type: String,
    pub compiled_class_hash: Option<String>,
    pub declared_at_block: Option<i64>,
}

/// SQLite-based indexer for complex queries
pub struct Indexer {
    conn: Connection,
}

impl Indexer {
    /// Create or open an indexer database
    pub fn open(path: impl AsRef<Path>) -> Result<Self, IndexerError> {
        let conn = Connection::open(path)?;
        let indexer = Self { conn };
        indexer.init_schema()?;
        Ok(indexer)
    }

    /// Create an in-memory indexer (for testing)
    pub fn in_memory() -> Result<Self, IndexerError> {
        let conn = Connection::open_in_memory()?;
        let indexer = Self { conn };
        indexer.init_schema()?;
        Ok(indexer)
    }

    /// Check and handle schema migration
    fn check_schema_version(&self) -> Result<bool, IndexerError> {
        // Try to get current schema version
        let version: Result<u32, _> = self.conn.query_row(
            "SELECT schema_version FROM index_status WHERE id = 1",
            [],
            |row| row.get(0),
        );

        match version {
            Ok(v) if v == SCHEMA_VERSION => Ok(false), // No migration needed
            Ok(_) => Ok(true),                          // Migration needed
            Err(_) => Ok(true),                         // Table doesn't exist or no column
        }
    }

    /// Drop all tables for schema migration
    fn drop_all_tables(&self) -> Result<(), IndexerError> {
        self.conn.execute_batch(
            r#"
            DROP TABLE IF EXISTS events;
            DROP TABLE IF EXISTS storage_updates;
            DROP TABLE IF EXISTS deployed_contracts;
            DROP TABLE IF EXISTS transactions;
            DROP TABLE IF EXISTS blocks;
            DROP TABLE IF EXISTS classes;
            DROP TABLE IF EXISTS contracts;
            DROP TABLE IF EXISTS index_status;
            "#,
        )?;
        Ok(())
    }

    /// Initialize the database schema
    fn init_schema(&self) -> Result<(), IndexerError> {
        // Check if we need schema migration
        let needs_migration = self.check_schema_version()?;

        if needs_migration {
            // Drop and recreate all tables
            self.drop_all_tables()?;
        }

        self.conn.execute_batch(
            r#"
            -- Blocks table (expanded)
            CREATE TABLE IF NOT EXISTS blocks (
                block_number INTEGER PRIMARY KEY,
                block_hash TEXT NOT NULL,
                parent_hash TEXT NOT NULL,
                state_root TEXT,
                sequencer_address TEXT,
                timestamp INTEGER,
                transaction_count INTEGER,
                event_count INTEGER,
                l1_gas_price TEXT,
                l1_data_gas_price TEXT
            );

            -- Transactions table (expanded)
            CREATE TABLE IF NOT EXISTS transactions (
                tx_hash TEXT PRIMARY KEY,
                block_number INTEGER NOT NULL,
                tx_index INTEGER NOT NULL,
                tx_type TEXT NOT NULL,
                version TEXT,
                status TEXT NOT NULL,
                revert_reason TEXT,
                sender_address TEXT,
                nonce TEXT,
                actual_fee TEXT,
                fee_unit TEXT,
                max_fee TEXT,
                calldata_length INTEGER,
                signature_length INTEGER,
                FOREIGN KEY (block_number) REFERENCES blocks(block_number)
            );

            -- Events table
            CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tx_hash TEXT NOT NULL,
                block_number INTEGER NOT NULL,
                event_index INTEGER NOT NULL,
                from_address TEXT NOT NULL,
                keys_count INTEGER,
                data_count INTEGER,
                key_0 TEXT,
                key_1 TEXT,
                FOREIGN KEY (tx_hash) REFERENCES transactions(tx_hash),
                FOREIGN KEY (block_number) REFERENCES blocks(block_number)
            );

            -- Storage updates table
            CREATE TABLE IF NOT EXISTS storage_updates (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                block_number INTEGER NOT NULL,
                contract_address TEXT NOT NULL,
                storage_key TEXT NOT NULL,
                storage_value TEXT NOT NULL,
                FOREIGN KEY (block_number) REFERENCES blocks(block_number)
            );

            -- Deployed contracts table
            CREATE TABLE IF NOT EXISTS deployed_contracts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                block_number INTEGER NOT NULL,
                contract_address TEXT NOT NULL,
                class_hash TEXT NOT NULL,
                FOREIGN KEY (block_number) REFERENCES blocks(block_number)
            );

            -- Classes table (expanded)
            CREATE TABLE IF NOT EXISTS classes (
                class_hash TEXT PRIMARY KEY,
                class_type TEXT NOT NULL,
                compiled_class_hash TEXT,
                declared_at_block INTEGER
            );

            -- Contracts table (kept for backward compatibility)
            CREATE TABLE IF NOT EXISTS contracts (
                address TEXT PRIMARY KEY,
                class_hash TEXT,
                nonce INTEGER
            );

            -- Index status table
            CREATE TABLE IF NOT EXISTS index_status (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                indexed_blocks INTEGER NOT NULL DEFAULT 0,
                latest_block INTEGER NOT NULL DEFAULT 0,
                schema_version INTEGER NOT NULL DEFAULT 0
            );

            -- Indexes for common queries
            CREATE INDEX IF NOT EXISTS idx_tx_status ON transactions(status);
            CREATE INDEX IF NOT EXISTS idx_tx_sender ON transactions(sender_address);
            CREATE INDEX IF NOT EXISTS idx_tx_type ON transactions(tx_type);
            CREATE INDEX IF NOT EXISTS idx_tx_block ON transactions(block_number);
            CREATE INDEX IF NOT EXISTS idx_events_address ON events(from_address);
            CREATE INDEX IF NOT EXISTS idx_events_key0 ON events(key_0);
            CREATE INDEX IF NOT EXISTS idx_events_block ON events(block_number);
            CREATE INDEX IF NOT EXISTS idx_events_tx ON events(tx_hash);
            CREATE INDEX IF NOT EXISTS idx_storage_contract ON storage_updates(contract_address);
            CREATE INDEX IF NOT EXISTS idx_storage_block ON storage_updates(block_number);
            CREATE INDEX IF NOT EXISTS idx_deployed_block ON deployed_contracts(block_number);
            CREATE INDEX IF NOT EXISTS idx_deployed_address ON deployed_contracts(contract_address);
            CREATE INDEX IF NOT EXISTS idx_contract_class ON contracts(class_hash);
            CREATE INDEX IF NOT EXISTS idx_blocks_hash ON blocks(block_hash);
            CREATE INDEX IF NOT EXISTS idx_blocks_timestamp ON blocks(timestamp);
            "#,
        )?;

        // Insert or update schema version
        self.conn.execute(
            "INSERT OR REPLACE INTO index_status (id, indexed_blocks, latest_block, schema_version)
             VALUES (1,
                     COALESCE((SELECT indexed_blocks FROM index_status WHERE id = 1), 0),
                     COALESCE((SELECT latest_block FROM index_status WHERE id = 1), 0),
                     ?1)",
            params![SCHEMA_VERSION],
        )?;

        Ok(())
    }

    /// Get current index status
    pub fn get_status(&self) -> Result<IndexStatus, IndexerError> {
        let (indexed_blocks, latest_block): (u64, u64) = self.conn.query_row(
            "SELECT indexed_blocks, latest_block FROM index_status WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        let total_transactions: u64 = self.conn.query_row(
            "SELECT COUNT(*) FROM transactions",
            [],
            |row| row.get(0),
        )?;

        let failed_transactions: u64 = self.conn.query_row(
            "SELECT COUNT(*) FROM transactions WHERE status = 'REVERTED'",
            [],
            |row| row.get(0),
        )?;

        let total_events: u64 = self.conn.query_row(
            "SELECT COUNT(*) FROM events",
            [],
            |row| row.get(0),
        )?;

        let total_storage_updates: u64 = self.conn.query_row(
            "SELECT COUNT(*) FROM storage_updates",
            [],
            |row| row.get(0),
        )?;

        let total_deployed_contracts: u64 = self.conn.query_row(
            "SELECT COUNT(*) FROM deployed_contracts",
            [],
            |row| row.get(0),
        )?;

        Ok(IndexStatus {
            indexed_blocks,
            latest_block,
            is_synced: indexed_blocks > 0 && indexed_blocks >= latest_block,
            total_transactions,
            failed_transactions,
            total_events,
            total_storage_updates,
            total_deployed_contracts,
        })
    }

    /// Sync index from RocksDB
    pub fn sync_from_db(&mut self, db: &DbReader) -> Result<u64, IndexerError> {
        let latest_block = db.get_latest_block_number().unwrap_or(0);
        let current_indexed: u64 = self.conn.query_row(
            "SELECT indexed_blocks FROM index_status WHERE id = 1",
            [],
            |row| row.get(0),
        )?;

        if current_indexed >= latest_block {
            // Already synced
            return Ok(0);
        }

        let start_block = current_indexed;
        let mut indexed_count = 0u64;

        // Begin transaction for batch insert
        let tx = self.conn.transaction()?;

        for block_n in start_block..=latest_block {
            // Index block info
            if let Some(block_detail) = db.get_block_detail(block_n) {
                tx.execute(
                    "INSERT OR REPLACE INTO blocks (block_number, block_hash, parent_hash, state_root, sequencer_address, timestamp, transaction_count, event_count) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![
                        block_n,
                        block_detail.block_hash,
                        block_detail.parent_hash,
                        block_detail.state_root,
                        block_detail.sequencer_address,
                        block_detail.timestamp as i64,
                        block_detail.transaction_count as i64,
                        block_detail.event_count as i64,
                    ],
                )?;
            }

            // Index transactions from this block
            let block = db.get_block_detail(block_n);
            if let Some(block_info) = &block {
                for (tx_idx, tx_hash) in block_info.tx_hashes.iter().enumerate() {
                    // Try to get full transaction details
                    let tx_detail = db.get_transaction_detail(block_n, tx_idx as u64);

                    let (tx_type, status, revert_reason, sender, version, actual_fee, fee_unit, nonce, calldata_len, sig_len) =
                        if let Some(ref detail) = tx_detail {
                            let status_str = match &detail.status {
                                db_reader::ExecutionStatus::Succeeded => "SUCCEEDED",
                                db_reader::ExecutionStatus::Reverted(_) => "REVERTED",
                            };
                            let revert = match &detail.status {
                                db_reader::ExecutionStatus::Reverted(reason) => Some(reason.clone()),
                                _ => None,
                            };
                            (
                                detail.tx_type.to_string(),
                                status_str.to_string(),
                                revert,
                                detail.sender_address.clone(),
                                detail.version.clone(),
                                Some(detail.actual_fee.clone()),
                                Some(detail.fee_unit.clone()),
                                detail.nonce.clone(),
                                Some(detail.calldata.len() as i64),
                                Some(detail.signature.len() as i64),
                            )
                        } else {
                            ("UNKNOWN".to_string(), "UNKNOWN".to_string(), None, None, None, None, None, None, None, None)
                        };

                    tx.execute(
                        "INSERT OR REPLACE INTO transactions (tx_hash, block_number, tx_index, tx_type, version, status, revert_reason, sender_address, nonce, actual_fee, fee_unit, calldata_length, signature_length) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                        params![
                            tx_hash,
                            block_n,
                            tx_idx as i64,
                            tx_type,
                            version,
                            status,
                            revert_reason,
                            sender,
                            nonce,
                            actual_fee,
                            fee_unit,
                            calldata_len,
                            sig_len,
                        ],
                    )?;

                    // Index events from this transaction
                    if let Some(ref detail) = tx_detail {
                        for (event_idx, event) in detail.events.iter().enumerate() {
                            let key_0 = event.keys.first().cloned();
                            let key_1 = event.keys.get(1).cloned();

                            tx.execute(
                                "INSERT INTO events (tx_hash, block_number, event_index, from_address, keys_count, data_count, key_0, key_1) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                                params![
                                    tx_hash,
                                    block_n,
                                    event_idx as i64,
                                    event.from_address,
                                    event.keys.len() as i64,
                                    event.data.len() as i64,
                                    key_0,
                                    key_1,
                                ],
                            )?;
                        }
                    }
                }
            }

            // Index state diff (storage updates, deployed contracts, classes)
            if let Some(state_diff) = db.get_state_diff(block_n) {
                // Index storage updates
                for storage_diff in &state_diff.storage_diffs {
                    for entry in &storage_diff.storage_entries {
                        tx.execute(
                            "INSERT INTO storage_updates (block_number, contract_address, storage_key, storage_value) VALUES (?1, ?2, ?3, ?4)",
                            params![
                                block_n,
                                storage_diff.address,
                                entry.key,
                                entry.value,
                            ],
                        )?;
                    }
                }

                // Index deployed contracts
                for deployed in &state_diff.deployed_contracts {
                    tx.execute(
                        "INSERT INTO deployed_contracts (block_number, contract_address, class_hash) VALUES (?1, ?2, ?3)",
                        params![
                            block_n,
                            deployed.address,
                            deployed.class_hash,
                        ],
                    )?;
                }

                // Index declared classes
                for declared in &state_diff.declared_classes {
                    tx.execute(
                        "INSERT OR REPLACE INTO classes (class_hash, class_type, compiled_class_hash, declared_at_block) VALUES (?1, ?2, ?3, ?4)",
                        params![
                            declared.class_hash,
                            "SIERRA", // Declared classes in state diff are Sierra classes
                            declared.compiled_class_hash,
                            block_n,
                        ],
                    )?;
                }
            }

            indexed_count += 1;

            // Update progress every 10 blocks
            if block_n % 10 == 0 {
                tx.execute(
                    "UPDATE index_status SET indexed_blocks = ?1, latest_block = ?2 WHERE id = 1",
                    params![block_n + 1, latest_block],
                )?;
            }
        }

        // Index contracts
        let contracts = db.list_contracts(10000); // Get all contracts
        for contract in contracts {
            tx.execute(
                "INSERT OR REPLACE INTO contracts (address, class_hash, nonce) VALUES (?1, ?2, ?3)",
                params![
                    contract.address,
                    contract.class_hash,
                    contract.nonce.map(|n| n as i64),
                ],
            )?;
        }

        // Index classes from class_info column family
        let classes = db.list_classes(10000);
        for class in classes {
            // Only insert if not already exists (to preserve declared_at_block)
            tx.execute(
                "INSERT OR IGNORE INTO classes (class_hash, class_type, compiled_class_hash) VALUES (?1, ?2, ?3)",
                params![
                    class.class_hash,
                    class.class_type.to_string(),
                    class.compiled_class_hash,
                ],
            )?;
        }

        // Final status update
        tx.execute(
            "UPDATE index_status SET indexed_blocks = ?1, latest_block = ?2 WHERE id = 1",
            params![latest_block + 1, latest_block],
        )?;

        tx.commit()?;

        Ok(indexed_count)
    }

    /// Query transactions with filters
    pub fn query_transactions(
        &self,
        status: Option<&str>,
        sender: Option<&str>,
        block_from: Option<u64>,
        block_to: Option<u64>,
        limit: usize,
    ) -> Result<Vec<IndexedTransaction>, IndexerError> {
        let mut sql = String::from("SELECT tx_hash, block_number, tx_index, tx_type, version, status, revert_reason, sender_address, nonce, actual_fee, fee_unit, max_fee, calldata_length, signature_length FROM transactions WHERE 1=1");
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(s) = status {
            sql.push_str(" AND status = ?");
            params_vec.push(Box::new(s.to_string()));
        }

        if let Some(s) = sender {
            sql.push_str(" AND sender_address = ?");
            params_vec.push(Box::new(s.to_string()));
        }

        if let Some(from) = block_from {
            sql.push_str(" AND block_number >= ?");
            params_vec.push(Box::new(from as i64));
        }

        if let Some(to) = block_to {
            sql.push_str(" AND block_number <= ?");
            params_vec.push(Box::new(to as i64));
        }

        sql.push_str(" ORDER BY block_number DESC, tx_index DESC LIMIT ?");
        params_vec.push(Box::new(limit as i64));

        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(IndexedTransaction {
                tx_hash: row.get(0)?,
                block_number: row.get(1)?,
                tx_index: row.get(2)?,
                tx_type: row.get(3)?,
                version: row.get(4)?,
                status: row.get(5)?,
                revert_reason: row.get(6)?,
                sender_address: row.get(7)?,
                nonce: row.get(8)?,
                actual_fee: row.get(9)?,
                fee_unit: row.get(10)?,
                max_fee: row.get(11)?,
                calldata_length: row.get(12)?,
                signature_length: row.get(13)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Query contracts with filters
    pub fn query_contracts(
        &self,
        class_hash: Option<&str>,
        limit: usize,
    ) -> Result<Vec<IndexedContract>, IndexerError> {
        let mut sql = String::from("SELECT address, class_hash, nonce FROM contracts WHERE 1=1");
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(hash) = class_hash {
            sql.push_str(" AND class_hash = ?");
            params_vec.push(Box::new(hash.to_string()));
        }

        sql.push_str(" LIMIT ?");
        params_vec.push(Box::new(limit as i64));

        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(IndexedContract {
                address: row.get(0)?,
                class_hash: row.get(1)?,
                nonce: row.get::<_, Option<i64>>(2)?.map(|n| n as u64),
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Query blocks with filters
    pub fn query_blocks(
        &self,
        block_from: Option<u64>,
        block_to: Option<u64>,
        limit: usize,
    ) -> Result<Vec<IndexedBlock>, IndexerError> {
        let mut sql = String::from("SELECT block_number, block_hash, parent_hash, state_root, sequencer_address, timestamp, transaction_count, event_count, l1_gas_price, l1_data_gas_price FROM blocks WHERE 1=1");
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(from) = block_from {
            sql.push_str(" AND block_number >= ?");
            params_vec.push(Box::new(from as i64));
        }

        if let Some(to) = block_to {
            sql.push_str(" AND block_number <= ?");
            params_vec.push(Box::new(to as i64));
        }

        sql.push_str(" ORDER BY block_number DESC LIMIT ?");
        params_vec.push(Box::new(limit as i64));

        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(IndexedBlock {
                block_number: row.get(0)?,
                block_hash: row.get(1)?,
                parent_hash: row.get(2)?,
                state_root: row.get(3)?,
                sequencer_address: row.get(4)?,
                timestamp: row.get(5)?,
                transaction_count: row.get(6)?,
                event_count: row.get(7)?,
                l1_gas_price: row.get(8)?,
                l1_data_gas_price: row.get(9)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Query events with filters
    pub fn query_events(
        &self,
        from_address: Option<&str>,
        key_0: Option<&str>,
        limit: usize,
    ) -> Result<Vec<IndexedEvent>, IndexerError> {
        let mut sql = String::from("SELECT id, tx_hash, block_number, event_index, from_address, keys_count, data_count, key_0, key_1 FROM events WHERE 1=1");
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(addr) = from_address {
            sql.push_str(" AND from_address = ?");
            params_vec.push(Box::new(addr.to_string()));
        }

        if let Some(key) = key_0 {
            sql.push_str(" AND key_0 = ?");
            params_vec.push(Box::new(key.to_string()));
        }

        sql.push_str(" ORDER BY block_number DESC, event_index DESC LIMIT ?");
        params_vec.push(Box::new(limit as i64));

        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(IndexedEvent {
                id: row.get(0)?,
                tx_hash: row.get(1)?,
                block_number: row.get(2)?,
                event_index: row.get(3)?,
                from_address: row.get(4)?,
                keys_count: row.get(5)?,
                data_count: row.get(6)?,
                key_0: row.get(7)?,
                key_1: row.get(8)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Query storage updates with filters
    pub fn query_storage_updates(
        &self,
        contract: Option<&str>,
        block_from: Option<u64>,
        block_to: Option<u64>,
    ) -> Result<Vec<StorageUpdate>, IndexerError> {
        let mut sql = String::from("SELECT id, block_number, contract_address, storage_key, storage_value FROM storage_updates WHERE 1=1");
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(addr) = contract {
            sql.push_str(" AND contract_address = ?");
            params_vec.push(Box::new(addr.to_string()));
        }

        if let Some(from) = block_from {
            sql.push_str(" AND block_number >= ?");
            params_vec.push(Box::new(from as i64));
        }

        if let Some(to) = block_to {
            sql.push_str(" AND block_number <= ?");
            params_vec.push(Box::new(to as i64));
        }

        sql.push_str(" ORDER BY block_number DESC, id DESC LIMIT 1000");

        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(StorageUpdate {
                id: row.get(0)?,
                block_number: row.get(1)?,
                contract_address: row.get(2)?,
                storage_key: row.get(3)?,
                storage_value: row.get(4)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Query deployed contracts with filters
    pub fn query_deployed_contracts(
        &self,
        block_from: Option<u64>,
        block_to: Option<u64>,
        limit: usize,
    ) -> Result<Vec<IndexedDeployedContract>, IndexerError> {
        let mut sql = String::from("SELECT id, block_number, contract_address, class_hash FROM deployed_contracts WHERE 1=1");
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(from) = block_from {
            sql.push_str(" AND block_number >= ?");
            params_vec.push(Box::new(from as i64));
        }

        if let Some(to) = block_to {
            sql.push_str(" AND block_number <= ?");
            params_vec.push(Box::new(to as i64));
        }

        sql.push_str(" ORDER BY block_number DESC LIMIT ?");
        params_vec.push(Box::new(limit as i64));

        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(IndexedDeployedContract {
                id: row.get(0)?,
                block_number: row.get(1)?,
                contract_address: row.get(2)?,
                class_hash: row.get(3)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Query classes with filters
    pub fn query_classes(
        &self,
        class_type: Option<&str>,
        limit: usize,
    ) -> Result<Vec<IndexedClass>, IndexerError> {
        let mut sql = String::from("SELECT class_hash, class_type, compiled_class_hash, declared_at_block FROM classes WHERE 1=1");
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ctype) = class_type {
            sql.push_str(" AND class_type = ?");
            params_vec.push(Box::new(ctype.to_string()));
        }

        sql.push_str(" ORDER BY declared_at_block DESC NULLS LAST LIMIT ?");
        params_vec.push(Box::new(limit as i64));

        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(IndexedClass {
                class_hash: row.get(0)?,
                class_type: row.get(1)?,
                compiled_class_hash: row.get(2)?,
                declared_at_block: row.get(3)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Execute a raw SQL query and return results as JSON-serializable rows
    pub fn execute_raw_query(&self, sql: &str) -> Result<Vec<Vec<(String, String)>>, IndexerError> {
        // Only allow SELECT queries for safety
        let sql_upper = sql.trim().to_uppercase();
        if !sql_upper.starts_with("SELECT") {
            return Err(IndexerError::Sqlite(rusqlite::Error::InvalidQuery));
        }

        let mut stmt = self.conn.prepare(sql)?;
        let column_count = stmt.column_count();
        let column_names: Vec<String> = (0..column_count)
            .map(|i| stmt.column_name(i).unwrap_or("?").to_string())
            .collect();

        let mut rows = stmt.query([])?;
        let mut results = Vec::new();

        while let Some(row) = rows.next()? {
            let mut row_data = Vec::new();
            for (i, name) in column_names.iter().enumerate() {
                let value: String = match row.get_ref(i)? {
                    rusqlite::types::ValueRef::Null => "NULL".to_string(),
                    rusqlite::types::ValueRef::Integer(i) => i.to_string(),
                    rusqlite::types::ValueRef::Real(r) => r.to_string(),
                    rusqlite::types::ValueRef::Text(t) => String::from_utf8_lossy(t).to_string(),
                    rusqlite::types::ValueRef::Blob(b) => format!("0x{}", hex::encode(b)),
                };
                row_data.push((name.clone(), value));
            }
            results.push(row_data);
        }

        Ok(results)
    }

    /// Get events for a specific transaction
    pub fn get_events_for_tx(&self, tx_hash: &str) -> Result<Vec<IndexedEvent>, IndexerError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, tx_hash, block_number, event_index, from_address, keys_count, data_count, key_0, key_1 FROM events WHERE tx_hash = ? ORDER BY event_index ASC"
        )?;

        let rows = stmt.query_map([tx_hash], |row| {
            Ok(IndexedEvent {
                id: row.get(0)?,
                tx_hash: row.get(1)?,
                block_number: row.get(2)?,
                event_index: row.get(3)?,
                from_address: row.get(4)?,
                keys_count: row.get(5)?,
                data_count: row.get(6)?,
                key_0: row.get(7)?,
                key_1: row.get(8)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Get storage updates for a specific contract
    pub fn get_storage_history(
        &self,
        contract_address: &str,
        storage_key: Option<&str>,
    ) -> Result<Vec<StorageUpdate>, IndexerError> {
        let mut sql = String::from("SELECT id, block_number, contract_address, storage_key, storage_value FROM storage_updates WHERE contract_address = ?");
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        params_vec.push(Box::new(contract_address.to_string()));

        if let Some(key) = storage_key {
            sql.push_str(" AND storage_key = ?");
            params_vec.push(Box::new(key.to_string()));
        }

        sql.push_str(" ORDER BY block_number ASC");

        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(StorageUpdate {
                id: row.get(0)?,
                block_number: row.get(1)?,
                contract_address: row.get(2)?,
                storage_key: row.get(3)?,
                storage_value: row.get(4)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Get block by number
    pub fn get_block(&self, block_number: u64) -> Result<Option<IndexedBlock>, IndexerError> {
        let result = self.conn.query_row(
            "SELECT block_number, block_hash, parent_hash, state_root, sequencer_address, timestamp, transaction_count, event_count, l1_gas_price, l1_data_gas_price FROM blocks WHERE block_number = ?",
            params![block_number],
            |row| {
                Ok(IndexedBlock {
                    block_number: row.get(0)?,
                    block_hash: row.get(1)?,
                    parent_hash: row.get(2)?,
                    state_root: row.get(3)?,
                    sequencer_address: row.get(4)?,
                    timestamp: row.get(5)?,
                    transaction_count: row.get(6)?,
                    event_count: row.get(7)?,
                    l1_gas_price: row.get(8)?,
                    l1_data_gas_price: row.get(9)?,
                })
            },
        );

        match result {
            Ok(block) => Ok(Some(block)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(IndexerError::Sqlite(e)),
        }
    }

    /// Get transaction by hash
    pub fn get_transaction(&self, tx_hash: &str) -> Result<Option<IndexedTransaction>, IndexerError> {
        let result = self.conn.query_row(
            "SELECT tx_hash, block_number, tx_index, tx_type, version, status, revert_reason, sender_address, nonce, actual_fee, fee_unit, max_fee, calldata_length, signature_length FROM transactions WHERE tx_hash = ?",
            params![tx_hash],
            |row| {
                Ok(IndexedTransaction {
                    tx_hash: row.get(0)?,
                    block_number: row.get(1)?,
                    tx_index: row.get(2)?,
                    tx_type: row.get(3)?,
                    version: row.get(4)?,
                    status: row.get(5)?,
                    revert_reason: row.get(6)?,
                    sender_address: row.get(7)?,
                    nonce: row.get(8)?,
                    actual_fee: row.get(9)?,
                    fee_unit: row.get(10)?,
                    max_fee: row.get(11)?,
                    calldata_length: row.get(12)?,
                    signature_length: row.get(13)?,
                })
            },
        );

        match result {
            Ok(tx) => Ok(Some(tx)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(IndexerError::Sqlite(e)),
        }
    }

    /// Count transactions by type
    pub fn count_transactions_by_type(&self) -> Result<Vec<(String, i64)>, IndexerError> {
        let mut stmt = self.conn.prepare("SELECT tx_type, COUNT(*) FROM transactions GROUP BY tx_type ORDER BY COUNT(*) DESC")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Count transactions by status
    pub fn count_transactions_by_status(&self) -> Result<Vec<(String, i64)>, IndexerError> {
        let mut stmt = self.conn.prepare("SELECT status, COUNT(*) FROM transactions GROUP BY status ORDER BY COUNT(*) DESC")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Get top event emitters
    pub fn get_top_event_emitters(&self, limit: usize) -> Result<Vec<(String, i64)>, IndexerError> {
        let mut stmt = self.conn.prepare("SELECT from_address, COUNT(*) as count FROM events GROUP BY from_address ORDER BY count DESC LIMIT ?")?;
        let rows = stmt.query_map([limit as i64], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Get top contracts by storage updates
    pub fn get_top_contracts_by_storage(&self, limit: usize) -> Result<Vec<(String, i64)>, IndexerError> {
        let mut stmt = self.conn.prepare("SELECT contract_address, COUNT(*) as count FROM storage_updates GROUP BY contract_address ORDER BY count DESC LIMIT ?")?;
        let rows = stmt.query_map([limit as i64], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }
}
