//! SQLite indexer for complex queries on Madara DB

use db_reader::DbReader;
use rusqlite::{params, Connection, Result as SqliteResult};
use std::path::Path;
use thiserror::Error;

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
}

/// Transaction record for queries
#[derive(Debug, Clone)]
pub struct IndexedTransaction {
    pub tx_hash: String,
    pub block_number: u64,
    pub tx_index: u64,
    pub tx_type: String,
    pub status: String,
    pub revert_reason: Option<String>,
    pub sender_address: Option<String>,
}

/// Contract record for queries
#[derive(Debug, Clone)]
pub struct IndexedContract {
    pub address: String,
    pub class_hash: Option<String>,
    pub nonce: Option<u64>,
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

    /// Initialize the database schema
    fn init_schema(&self) -> Result<(), IndexerError> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS transactions (
                tx_hash TEXT PRIMARY KEY,
                block_number INTEGER NOT NULL,
                tx_index INTEGER NOT NULL,
                tx_type TEXT NOT NULL,
                status TEXT NOT NULL,
                revert_reason TEXT,
                sender_address TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_tx_block ON transactions(block_number);
            CREATE INDEX IF NOT EXISTS idx_tx_status ON transactions(status);
            CREATE INDEX IF NOT EXISTS idx_tx_sender ON transactions(sender_address);
            CREATE INDEX IF NOT EXISTS idx_tx_type ON transactions(tx_type);

            CREATE TABLE IF NOT EXISTS contracts (
                address TEXT PRIMARY KEY,
                class_hash TEXT,
                nonce INTEGER
            );

            CREATE INDEX IF NOT EXISTS idx_contract_class ON contracts(class_hash);

            CREATE TABLE IF NOT EXISTS index_status (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                indexed_blocks INTEGER NOT NULL DEFAULT 0,
                latest_block INTEGER NOT NULL DEFAULT 0
            );

            INSERT OR IGNORE INTO index_status (id, indexed_blocks, latest_block) VALUES (1, 0, 0);
            "#,
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

        Ok(IndexStatus {
            indexed_blocks,
            latest_block,
            is_synced: indexed_blocks > 0 && indexed_blocks >= latest_block,
            total_transactions,
            failed_transactions,
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
            // Index transactions from this block
            let transactions = db.get_block_transactions(block_n);

            for tx_info in transactions {
                let status = match &tx_info.status {
                    db_reader::ExecutionStatus::Succeeded => "SUCCEEDED",
                    db_reader::ExecutionStatus::Reverted(_) => "REVERTED",
                };
                let revert_reason = match &tx_info.status {
                    db_reader::ExecutionStatus::Reverted(reason) => Some(reason.clone()),
                    _ => None,
                };

                // Get full transaction detail for sender address
                let sender = db.get_transaction_detail(block_n, tx_info.tx_index as u64)
                    .and_then(|t| t.sender_address);

                tx.execute(
                    "INSERT OR REPLACE INTO transactions (tx_hash, block_number, tx_index, tx_type, status, revert_reason, sender_address) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        tx_info.tx_hash,
                        block_n,
                        tx_info.tx_index,
                        tx_info.tx_type.to_string(),
                        status,
                        revert_reason,
                        sender,
                    ],
                )?;
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
        let mut sql = String::from("SELECT tx_hash, block_number, tx_index, tx_type, status, revert_reason, sender_address FROM transactions WHERE 1=1");
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
                status: row.get(4)?,
                revert_reason: row.get(5)?,
                sender_address: row.get(6)?,
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
}
