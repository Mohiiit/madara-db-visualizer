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
