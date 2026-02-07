//! Query functions for reading data from the database

use crate::{DbError, DbReader};
use rocksdb::IteratorMode;

/// Database statistics
#[derive(Debug, Clone)]
pub struct DbStats {
    pub db_path: String,
    pub column_count: usize,
    pub columns: Vec<String>,
    pub latest_block: Option<u64>,
}

impl DbReader {
    /// Get basic database statistics
    pub fn get_stats(&self) -> DbStats {
        let columns = self.column_families();
        let column_count = columns.len();

        // Try to get latest block from chain tip
        let latest_block = self.get_latest_block_number();

        DbStats {
            db_path: self.path().display().to_string(),
            column_count,
            columns,
            latest_block,
        }
    }

    /// Get the latest confirmed block number from the database
    pub fn get_latest_block_number(&self) -> Option<u64> {
        // Try chain tip first (bincode varint encoding)
        if let Some(block_n) = self.get_chain_tip_block() {
            return Some(block_n);
        }

        // Fallback: find highest block number from block_info column
        self.get_highest_block_from_block_info()
    }

    /// Parse chain tip from meta column
    /// The format is bincode DefaultOptions which uses varint encoding:
    /// - variant 0 (Confirmed): 1 byte + varint u64
    fn get_chain_tip_block(&self) -> Option<u64> {
        let cf = self.db.cf_handle("meta")?;
        let value = self.db.get_cf(&cf, b"CHAIN_TIP").ok()??;

        // First byte is variant index (0 = Confirmed, 1 = Preconfirmed)
        if value.is_empty() || value[0] != 0 {
            return None;
        }

        // Rest is varint-encoded u64 block number
        // For small numbers (< 251), it's just one byte
        // For larger numbers, it uses multi-byte encoding
        if value.len() == 2 {
            // Single byte block number
            return Some(value[1] as u64);
        } else if value.len() >= 2 {
            // Try to decode varint
            // Bincode uses a custom varint format:
            // 0-250: single byte
            // 251: 2-byte LE
            // 252: 4-byte LE
            // 253: 8-byte LE
            let first = value[1];
            if first <= 250 {
                return Some(first as u64);
            } else if first == 251 && value.len() >= 4 {
                return Some(u16::from_le_bytes([value[2], value[3]]) as u64);
            } else if first == 252 && value.len() >= 6 {
                return Some(u32::from_le_bytes([value[2], value[3], value[4], value[5]]) as u64);
            } else if first == 253 && value.len() >= 10 {
                return Some(u64::from_le_bytes([
                    value[2], value[3], value[4], value[5], value[6], value[7], value[8], value[9],
                ]));
            }
        }

        None
    }

    /// Fallback method to find the highest block number by scanning block_info column
    fn get_highest_block_from_block_info(&self) -> Option<u64> {
        let cf = self.db.cf_handle("block_info")?;

        // Iterate in reverse to get the highest key
        // Keys can be 4 bytes (u32) or 8 bytes (u64) depending on version
        let iter = self.db.iterator_cf(&cf, IteratorMode::End);

        for item in iter {
            if let Ok((key, _)) = item {
                if key.len() == 4 {
                    // 4-byte big-endian block number
                    let block_n = u32::from_be_bytes([key[0], key[1], key[2], key[3]]);
                    return Some(block_n as u64);
                } else if key.len() == 8 {
                    // 8-byte big-endian block number
                    let block_n = u64::from_be_bytes([
                        key[0], key[1], key[2], key[3], key[4], key[5], key[6], key[7],
                    ]);
                    return Some(block_n);
                }
            }
            break; // Only need the first (highest) key
        }
        None
    }

    /// Get the number of entries in a column family
    pub fn get_column_count(&self, column_name: &str) -> Result<u64, DbError> {
        let cf = self.db.cf_handle(column_name).ok_or_else(|| {
            DbError::Deserialize(format!("Column family not found: {column_name}"))
        })?;

        // Use RocksDB property to get approximate number of keys
        let prop = self
            .db
            .property_int_value_cf(&cf, "rocksdb.estimate-num-keys")
            .ok()
            .flatten()
            .unwrap_or(0);

        Ok(prop)
    }

    /// Search for blocks, transactions, contracts, or classes
    /// Returns a SearchResult indicating what was found
    pub fn search(&self, query: &str) -> SearchResult {
        let query = query.trim();

        // Try to parse as block number first
        if let Ok(block_n) = query.parse::<u64>() {
            if self.get_block_detail(block_n).is_some() {
                return SearchResult::Block(block_n);
            }
        }

        // Try as hex (transaction hash, contract address, or class hash)
        let hex_query = query.strip_prefix("0x").unwrap_or(query);
        if hex::decode(hex_query).is_ok() {
            // Try as transaction hash
            if let Some((block_n, tx_index)) = self.find_transaction_by_hash(query) {
                return SearchResult::Transaction { block_n, tx_index };
            }

            // Try as contract address
            if let Some(contract) = self.get_contract(query) {
                return SearchResult::Contract(contract.address);
            }

            // Try as class hash
            if let Some(class) = self.get_class(query) {
                return SearchResult::Class(class.class_hash);
            }
        }

        SearchResult::NotFound
    }
}

/// Search result types
#[derive(Debug, Clone)]
pub enum SearchResult {
    Block(u64),
    Transaction { block_n: u64, tx_index: u64 },
    Contract(String),
    Class(String),
    NotFound,
}
