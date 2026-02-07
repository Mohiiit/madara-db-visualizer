//! Contract and class reading functionality

use crate::blocks::Felt;
use crate::DbReader;
use rocksdb::IteratorMode;
use serde::Deserialize;
use serde_bytes::ByteBuf;

/// Contract information
#[derive(Debug, Clone)]
pub struct ContractInfo {
    pub address: String,
    pub class_hash: Option<String>,
    pub nonce: Option<u64>,
}

/// Storage entry
#[derive(Debug, Clone)]
pub struct StorageEntry {
    pub key: String,
    pub value: String,
}

/// Class type enum
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassType {
    Legacy,
    Sierra,
    Unknown,
}

impl std::fmt::Display for ClassType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClassType::Legacy => write!(f, "LEGACY"),
            ClassType::Sierra => write!(f, "SIERRA"),
            ClassType::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

/// Class information
#[derive(Debug, Clone)]
pub struct ClassInfo {
    pub class_hash: String,
    pub class_type: ClassType,
    pub compiled_class_hash: Option<String>,
}

// Raw deserialization types for class_info
// ClassInfo in madara has the structure with class type enum

#[derive(Debug, Clone, Deserialize)]
struct RawClassInfo {
    // The format is: enum variant (CompiledClass) + fields
    // CompiledClass { class: ContractClass, compiled_class_hash: Option<Felt> }
    pub class: RawContractClass,
    pub compiled_class_hash: Option<ByteBuf>,
}

#[derive(Debug, Clone, Deserialize)]
enum RawContractClass {
    Sierra(RawFlattenedSierraClass),
    Legacy(RawCompressedLegacyContractClass),
}

#[derive(Debug, Clone, Deserialize)]
struct RawFlattenedSierraClass {
    pub sierra_program: Vec<ByteBuf>,
    pub contract_class_version: String,
    pub entry_points_by_type: RawEntryPointsByType,
    pub abi: String,
}

