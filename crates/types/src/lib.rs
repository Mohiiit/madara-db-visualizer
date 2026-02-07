use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsResponse {
    pub db_path: String,
    pub latest_block: Option<u64>,
    pub column_count: usize,
    pub columns: Vec<String>,
    pub madara_db_version: MadaraDbVersionInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MadaraDbVersionInfo {
    /// The detected Madara DB schema version (from `.db-version`), if present.
    pub version: Option<u32>,
    /// Whether the visualizer considers this DB version supported.
    /// `None` means "unknown" (version could not be detected).
    pub supported: Option<bool>,
    /// Versions supported by this visualizer build.
    pub supported_versions: Vec<u32>,
    /// Best-effort hint of where the version was detected from (e.g. path to `.db-version`).
    pub source: Option<String>,
    /// Best-effort error message if detection failed (e.g. invalid file content).
    pub error: Option<String>,
}

/// Summary of a block for list views
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockSummary {
    pub block_number: u64,
    pub block_hash: String,
    pub parent_hash: String,
    pub timestamp: u64,
    pub transaction_count: u64,
}

/// Full block details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDetail {
    pub block_number: u64,
    pub block_hash: String,
    pub parent_hash: String,
    pub state_root: String,
    pub sequencer_address: String,
    pub timestamp: u64,
    pub transaction_count: u64,
    pub event_count: u64,
    pub l2_gas_used: u128,
    pub tx_hashes: Vec<String>,
}

/// Paginated list of blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockListResponse {
    pub blocks: Vec<BlockSummary>,
    pub total: u64,
    pub offset: u64,
    pub limit: u64,
}

/// Transaction summary for list views
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSummary {
    pub tx_hash: String,
    pub tx_type: String,
    pub status: String,
    pub revert_reason: Option<String>,
    pub block_number: u64,
    pub tx_index: usize,
}

/// Full transaction details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionDetail {
    pub tx_hash: String,
    pub tx_type: String,
    pub status: String,
    pub revert_reason: Option<String>,
    pub block_number: u64,
    pub tx_index: usize,
    pub actual_fee: String,
    pub fee_unit: String,
    pub events: Vec<EventInfo>,
    pub messages_sent: Vec<MessageInfo>,
    pub sender_address: Option<String>,
    pub calldata: Vec<String>,
    pub signature: Vec<String>,
    pub nonce: Option<String>,
    pub version: Option<String>,
}

/// Event information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventInfo {
    pub from_address: String,
    pub keys: Vec<String>,
    pub data: Vec<String>,
}

/// Message to L1 information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageInfo {
    pub from_address: String,
    pub to_address: String,
    pub payload: Vec<String>,
}

/// List of transactions for a block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionListResponse {
    pub transactions: Vec<TransactionSummary>,
    pub block_number: u64,
    pub total: usize,
}

/// Contract information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractResponse {
    pub address: String,
    pub class_hash: Option<String>,
    pub nonce: Option<u64>,
}

/// Storage entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageEntryResponse {
    pub key: String,
    pub value: String,
}

/// Contract storage response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractStorageResponse {
    pub address: String,
    pub entries: Vec<StorageEntryResponse>,
    pub total: usize,
}

/// Class information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassResponse {
    pub class_hash: String,
    pub class_type: String,
    pub compiled_class_hash: Option<String>,
}

/// List of contracts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractListResponse {
    pub contracts: Vec<ContractResponse>,
    pub total: usize,
}

/// List of classes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassListResponse {
    pub classes: Vec<ClassResponse>,
    pub total: usize,
}

// State diff types

/// State diff response for a block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDiffResponse {
    pub block_number: u64,
    pub deployed_contracts: Vec<DeployedContractInfo>,
    pub storage_diffs: Vec<ContractStorageDiffInfo>,
    pub declared_classes: Vec<DeclaredClassInfo>,
    pub nonces: Vec<NonceUpdateResponse>,
    pub replaced_classes: Vec<ReplacedClassInfo>,
}

/// Deployed contract info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployedContractInfo {
    pub address: String,
    pub class_hash: String,
}

/// Storage diff for a contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractStorageDiffInfo {
    pub address: String,
    pub storage_entries: Vec<StorageDiffEntryInfo>,
}

/// Single storage diff entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageDiffEntryInfo {
    pub key: String,
    pub value: String,
}

/// Declared class info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeclaredClassInfo {
    pub class_hash: String,
    pub compiled_class_hash: String,
}

/// Nonce update info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonceUpdateResponse {
    pub contract_address: String,
    pub nonce: String,
}

/// Replaced class info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplacedClassInfo {
    pub contract_address: String,
    pub class_hash: String,
}

// Search types

/// Search result response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub result_type: String,
    pub block_number: Option<u64>,
    pub tx_index: Option<u64>,
    pub address: Option<String>,
    pub class_hash: Option<String>,
}

// Index types

/// Index status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStatusResponse {
    pub indexed_blocks: u64,
    pub latest_block: u64,
    pub is_synced: bool,
    pub total_transactions: u64,
    pub failed_transactions: u64,
}

