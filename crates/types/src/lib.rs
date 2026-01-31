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