#[derive(Debug, Clone, Deserialize)]
struct RawEntryPointsByType {
    pub constructor: Vec<RawSierraEntryPoint>,
    pub external: Vec<RawSierraEntryPoint>,
    pub l1_handler: Vec<RawSierraEntryPoint>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawSierraEntryPoint {
    pub selector: ByteBuf,
    pub function_idx: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct RawCompressedLegacyContractClass {
    // Legacy class is compressed, we don't need to fully parse it
    pub program: Vec<u8>,
    pub entry_points_by_type: RawLegacyEntryPointsByType,
    pub abi: Option<Vec<RawAbiEntry>>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawLegacyEntryPointsByType {
    pub constructor: Vec<RawLegacyEntryPoint>,
    pub external: Vec<RawLegacyEntryPoint>,
    pub l1_handler: Vec<RawLegacyEntryPoint>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawLegacyEntryPoint {
    pub offset: u64,
    pub selector: ByteBuf,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum RawAbiEntry {
    Function(RawAbiFunctionEntry),
    Event(RawAbiEventEntry),
    Struct(RawAbiStructEntry),
}

#[derive(Debug, Clone, Deserialize)]
struct RawAbiFunctionEntry {
    pub r#type: String,
    pub name: String,
    pub inputs: Vec<RawAbiInput>,
    pub outputs: Vec<RawAbiOutput>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawAbiEventEntry {
    pub r#type: String,
    pub name: String,
    pub data: Vec<RawAbiInput>,
    pub keys: Vec<RawAbiInput>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawAbiStructEntry {
    pub r#type: String,
    pub name: String,
    pub size: u64,
    pub members: Vec<RawAbiMember>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawAbiInput {
    pub name: String,
    pub r#type: String,
}

#[derive(Debug, Clone, Deserialize)]
struct RawAbiOutput {
    pub r#type: String,
}

#[derive(Debug, Clone, Deserialize)]
struct RawAbiMember {
    pub name: String,
    pub r#type: String,
    pub offset: u64,
}

impl DbReader {
    /// Get contract information by address
    pub fn get_contract(&self, address: &str) -> Option<ContractInfo> {
        // Parse address to bytes
        let address_str = address.strip_prefix("0x").unwrap_or(address);
        let address_bytes = hex::decode(address_str).ok()?;

        // Pad to 32 bytes
        let mut key = [0u8; 32];
        let len = address_bytes.len().min(32);
        key[32 - len..].copy_from_slice(&address_bytes[..len]);

        // Get class hash
        let class_hash = self.get_contract_class_hash(&key);

        // Get nonce
        let nonce = self.get_contract_nonce(&key);

        Some(ContractInfo {
            address: format!("0x{}", hex::encode(&key)),
            class_hash,
            nonce,
        })
    }

    /// Get contract class hash
    fn get_contract_class_hash(&self, address: &[u8; 32]) -> Option<String> {
        use bincode::Options;

        let cf = self.db.cf_handle("contract_class_hashes")?;
        let value = self.db.get_cf(&cf, address).ok()??;

        // Value is a ByteBuf (length-prefixed bytes)
        let opts = bincode::DefaultOptions::new();
        let hash: ByteBuf = opts.deserialize(&value).ok()?;

        Some(Felt::from_bytes(&hash).to_hex())
    }

    /// Get contract nonce
    fn get_contract_nonce(&self, address: &[u8; 32]) -> Option<u64> {
        use bincode::Options;

        let cf = self.db.cf_handle("contract_nonces")?;
        let value = self.db.get_cf(&cf, address).ok()??;

        // Value is a u64 with varint encoding
        let opts = bincode::DefaultOptions::new();
        let nonce: u64 = opts.deserialize(&value).ok()?;

        Some(nonce)
    }

    /// Get contract storage entries
    pub fn get_contract_storage(&self, address: &str, limit: usize) -> Vec<StorageEntry> {
        // Parse address to bytes
        let address_str = address.strip_prefix("0x").unwrap_or(address);
        let address_bytes = match hex::decode(address_str) {
            Ok(b) => b,
            Err(_) => return vec![],
        };

        // Pad to 32 bytes
        let mut prefix = [0u8; 32];
        let len = address_bytes.len().min(32);
        prefix[32 - len..].copy_from_slice(&address_bytes[..len]);

        let cf = match self.db.cf_handle("contract_storage") {
            Some(cf) => cf,
            None => return vec![],
        };

        let mut entries = Vec::new();

        // Iterate with prefix
        let iter = self.db.iterator_cf(
            &cf,
            IteratorMode::From(&prefix, rocksdb::Direction::Forward),
        );

        for item in iter {
            if let Ok((key, value)) = item {
                // Key format: address (32 bytes) + storage_key (32 bytes)
                if key.len() < 64 {
                    continue;
                }

                // Check if still same address
                if &key[..32] != &prefix[..] {
                    break;
                }

                let storage_key = &key[32..64];

                // Value is a ByteBuf (Felt)
                use bincode::Options;
                let opts = bincode::DefaultOptions::new();
                if let Ok(val_bytes) = opts.deserialize::<ByteBuf>(&value) {
                    entries.push(StorageEntry {
                        key: format!("0x{}", hex::encode(storage_key)),
                        value: Felt::from_bytes(&val_bytes).to_hex(),
                    });
                }

                if entries.len() >= limit {
                    break;
                }
            }
        }

        entries
    }

    /// Get class information by hash
    pub fn get_class(&self, class_hash: &str) -> Option<ClassInfo> {
        use bincode::Options;

        // Parse class hash to bytes
        let hash_str = class_hash.strip_prefix("0x").unwrap_or(class_hash);
        let hash_bytes = hex::decode(hash_str).ok()?;

        // Pad to 32 bytes
        let mut key = [0u8; 32];
        let len = hash_bytes.len().min(32);
        key[32 - len..].copy_from_slice(&hash_bytes[..len]);

        let cf = self.db.cf_handle("class_info")?;
        let value = self.db.get_cf(&cf, key).ok()??;

        let opts = bincode::DefaultOptions::new();

        // Try to deserialize - but this might fail due to complex structure
        // Let's just detect the class type from the first byte (enum variant)
        if value.is_empty() {
            return None;
        }

        let class_type = match value[0] {
            0 => ClassType::Sierra,
            1 => ClassType::Legacy,
            _ => ClassType::Unknown,
        };

        // For compiled class hash, we need to try parsing
        // For now, return basic info
        Some(ClassInfo {
            class_hash: format!("0x{}", hex::encode(&key)),
            class_type,
            compiled_class_hash: None, // Would need full deserialization
        })
    }

    /// List contracts (first N contracts from contract_class_hashes)
    pub fn list_contracts(&self, limit: usize) -> Vec<ContractInfo> {
        let cf = match self.db.cf_handle("contract_class_hashes") {
            Some(cf) => cf,
            None => return vec![],
        };

        let mut contracts = Vec::new();
        let iter = self.db.iterator_cf(&cf, IteratorMode::Start);

        for item in iter {
            if let Ok((key, value)) = item {
                // Keys could be 32 bytes or have different lengths
                // Try to extract address and class hash directly
                if key.len() >= 32 {
                    // Use first 32 bytes as address
                    let address = format!("0x{}", hex::encode(&key[..32]));

                    // Try to parse value as class hash
                    use bincode::Options;
                    let opts = bincode::DefaultOptions::new();
                    let class_hash = opts
                        .deserialize::<serde_bytes::ByteBuf>(&value)
                        .ok()
                        .map(|h| Felt::from_bytes(&h).to_hex());

                    // Get nonce
                    let mut addr_bytes = [0u8; 32];
                    addr_bytes.copy_from_slice(&key[..32]);
                    let nonce = self.get_contract_nonce(&addr_bytes);

                    contracts.push(ContractInfo {
                        address,
                        class_hash,
                        nonce,
                    });
                } else if !key.is_empty() {
                    // Shorter key - still try to use it
                    let address = format!("0x{}", hex::encode(&key));

                    use bincode::Options;
                    let opts = bincode::DefaultOptions::new();
                    let class_hash = opts
                        .deserialize::<serde_bytes::ByteBuf>(&value)
                        .ok()
                        .map(|h| Felt::from_bytes(&h).to_hex());

                    contracts.push(ContractInfo {
                        address,
                        class_hash,
                        nonce: None,
                    });
                }

                if contracts.len() >= limit {
                    break;
                }
            }
        }

        contracts
    }

    /// List classes (first N classes from class_info)
    pub fn list_classes(&self, limit: usize) -> Vec<ClassInfo> {
        let cf = match self.db.cf_handle("class_info") {
            Some(cf) => cf,
            None => return vec![],
        };

        let mut classes = Vec::new();
        let iter = self.db.iterator_cf(&cf, IteratorMode::Start);

        for item in iter {
            if let Ok((key, value)) = item {
                if key.len() == 32 {
                    let class_hash = format!("0x{}", hex::encode(&key));

                    let class_type = if value.is_empty() {
                        ClassType::Unknown
                    } else {
                        match value[0] {
                            0 => ClassType::Sierra,
                            1 => ClassType::Legacy,
                            _ => ClassType::Unknown,
                        }
                    };

                    classes.push(ClassInfo {
                        class_hash,
                        class_type,
                        compiled_class_hash: None,
                    });
                }

                if classes.len() >= limit {
                    break;
                }
            }
        }

        classes
    }
}
