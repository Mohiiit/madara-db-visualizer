//! Block reading functionality

use crate::DbReader;
use serde::Deserialize;
use serde_bytes::ByteBuf;

/// Wrapper for Felt bytes that auto-pads to 32 bytes
#[derive(Debug, Clone)]
pub struct Felt(pub [u8; 32]);

impl Felt {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut buffer = [0u8; 32];
        let len = bytes.len().min(32);
        buffer[32 - len..].copy_from_slice(&bytes[..len]);
        Felt(buffer)
    }

    pub fn to_hex(&self) -> String {
        // Skip leading zeros for display
        let first_nonzero = self.0.iter().position(|&b| b != 0).unwrap_or(31);
        format!("0x{}", hex::encode(&self.0[first_nonzero..]))
    }
}

// Raw types for deserialization (bincode serializes Felt using serialize_bytes)

#[derive(Debug, Clone, Deserialize)]
struct RawGasPrices {
    pub eth_l1_gas_price: u128,
    pub strk_l1_gas_price: u128,
    pub eth_l1_data_gas_price: u128,
    pub strk_l1_data_gas_price: u128,
    pub eth_l2_gas_price: u128,
    pub strk_l2_gas_price: u128,
}

#[derive(Debug, Clone, Deserialize)]
enum RawL1DataAvailabilityMode {
    Calldata,
    Blob,
}

// StarknetVersion is a newtype around [u8; 4], so we match that
#[derive(Debug, Clone, Deserialize)]
struct RawStarknetVersion([u8; 4]);

// Use ByteBuf for Felt fields since Felt uses serialize_bytes/deserialize_bytes
#[derive(Debug, Clone, Deserialize)]
struct RawHeader {
    pub parent_block_hash: ByteBuf,
    pub block_number: u64,
    pub global_state_root: ByteBuf,
    pub sequencer_address: ByteBuf,
    pub block_timestamp: u64,
    pub transaction_count: u64,
    pub transaction_commitment: ByteBuf,
    pub event_count: u64,
    pub event_commitment: ByteBuf,
    pub state_diff_length: Option<u64>,
    pub state_diff_commitment: Option<ByteBuf>,
    pub receipt_commitment: Option<ByteBuf>,
    pub protocol_version: RawStarknetVersion,
    pub gas_prices: RawGasPrices,
    pub l1_da_mode: RawL1DataAvailabilityMode,
}

#[derive(Debug, Clone, Deserialize)]
struct RawMadaraBlockInfo {
    pub header: RawHeader,
    pub block_hash: ByteBuf,
    pub total_l2_gas_used: u128,
    pub tx_hashes: Vec<ByteBuf>,
}

// Internal type for transactions module
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RawMadaraBlockInfoInternal {
    pub header: RawHeaderInternal,
    pub block_hash: ByteBuf,
    pub total_l2_gas_used: u128,
    pub tx_hashes: Vec<ByteBuf>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RawHeaderInternal {
    pub parent_block_hash: ByteBuf,
    pub block_number: u64,
    pub global_state_root: ByteBuf,
    pub sequencer_address: ByteBuf,
    pub block_timestamp: u64,
    pub transaction_count: u64,
    pub transaction_commitment: ByteBuf,
    pub event_count: u64,
    pub event_commitment: ByteBuf,
    pub state_diff_length: Option<u64>,
    pub state_diff_commitment: Option<ByteBuf>,
    pub receipt_commitment: Option<ByteBuf>,
    pub protocol_version: RawStarknetVersion,
    pub gas_prices: RawGasPrices,
    pub l1_da_mode: RawL1DataAvailabilityMode,
}

/// Simplified block summary for API responses
#[derive(Debug, Clone)]
pub struct BlockSummary {
    pub block_number: u64,
    pub block_hash: String,
    pub parent_hash: String,
    pub timestamp: u64,
    pub transaction_count: u64,
}

/// Full block details
#[derive(Debug, Clone)]
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

impl From<RawMadaraBlockInfo> for BlockSummary {
    fn from(info: RawMadaraBlockInfo) -> Self {
        Self {
            block_number: info.header.block_number,
            block_hash: Felt::from_bytes(&info.block_hash).to_hex(),
            parent_hash: Felt::from_bytes(&info.header.parent_block_hash).to_hex(),
            timestamp: info.header.block_timestamp,
            transaction_count: info.header.transaction_count,
        }
    }
}

impl From<RawMadaraBlockInfo> for BlockDetail {
    fn from(info: RawMadaraBlockInfo) -> Self {
        Self {
            block_number: info.header.block_number,
            block_hash: Felt::from_bytes(&info.block_hash).to_hex(),
            parent_hash: Felt::from_bytes(&info.header.parent_block_hash).to_hex(),
            state_root: Felt::from_bytes(&info.header.global_state_root).to_hex(),
            sequencer_address: Felt::from_bytes(&info.header.sequencer_address).to_hex(),
            timestamp: info.header.block_timestamp,
            transaction_count: info.header.transaction_count,
            event_count: info.header.event_count,
            l2_gas_used: info.total_l2_gas_used,
            tx_hashes: info
                .tx_hashes
                .into_iter()
                .map(|f| Felt::from_bytes(&f).to_hex())
                .collect(),
        }
    }
}

impl DbReader {
    /// Get block info by block number with detailed error reporting
    fn get_raw_block(&self, block_n: u64) -> Option<RawMadaraBlockInfo> {
        use bincode::Options;

        let cf = self.db.cf_handle("block_info")?;
        let block_n_u32 = u32::try_from(block_n).ok()?;
        let value = self.db.get_cf(&cf, block_n_u32.to_be_bytes()).ok()??;

        // Use same options as madara: bincode::DefaultOptions::new()
        let opts = bincode::DefaultOptions::new();
        match opts.deserialize::<RawMadaraBlockInfo>(&value) {
            Ok(info) => Some(info),
            Err(e) => {
                eprintln!("Failed to deserialize block {}: {}", block_n, e);
                eprintln!("First 50 bytes: {:?}", &value[..50.min(value.len())]);
                None
            }
        }
    }

    /// Get block summary by block number
    pub fn get_block_summary(&self, block_n: u64) -> Option<BlockSummary> {
        self.get_raw_block(block_n).map(BlockSummary::from)
    }

    /// Get block detail by block number
    pub fn get_block_detail(&self, block_n: u64) -> Option<BlockDetail> {
        self.get_raw_block(block_n).map(BlockDetail::from)
    }

    /// Get paginated list of blocks (newest first)
    pub fn get_blocks(&self, offset: u64, limit: u64) -> Vec<BlockSummary> {
        let latest = match self.get_latest_block_number() {
            Some(n) => n,
            None => return vec![],
        };

        let mut blocks = Vec::with_capacity(limit as usize);

        // Calculate starting block (newest first, then apply offset)
        let start_block = latest.saturating_sub(offset);

        for i in 0..limit {
            let block_n = match start_block.checked_sub(i) {
                Some(n) => n,
                None => break,
            };

            if let Some(summary) = self.get_block_summary(block_n) {
                blocks.push(summary);
            }
        }

        blocks
    }
}
