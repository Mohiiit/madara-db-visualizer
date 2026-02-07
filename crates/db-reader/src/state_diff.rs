//! State diff reading functionality

use crate::blocks::Felt;
use crate::DbReader;
use serde::Deserialize;
use serde_bytes::ByteBuf;

/// State diff for a block
#[derive(Debug, Clone, Default)]
pub struct StateDiffInfo {
    /// Deployed contracts
    pub deployed_contracts: Vec<DeployedContract>,
    /// Storage changes
    pub storage_diffs: Vec<ContractStorageDiff>,
    /// Declared classes
    pub declared_classes: Vec<DeclaredClass>,
    /// Nonce updates
    pub nonces: Vec<NonceUpdateInfo>,
    /// Replaced classes
    pub replaced_classes: Vec<ReplacedClass>,
}

/// Deployed contract info
#[derive(Debug, Clone)]
pub struct DeployedContract {
    pub address: String,
    pub class_hash: String,
}

/// Storage diff for a contract
#[derive(Debug, Clone)]
pub struct ContractStorageDiff {
    pub address: String,
    pub storage_entries: Vec<StorageDiffEntry>,
}

/// Single storage diff entry
#[derive(Debug, Clone)]
pub struct StorageDiffEntry {
    pub key: String,
    pub value: String,
}

/// Declared class info
#[derive(Debug, Clone)]
pub struct DeclaredClass {
    pub class_hash: String,
    pub compiled_class_hash: String,
}

/// Nonce update info
#[derive(Debug, Clone)]
pub struct NonceUpdateInfo {
    pub contract_address: String,
    pub nonce: String,
}

/// Replaced class info
#[derive(Debug, Clone)]
pub struct ReplacedClass {
    pub contract_address: String,
    pub class_hash: String,
}

// Raw deserialization types matching Madara's StateDiff

#[derive(Debug, Clone, Deserialize)]
struct RawStateDiff {
    pub storage_diffs: Vec<RawContractStorageDiffItem>,
    #[serde(default)]
    pub old_declared_contracts: Vec<ByteBuf>,
    pub declared_classes: Vec<RawDeclaredClassItem>,
    pub deployed_contracts: Vec<RawDeployedContractItem>,
    pub replaced_classes: Vec<RawReplacedClassItem>,
    pub nonces: Vec<RawNonceUpdate>,
    #[serde(default)]
    pub migrated_compiled_classes: Vec<RawMigratedClassItem>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawContractStorageDiffItem {
    pub address: ByteBuf,
    pub storage_entries: Vec<RawStorageEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawStorageEntry {
    pub key: ByteBuf,
    pub value: ByteBuf,
}

#[derive(Debug, Clone, Deserialize)]
struct RawDeclaredClassItem {
    pub class_hash: ByteBuf,
    pub compiled_class_hash: ByteBuf,
}

#[derive(Debug, Clone, Deserialize)]
struct RawDeployedContractItem {
    pub address: ByteBuf,
    pub class_hash: ByteBuf,
}

#[derive(Debug, Clone, Deserialize)]
struct RawReplacedClassItem {
    pub contract_address: ByteBuf,
    pub class_hash: ByteBuf,
}

#[derive(Debug, Clone, Deserialize)]
struct RawNonceUpdate {
    pub contract_address: ByteBuf,
    pub nonce: ByteBuf,
}

#[derive(Debug, Clone, Deserialize)]
struct RawMigratedClassItem {
    pub class_hash: ByteBuf,
    pub compiled_class_hash: ByteBuf,
}

impl DbReader {
    /// Get state diff for a block
    pub fn get_state_diff(&self, block_n: u64) -> Option<StateDiffInfo> {
        use bincode::Options;

        let block_n_u32 = u32::try_from(block_n).ok()?;
        let cf = self.db.cf_handle("block_state_diff")?;
        let value = self.db.get_cf(&cf, block_n_u32.to_be_bytes()).ok()??;

        let opts = bincode::DefaultOptions::new();
        let raw: RawStateDiff = match opts.deserialize(&value) {
            Ok(r) => r,
            Err(e) => {
                eprintln!(
                    "State diff deserialization error for block {}: {}",
                    block_n, e
                );
                eprintln!(
                    "Raw value length: {}, first 20 bytes: {:?}",
                    value.len(),
                    &value[..20.min(value.len())]
                );
                return None;
            }
        };

        Some(StateDiffInfo {
            deployed_contracts: raw
                .deployed_contracts
                .iter()
                .map(|d| DeployedContract {
                    address: Felt::from_bytes(&d.address).to_hex(),
                    class_hash: Felt::from_bytes(&d.class_hash).to_hex(),
                })
                .collect(),
            storage_diffs: raw
                .storage_diffs
                .iter()
                .map(|s| ContractStorageDiff {
                    address: Felt::from_bytes(&s.address).to_hex(),
                    storage_entries: s
                        .storage_entries
                        .iter()
                        .map(|e| StorageDiffEntry {
                            key: Felt::from_bytes(&e.key).to_hex(),
                            value: Felt::from_bytes(&e.value).to_hex(),
                        })
                        .collect(),
                })
                .collect(),
            declared_classes: raw
                .declared_classes
                .iter()
                .map(|d| DeclaredClass {
                    class_hash: Felt::from_bytes(&d.class_hash).to_hex(),
                    compiled_class_hash: Felt::from_bytes(&d.compiled_class_hash).to_hex(),
                })
                .collect(),
            nonces: raw
                .nonces
                .iter()
                .map(|n| NonceUpdateInfo {
                    contract_address: Felt::from_bytes(&n.contract_address).to_hex(),
                    nonce: Felt::from_bytes(&n.nonce).to_hex(),
                })
                .collect(),
            replaced_classes: raw
                .replaced_classes
                .iter()
                .map(|r| ReplacedClass {
                    contract_address: Felt::from_bytes(&r.contract_address).to_hex(),
                    class_hash: Felt::from_bytes(&r.class_hash).to_hex(),
                })
                .collect(),
        })
    }
}
