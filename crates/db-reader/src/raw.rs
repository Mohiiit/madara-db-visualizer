//! Raw column family browsing functionality

use crate::DbReader;
use rocksdb::IteratorMode;

/// Statistics for a column family
#[derive(Debug, Clone)]
pub struct CfStats {
    pub name: String,
    pub key_count: usize,
    pub first_key_hex: Option<String>,
    pub last_key_hex: Option<String>,
}

impl DbReader {
    /// List all column family names in the database
    pub fn list_column_families(&self) -> Vec<String> {
        self.column_families()
    }

    /// Get statistics for a specific column family
    pub fn get_cf_stats(&self, cf_name: &str) -> Option<CfStats> {
        let cf = self.db.cf_handle(cf_name)?;

        // Count keys by iterating (expensive but accurate)
        let key_count = self.count_keys(cf_name);

        // Get first key
        let first_key_hex = {
            let mut iter = self.db.iterator_cf(&cf, IteratorMode::Start);
            iter.next()
                .and_then(|result| result.ok())
                .map(|(key, _)| format!("0x{}", hex::encode(&key)))
        };

        // Get last key
        let last_key_hex = {
            let mut iter = self.db.iterator_cf(&cf, IteratorMode::End);
            iter.next()
                .and_then(|result| result.ok())
                .map(|(key, _)| format!("0x{}", hex::encode(&key)))
        };

        Some(CfStats {
            name: cf_name.to_string(),
            key_count,
            first_key_hex,
            last_key_hex,
        })
    }

    /// List keys in a column family with pagination and optional prefix filtering
    pub fn list_keys(
        &self,
        cf_name: &str,
        limit: usize,
        offset: usize,
        prefix: Option<&[u8]>,
    ) -> Vec<Vec<u8>> {
        let cf = match self.db.cf_handle(cf_name) {
            Some(cf) => cf,
            None => return vec![],
        };

        let mut keys = Vec::with_capacity(limit);
        let mut skipped = 0;
        let mut collected = 0;

        let iter = match prefix {
            Some(prefix_bytes) => {
                // Use prefix iterator if prefix is provided
                self.db.prefix_iterator_cf(&cf, prefix_bytes)
            }
            None => {
                // Start from beginning
                self.db.iterator_cf(&cf, IteratorMode::Start)
            }
        };

        for item in iter {
            match item {
                Ok((key, _)) => {
                    // If prefix is set, verify the key still matches the prefix
                    if let Some(prefix_bytes) = prefix {
                        if !key.starts_with(prefix_bytes) {
                            break;
                        }
                    }

                    // Skip offset entries
                    if skipped < offset {
                        skipped += 1;
                        continue;
                    }

                    // Collect up to limit entries
                    if collected < limit {
                        keys.push(key.to_vec());
                        collected += 1;
                    } else {
                        break;
                    }
                }
                Err(_) => break,
            }
        }

        keys
    }

    /// Count the total number of keys in a column family
    /// Note: This can be expensive for large column families
    pub fn count_keys(&self, cf_name: &str) -> usize {
        let cf = match self.db.cf_handle(cf_name) {
            Some(cf) => cf,
            None => return 0,
        };

        // First try to use RocksDB's estimate (fast but approximate)
        if let Ok(Some(estimate)) = self.db.property_int_value_cf(&cf, "rocksdb.estimate-num-keys")
        {
            // If estimate is available and reasonable, use it
            // For accuracy, we could iterate, but that's expensive
            // The estimate is usually within 10-20% of actual
            return estimate as usize;
        }

        // Fallback: iterate and count (expensive but accurate)
        let iter = self.db.iterator_cf(&cf, IteratorMode::Start);
        iter.filter_map(|r| r.ok()).count()
    }

    /// Count keys with a specific prefix in a column family
    pub fn count_keys_with_prefix(&self, cf_name: &str, prefix: &[u8]) -> usize {
        let cf = match self.db.cf_handle(cf_name) {
            Some(cf) => cf,
            None => return 0,
        };

        let iter = self.db.prefix_iterator_cf(&cf, prefix);
        iter.filter_map(|r| r.ok())
            .take_while(|(key, _)| key.starts_with(prefix))
            .count()
    }
}