/// Filtered transaction query response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilteredTransactionsResponse {
    pub transactions: Vec<IndexedTransactionInfo>,
    pub total: usize,
}

/// Indexed transaction info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedTransactionInfo {
    pub tx_hash: String,
    pub block_number: u64,
    pub tx_index: u64,
    pub tx_type: String,
    pub status: String,
    pub revert_reason: Option<String>,
    pub sender_address: Option<String>,
}

/// Filtered contracts query response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilteredContractsResponse {
    pub contracts: Vec<ContractResponse>,
    pub total: usize,
}

// Raw column family browsing types

/// Information about a column family
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnFamilyInfo {
    pub name: String,
    pub key_count: usize,
}

/// Response listing all column families
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnFamilyListResponse {
    pub column_families: Vec<ColumnFamilyInfo>,
}

/// Statistics for a single column family
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnFamilyStats {
    pub name: String,
    pub key_count: usize,
    pub first_key_hex: Option<String>,
    pub last_key_hex: Option<String>,
}

/// Information about a single key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyInfo {
    pub raw_hex: String,
}

/// Response listing keys in a column family with pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyListResponse {
    pub cf_name: String,
    pub keys: Vec<KeyInfo>,
    pub total: usize,
    pub offset: usize,
    pub limit: usize,
    pub has_more: bool,
}

/// Raw key-value pair with decoded hints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawKeyValue {
    pub key_hex: String,
    pub value_hex: String,
    pub value_size: usize,
    pub decoded_hint: Option<String>,
}

/// Response for fetching a single raw key-value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawKeyValueResponse {
    pub cf_name: String,
    pub key_value: Option<RawKeyValue>,
    pub found: bool,
}

/// Request body for batch fetching multiple keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchKeysRequest {
    pub keys: Vec<String>,
}

/// Response for batch fetching multiple key-values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchKeyValueResponse {
    pub cf_name: String,
    pub key_values: Vec<RawKeyValue>,
    pub requested_count: usize,
    pub found_count: usize,
}

// SQL Query execution types

/// Request for executing a raw SQL query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryRequest {
    pub sql: String,
    #[serde(default)]
    pub params: Vec<String>,
}

/// Result of a SQL query execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub row_count: usize,
    pub truncated: bool,
}

/// Information about an indexed table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub row_count: usize,
    pub columns: Vec<ColumnInfo>,
}

/// Information about a table column
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
}

/// Response listing all indexed tables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableListResponse {
    pub tables: Vec<TableInfo>,
}

/// Response for table schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSchemaResponse {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
}

// Schema documentation types

/// Schema category with summary information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaCategoryInfo {
    /// Category name (e.g., "blocks", "contracts", "classes")
    pub name: String,
    /// Number of column families in this category
    pub column_family_count: usize,
    /// Brief description of what this category contains
    pub description: String,
}

/// Response listing all schema categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaCategoriesResponse {
    pub categories: Vec<SchemaCategoryInfo>,
    pub total: usize,
}

/// Schema for a field within a value struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaFieldInfo {
    /// Field name
    pub name: String,
    /// Rust type of the field
    pub rust_type: String,
    /// Description of what this field contains
    pub description: String,
}

/// Key encoding documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaKeyInfo {
    /// Rust type used for the key (e.g., "u64", "Felt")
    pub rust_type: String,
    /// Encoding format (e.g., "big-endian", "raw", "bincode")
    pub encoding: String,
    /// Size in bytes (if fixed-size)
    pub size_bytes: Option<usize>,
    /// Detailed description of key format
    pub description: String,
    /// Example in raw hex format
    pub example_raw: String,
    /// Example in decoded/human-readable form
    pub example_decoded: String,
}

/// Value serialization documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaValueInfo {
    /// Rust type for the value
    pub rust_type: String,
    /// Encoding format (e.g., "bincode", "raw", "json")
    pub encoding: String,
    /// Description of value contents
    pub description: String,
    /// Fields within the value (for struct types)
    pub fields: Vec<SchemaFieldInfo>,
}

/// Relationship to another column family
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaRelationshipInfo {
    /// Target column family name
    pub target_cf: String,
    /// Relationship type (e.g., "inverse", "references", "contains", "indexed_by")
    pub relationship_type: String,
    /// Description of the relationship
    pub description: String,
}

/// Complete schema information for a column family
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnFamilySchemaInfo {
    /// Column family name (e.g., "block_info", "contract_storage")
    pub name: String,
    /// Category (e.g., "blocks", "contracts", "classes")
    pub category: String,
    /// Purpose and description of this column family
    pub purpose: String,
    /// Key encoding documentation
    pub key: SchemaKeyInfo,
    /// Value serialization documentation
    pub value: SchemaValueInfo,
    /// Relationships to other column families
    pub relationships: Vec<SchemaRelationshipInfo>,
}

/// Response listing all column family schemas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaColumnFamiliesResponse {
    pub column_families: Vec<ColumnFamilySchemaInfo>,
    pub total: usize,
}
