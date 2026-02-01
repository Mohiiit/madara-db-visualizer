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

    /// Fetch raw value bytes for a specific key in a column family
    pub fn get_raw_value(&self, cf_name: &str, key: &[u8]) -> Option<Vec<u8>> {
        let cf = self.db.cf_handle(cf_name)?;
        self.db.get_cf(&cf, key).ok().flatten()
    }

    /// Batch fetch multiple key-value pairs from a column family
    pub fn get_key_value_pairs(&self, cf_name: &str, keys: &[Vec<u8>]) -> Vec<(Vec<u8>, Vec<u8>)> {
        let cf = match self.db.cf_handle(cf_name) {
            Some(cf) => cf,
            None => return vec![],
        };

        keys.iter()
            .filter_map(|key| {
                self.db
                    .get_cf(&cf, key)
                    .ok()
                    .flatten()
                    .map(|value| (key.clone(), value))
            })
            .collect()
    }

    /// Attempt to decode a value based on known column family schemas
    /// Returns a human-readable hint about what the value represents
    pub fn decode_value_hint(&self, cf_name: &str, key: &[u8], value: &[u8]) -> Option<String> {
        match cf_name {
            // Block-related column families
            "block_hash" => {
                // Key is likely block number (u64 big-endian), value is block hash
                if key.len() == 8 {
                    let block_num = u64::from_be_bytes(key.try_into().ok()?);
                    Some(format!("block_number: {}", block_num))
                } else {
                    None
                }
            }
            "block_n" | "block_number" => {
                // Key might be block hash, value might be block number
                if value.len() == 8 {
                    let block_num = u64::from_be_bytes(value.try_into().ok()?);
                    Some(format!("block_number: {}", block_num))
                } else {
                    None
                }
            }
            "block_statuses" => {
                // Try to interpret status byte
                if value.len() >= 1 {
                    let status = match value[0] {
                        0 => "pending",
                        1 => "accepted_on_l2",
                        2 => "accepted_on_l1",
                        3 => "rejected",
                        _ => "unknown",
                    };
                    Some(format!("status: {}", status))
                } else {
                    None
                }
            }
            // Transaction-related
            "tx_hash" | "tx_hashes" => {
                if key.len() >= 8 {
                    // First 8 bytes might be block number
                    let block_num = u64::from_be_bytes(key[..8].try_into().ok()?);
                    if key.len() > 8 {
                        let tx_idx = if key.len() >= 16 {
                            u64::from_be_bytes(key[8..16].try_into().ok()?)
                        } else {
                            0
                        };
                        Some(format!("block: {}, tx_index: {}", block_num, tx_idx))
                    } else {
                        Some(format!("block: {}", block_num))
                    }
                } else {
                    None
                }
            }
            // Contract-related
            "contract_class_hash" | "contract_class_hashes" => {
                Some("contract_address -> class_hash mapping".to_string())
            }
            "contract_nonces" => {
                if value.len() >= 8 {
                    // Try to decode as u64 nonce
                    if let Ok(bytes) = value[..8].try_into() {
                        let nonce = u64::from_be_bytes(bytes);
                        return Some(format!("nonce: {}", nonce));
                    }
                }
                None
            }
            // Storage
            "contract_storage" => {
                Some("contract storage key-value pair".to_string())
            }
            // Class-related
            "class_info" | "sierra_classes" | "compiled_classes" => {
                Some(format!("class data, size: {} bytes", value.len()))
            }
            // State diff
            "state_diff" => {
                if key.len() == 8 {
                    let block_num = u64::from_be_bytes(key.try_into().ok()?);
                    Some(format!("state_diff for block: {}", block_num))
                } else {
                    None
                }
            }
            // Trie-related
            name if name.contains("trie") || name.contains("bonsai") => {
                Some(format!("trie node, size: {} bytes", value.len()))
            }
            // Default: try to provide size info
            _ => {
                if value.len() > 100 {
                    Some(format!("large value: {} bytes", value.len()))
                } else {
                    None
                }
            }
        }
    }
}
