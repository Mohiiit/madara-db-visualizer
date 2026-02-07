//! Transaction reading functionality

use crate::blocks::Felt;
use crate::DbReader;
use serde::Deserialize;
use serde_bytes::ByteBuf;

// We need to deserialize the TransactionWithReceipt structure from madara's db.
// The structure is complex with enums for different transaction types.
// For now, we'll focus on extracting basic transaction info.

// Key structure (from blocks.rs in madara):
// block_transactions column: key = block_n(4 bytes) + tx_index(2 bytes)
// value = bincode(TransactionWithReceipt)

/// Transaction type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionType {
    Invoke,
    L1Handler,
    Declare,
    Deploy,
    DeployAccount,
}

impl std::fmt::Display for TransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionType::Invoke => write!(f, "INVOKE"),
            TransactionType::L1Handler => write!(f, "L1_HANDLER"),
            TransactionType::Declare => write!(f, "DECLARE"),
            TransactionType::Deploy => write!(f, "DEPLOY"),
            TransactionType::DeployAccount => write!(f, "DEPLOY_ACCOUNT"),
        }
    }
}

/// Execution result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionStatus {
    Succeeded,
    Reverted(String),
}

/// Transaction summary for list views
#[derive(Debug, Clone)]
pub struct TransactionSummary {
    pub tx_hash: String,
    pub tx_type: TransactionType,
    pub status: ExecutionStatus,
    pub block_number: u64,
    pub tx_index: usize,
}

/// Full transaction details
#[derive(Debug, Clone)]
pub struct TransactionDetail {
    pub tx_hash: String,
    pub tx_type: TransactionType,
    pub status: ExecutionStatus,
    pub block_number: u64,
    pub tx_index: usize,
    pub actual_fee: String,
    pub fee_unit: String,
    pub events: Vec<EventInfo>,
    pub messages_sent: Vec<MessageInfo>,
    // Common transaction fields
    pub sender_address: Option<String>,
    pub calldata: Vec<String>,
    pub signature: Vec<String>,
    pub nonce: Option<String>,
    pub version: Option<String>,
}

/// Event information
#[derive(Debug, Clone)]
pub struct EventInfo {
    pub from_address: String,
    pub keys: Vec<String>,
    pub data: Vec<String>,
}

/// Message to L1 information
#[derive(Debug, Clone)]
pub struct MessageInfo {
    pub from_address: String,
    pub to_address: String,
    pub payload: Vec<String>,
}

// Raw deserialization types for TransactionWithReceipt
// This matches madara's serialization format

#[derive(Debug, Clone, Deserialize)]
struct RawTransactionWithReceipt {
    pub transaction: RawTransaction,
    pub receipt: RawTransactionReceipt,
}

// Transaction is an enum with 5 variants
#[derive(Debug, Clone, Deserialize)]
enum RawTransaction {
    Invoke(RawInvokeTransaction),
    L1Handler(RawL1HandlerTransaction),
    Declare(RawDeclareTransaction),
    Deploy(RawDeployTransaction),
    DeployAccount(RawDeployAccountTransaction),
}

// Invoke transaction is also an enum with multiple versions
#[derive(Debug, Clone, Deserialize)]
enum RawInvokeTransaction {
    V0(RawInvokeTransactionV0),
    V1(RawInvokeTransactionV1),
    V3(RawInvokeTransactionV3),
}

#[derive(Debug, Clone, Deserialize)]
struct RawInvokeTransactionV0 {
    pub max_fee: ByteBuf,
    pub signature: Vec<ByteBuf>,
    pub contract_address: ByteBuf,
    pub entry_point_selector: ByteBuf,
    pub calldata: Vec<ByteBuf>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawInvokeTransactionV1 {
    pub sender_address: ByteBuf,
    pub calldata: Vec<ByteBuf>,
    pub max_fee: ByteBuf,
    pub signature: Vec<ByteBuf>,
    pub nonce: ByteBuf,
}

#[derive(Debug, Clone, Deserialize)]
struct RawInvokeTransactionV3 {
    pub sender_address: ByteBuf,
    pub calldata: Vec<ByteBuf>,
    pub signature: Vec<ByteBuf>,
    pub nonce: ByteBuf,
    pub resource_bounds: RawResourceBoundsMapping,
    pub tip: u64,
    pub paymaster_data: Vec<ByteBuf>,
    pub account_deployment_data: Vec<ByteBuf>,
    pub nonce_data_availability_mode: u32,
    pub fee_data_availability_mode: u32,
}

#[derive(Debug, Clone, Deserialize)]
struct RawResourceBoundsMapping {
    pub l1_gas: RawResourceBounds,
    pub l2_gas: RawResourceBounds,
    pub l1_data_gas: Option<RawResourceBounds>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawResourceBounds {
    pub max_amount: u64,
    pub max_price_per_unit: u128,
}

#[derive(Debug, Clone, Deserialize)]
struct RawL1HandlerTransaction {
    pub version: ByteBuf,
    pub nonce: u64,
    pub contract_address: ByteBuf,
    pub entry_point_selector: ByteBuf,
    pub calldata: Vec<ByteBuf>,
}

#[derive(Debug, Clone, Deserialize)]
enum RawDeclareTransaction {
    V0(RawDeclareTransactionV0),
    V1(RawDeclareTransactionV1),
    V2(RawDeclareTransactionV2),
    V3(RawDeclareTransactionV3),
}

#[derive(Debug, Clone, Deserialize)]
struct RawDeclareTransactionV0 {
    pub sender_address: ByteBuf,
    pub max_fee: ByteBuf,
    pub signature: Vec<ByteBuf>,
    pub class_hash: ByteBuf,
}

#[derive(Debug, Clone, Deserialize)]
struct RawDeclareTransactionV1 {
    pub sender_address: ByteBuf,
    pub max_fee: ByteBuf,
    pub signature: Vec<ByteBuf>,
    pub nonce: ByteBuf,
    pub class_hash: ByteBuf,
}

#[derive(Debug, Clone, Deserialize)]
struct RawDeclareTransactionV2 {
    pub sender_address: ByteBuf,
    pub compiled_class_hash: ByteBuf,
    pub max_fee: ByteBuf,
    pub signature: Vec<ByteBuf>,
    pub nonce: ByteBuf,
    pub class_hash: ByteBuf,
}

#[derive(Debug, Clone, Deserialize)]
struct RawDeclareTransactionV3 {
    pub sender_address: ByteBuf,
    pub compiled_class_hash: ByteBuf,
    pub signature: Vec<ByteBuf>,
    pub nonce: ByteBuf,
    pub class_hash: ByteBuf,
    pub resource_bounds: RawResourceBoundsMapping,
    pub tip: u64,
    pub paymaster_data: Vec<ByteBuf>,
    pub account_deployment_data: Vec<ByteBuf>,
    pub nonce_data_availability_mode: u32,
    pub fee_data_availability_mode: u32,
}

#[derive(Debug, Clone, Deserialize)]
struct RawDeployTransaction {
    pub version: ByteBuf,
    pub class_hash: ByteBuf,
    pub contract_address_salt: ByteBuf,
    pub constructor_calldata: Vec<ByteBuf>,
}

#[derive(Debug, Clone, Deserialize)]
enum RawDeployAccountTransaction {
    V1(RawDeployAccountTransactionV1),
    V3(RawDeployAccountTransactionV3),
}

#[derive(Debug, Clone, Deserialize)]
struct RawDeployAccountTransactionV1 {
    pub max_fee: ByteBuf,
    pub signature: Vec<ByteBuf>,
    pub nonce: ByteBuf,
    pub contract_address_salt: ByteBuf,
    pub constructor_calldata: Vec<ByteBuf>,
    pub class_hash: ByteBuf,
}

#[derive(Debug, Clone, Deserialize)]
struct RawDeployAccountTransactionV3 {
    pub signature: Vec<ByteBuf>,
    pub nonce: ByteBuf,
    pub contract_address_salt: ByteBuf,
    pub constructor_calldata: Vec<ByteBuf>,
    pub class_hash: ByteBuf,
    pub resource_bounds: RawResourceBoundsMapping,
    pub tip: u64,
    pub paymaster_data: Vec<ByteBuf>,
    pub nonce_data_availability_mode: u32,
    pub fee_data_availability_mode: u32,
}

// Receipt types
#[derive(Debug, Clone, Deserialize)]
enum RawTransactionReceipt {
    Invoke(RawInvokeTransactionReceipt),
    L1Handler(RawL1HandlerTransactionReceipt),
    Declare(RawDeclareTransactionReceipt),
    Deploy(RawDeployTransactionReceipt),
    DeployAccount(RawDeployAccountTransactionReceipt),
}

#[derive(Debug, Clone, Deserialize)]
struct RawInvokeTransactionReceipt {
    pub transaction_hash: ByteBuf,
    pub actual_fee: RawFeePayment,
    pub messages_sent: Vec<RawMsgToL1>,
    pub events: Vec<RawEvent>,
    pub execution_resources: RawExecutionResources,
    pub execution_result: RawExecutionResult,
}

#[derive(Debug, Clone, Deserialize)]
struct RawL1HandlerTransactionReceipt {
    pub message_hash: [u8; 32],
    pub transaction_hash: ByteBuf,
    pub actual_fee: RawFeePayment,
    pub messages_sent: Vec<RawMsgToL1>,
    pub events: Vec<RawEvent>,
    pub execution_resources: RawExecutionResources,
    pub execution_result: RawExecutionResult,
}

#[derive(Debug, Clone, Deserialize)]
struct RawDeclareTransactionReceipt {
    pub transaction_hash: ByteBuf,
    pub actual_fee: RawFeePayment,
    pub messages_sent: Vec<RawMsgToL1>,
    pub events: Vec<RawEvent>,
    pub execution_resources: RawExecutionResources,
    pub execution_result: RawExecutionResult,
}

#[derive(Debug, Clone, Deserialize)]
struct RawDeployTransactionReceipt {
    pub transaction_hash: ByteBuf,
    pub actual_fee: RawFeePayment,
    pub messages_sent: Vec<RawMsgToL1>,
    pub events: Vec<RawEvent>,
    pub execution_resources: RawExecutionResources,
    pub execution_result: RawExecutionResult,
    pub contract_address: ByteBuf,
}

#[derive(Debug, Clone, Deserialize)]
struct RawDeployAccountTransactionReceipt {
    pub transaction_hash: ByteBuf,
    pub actual_fee: RawFeePayment,
    pub messages_sent: Vec<RawMsgToL1>,
    pub events: Vec<RawEvent>,
    pub execution_resources: RawExecutionResources,
    pub execution_result: RawExecutionResult,
    pub contract_address: ByteBuf,
}

#[derive(Debug, Clone, Deserialize)]
struct RawFeePayment {
    pub amount: ByteBuf,
    pub unit: RawPriceUnit,
}

#[derive(Debug, Clone, Deserialize)]
enum RawPriceUnit {
    Wei,
    Fri,
}

#[derive(Debug, Clone, Deserialize)]
struct RawMsgToL1 {
    pub from_address: ByteBuf,
    pub to_address: ByteBuf,
    pub payload: Vec<ByteBuf>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawEvent {
    pub from_address: ByteBuf,
    pub keys: Vec<ByteBuf>,
    pub data: Vec<ByteBuf>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawExecutionResources {
    pub steps: u64,
    pub memory_holes: u64,
    pub range_check_builtin_applications: u64,
    pub pedersen_builtin_applications: u64,
    pub poseidon_builtin_applications: u64,
    pub ec_op_builtin_applications: u64,
    pub ecdsa_builtin_applications: u64,
    pub bitwise_builtin_applications: u64,
    pub keccak_builtin_applications: u64,
    pub segment_arena_builtin: u64,
    pub data_availability: RawGasVector,
    pub total_gas_consumed: RawGasVector,
}

#[derive(Debug, Clone, Deserialize)]
struct RawGasVector {
    pub l1_gas: u128,
    pub l1_data_gas: u128,
    pub l2_gas: u128,
}

#[derive(Debug, Clone, Deserialize)]
enum RawExecutionResult {
    Succeeded,
    Reverted { reason: String },
}

// Helper to make transaction column key (same as madara)
fn make_transaction_column_key(block_n: u32, tx_index: u16) -> [u8; 6] {
    let mut key = [0u8; 6];
    key[..4].copy_from_slice(&block_n.to_be_bytes());
    key[4..].copy_from_slice(&tx_index.to_be_bytes());
    key
}

impl DbReader {
    /// Get transaction by block number and index
    fn get_raw_transaction(
        &self,
        block_n: u64,
        tx_index: u64,
    ) -> Option<RawTransactionWithReceipt> {
        use bincode::Options;

        let cf = self.db.cf_handle("block_transactions")?;
        let block_n_u32 = u32::try_from(block_n).ok()?;
        let tx_index_u16 = u16::try_from(tx_index).ok()?;
        let key = make_transaction_column_key(block_n_u32, tx_index_u16);
        let value = self.db.get_cf(&cf, key).ok()??;

        let opts = bincode::DefaultOptions::new();
        match opts.deserialize::<RawTransactionWithReceipt>(&value) {
            Ok(tx) => Some(tx),
            Err(e) => {
                eprintln!("Failed to deserialize tx {}/{}: {}", block_n, tx_index, e);
                eprintln!("First 100 bytes: {:?}", &value[..100.min(value.len())]);
                None
            }
        }
    }

    /// Get transactions for a block using tx_hashes from block_info
    /// (simplified - no full transaction deserialization for now)
    pub fn get_block_transactions(&self, block_n: u64) -> Vec<TransactionSummary> {
        // Get the block info which has tx_hashes
        let block = match self.get_block_detail(block_n) {
            Some(b) => b,
            None => return vec![],
        };

        // Create transaction summaries from tx_hashes
        block
            .tx_hashes
            .into_iter()
            .enumerate()
            .map(|(tx_index, tx_hash)| TransactionSummary {
                tx_hash,
                tx_type: TransactionType::Invoke, // Default type - we'd need full deserialization to know
                status: ExecutionStatus::Succeeded, // Default - we'd need full deserialization
                block_number: block_n,
                tx_index,
            })
            .collect()
    }

    /// Find transaction by hash
    pub fn find_transaction_by_hash(&self, tx_hash: &str) -> Option<(u64, u64)> {
        // tx_hash_to_index column: key = tx_hash (32 bytes), value = (block_n u32, tx_index u16)
        use bincode::Options;

        let cf = self.db.cf_handle("tx_hash_to_index")?;

        // Parse the hex hash
        let hash_str = tx_hash.strip_prefix("0x").unwrap_or(tx_hash);
        let hash_bytes = hex::decode(hash_str).ok()?;

        // Pad to 32 bytes if needed
        let mut key = [0u8; 32];
        let len = hash_bytes.len().min(32);
        key[32 - len..].copy_from_slice(&hash_bytes[..len]);

        let value = self.db.get_cf(&cf, key).ok()??;

        let opts = bincode::DefaultOptions::new();
        let (block_n, tx_index): (u32, u16) = opts.deserialize(&value).ok()?;

        Some((block_n as u64, tx_index as u64))
    }

    /// Get transaction detail
    /// Note: Full deserialization is complex, so we return minimal info for now
    pub fn get_transaction_detail(&self, block_n: u64, tx_index: u64) -> Option<TransactionDetail> {
        // Try full deserialization first
        if let Some(raw_tx) = self.get_raw_transaction(block_n, tx_index) {
            return Some(raw_tx.to_detail(block_n, tx_index as usize));
        }

        // Fallback: return minimal info from block's tx_hashes
        let block = self.get_block_detail(block_n)?;
        let tx_hash = block.tx_hashes.get(tx_index as usize)?.clone();

        Some(TransactionDetail {
            tx_hash,
            tx_type: TransactionType::Invoke, // Unknown - full deserialization failed
            status: ExecutionStatus::Succeeded, // Unknown
            block_number: block_n,
            tx_index: tx_index as usize,
            actual_fee: "0x0".to_string(),
            fee_unit: "UNKNOWN".to_string(),
            events: vec![],
            messages_sent: vec![],
            sender_address: None,
            calldata: vec![],
            signature: vec![],
            nonce: None,
            version: None,
        })
    }

    /// Internal method to get raw block (re-exported from blocks module)
    fn get_raw_block_internal(
        &self,
        block_n: u64,
    ) -> Option<super::blocks::RawMadaraBlockInfoInternal> {
        use bincode::Options;

        let cf = self.db.cf_handle("block_info")?;
        let block_n_u32 = u32::try_from(block_n).ok()?;
        let value = self.db.get_cf(&cf, block_n_u32.to_be_bytes()).ok()??;

        let opts = bincode::DefaultOptions::new();
        match opts.deserialize::<super::blocks::RawMadaraBlockInfoInternal>(&value) {
            Ok(block) => Some(block),
            Err(e) => {
                eprintln!("Failed to deserialize block_info for {}: {}", block_n, e);
                None
            }
        }
    }
}

impl RawTransactionWithReceipt {
    fn to_summary(&self, block_number: u64, tx_index: usize) -> TransactionSummary {
        let tx_hash = self.get_tx_hash();
        let tx_type = self.get_tx_type();
        let status = self.get_status();

        TransactionSummary {
            tx_hash,
            tx_type,
            status,
            block_number,
            tx_index,
        }
    }

    fn to_detail(&self, block_number: u64, tx_index: usize) -> TransactionDetail {
        let tx_hash = self.get_tx_hash();
        let tx_type = self.get_tx_type();
        let status = self.get_status();
        let (actual_fee, fee_unit) = self.get_fee();
        let events = self.get_events();
        let messages_sent = self.get_messages();
        let (sender_address, calldata, signature, nonce, version) = self.get_tx_fields();

        TransactionDetail {
            tx_hash,
            tx_type,
            status,
            block_number,
            tx_index,
            actual_fee,
            fee_unit,
            events,
            messages_sent,
            sender_address,
            calldata,
            signature,
            nonce,
            version,
        }
    }

    fn get_tx_hash(&self) -> String {
        let bytes = match &self.receipt {
            RawTransactionReceipt::Invoke(r) => &r.transaction_hash,
            RawTransactionReceipt::L1Handler(r) => &r.transaction_hash,
            RawTransactionReceipt::Declare(r) => &r.transaction_hash,
            RawTransactionReceipt::Deploy(r) => &r.transaction_hash,
            RawTransactionReceipt::DeployAccount(r) => &r.transaction_hash,
        };
        Felt::from_bytes(bytes).to_hex()
    }

    fn get_tx_type(&self) -> TransactionType {
        match &self.transaction {
            RawTransaction::Invoke(_) => TransactionType::Invoke,
            RawTransaction::L1Handler(_) => TransactionType::L1Handler,
            RawTransaction::Declare(_) => TransactionType::Declare,
            RawTransaction::Deploy(_) => TransactionType::Deploy,
            RawTransaction::DeployAccount(_) => TransactionType::DeployAccount,
        }
    }

    fn get_status(&self) -> ExecutionStatus {
        let result = match &self.receipt {
            RawTransactionReceipt::Invoke(r) => &r.execution_result,
            RawTransactionReceipt::L1Handler(r) => &r.execution_result,
            RawTransactionReceipt::Declare(r) => &r.execution_result,
            RawTransactionReceipt::Deploy(r) => &r.execution_result,
            RawTransactionReceipt::DeployAccount(r) => &r.execution_result,
        };
        match result {
            RawExecutionResult::Succeeded => ExecutionStatus::Succeeded,
            RawExecutionResult::Reverted { reason } => ExecutionStatus::Reverted(reason.clone()),
        }
    }

    fn get_fee(&self) -> (String, String) {
        let fee = match &self.receipt {
            RawTransactionReceipt::Invoke(r) => &r.actual_fee,
            RawTransactionReceipt::L1Handler(r) => &r.actual_fee,
            RawTransactionReceipt::Declare(r) => &r.actual_fee,
            RawTransactionReceipt::Deploy(r) => &r.actual_fee,
            RawTransactionReceipt::DeployAccount(r) => &r.actual_fee,
        };
        let amount = Felt::from_bytes(&fee.amount).to_hex();
        let unit = match fee.unit {
            RawPriceUnit::Wei => "WEI".to_string(),
            RawPriceUnit::Fri => "FRI".to_string(),
        };
        (amount, unit)
    }

    fn get_events(&self) -> Vec<EventInfo> {
        let events = match &self.receipt {
            RawTransactionReceipt::Invoke(r) => &r.events,
            RawTransactionReceipt::L1Handler(r) => &r.events,
            RawTransactionReceipt::Declare(r) => &r.events,
            RawTransactionReceipt::Deploy(r) => &r.events,
            RawTransactionReceipt::DeployAccount(r) => &r.events,
        };
        events
            .iter()
            .map(|e| EventInfo {
                from_address: Felt::from_bytes(&e.from_address).to_hex(),
                keys: e
                    .keys
                    .iter()
                    .map(|k| Felt::from_bytes(k).to_hex())
                    .collect(),
                data: e
                    .data
                    .iter()
                    .map(|d| Felt::from_bytes(d).to_hex())
                    .collect(),
            })
            .collect()
    }

    fn get_messages(&self) -> Vec<MessageInfo> {
        let messages = match &self.receipt {
            RawTransactionReceipt::Invoke(r) => &r.messages_sent,
            RawTransactionReceipt::L1Handler(r) => &r.messages_sent,
            RawTransactionReceipt::Declare(r) => &r.messages_sent,
            RawTransactionReceipt::Deploy(r) => &r.messages_sent,
            RawTransactionReceipt::DeployAccount(r) => &r.messages_sent,
        };
        messages
            .iter()
            .map(|m| MessageInfo {
                from_address: Felt::from_bytes(&m.from_address).to_hex(),
                to_address: Felt::from_bytes(&m.to_address).to_hex(),
                payload: m
                    .payload
                    .iter()
                    .map(|p| Felt::from_bytes(p).to_hex())
                    .collect(),
            })
            .collect()
    }

    fn get_tx_fields(
        &self,
    ) -> (
        Option<String>,
        Vec<String>,
        Vec<String>,
        Option<String>,
        Option<String>,
    ) {
        match &self.transaction {
            RawTransaction::Invoke(tx) => match tx {
                RawInvokeTransaction::V0(t) => (
                    Some(Felt::from_bytes(&t.contract_address).to_hex()),
                    t.calldata
                        .iter()
                        .map(|c| Felt::from_bytes(c).to_hex())
                        .collect(),
                    t.signature
                        .iter()
                        .map(|s| Felt::from_bytes(s).to_hex())
                        .collect(),
                    None,
                    Some("0".to_string()),
                ),
                RawInvokeTransaction::V1(t) => (
                    Some(Felt::from_bytes(&t.sender_address).to_hex()),
                    t.calldata
                        .iter()
                        .map(|c| Felt::from_bytes(c).to_hex())
                        .collect(),
                    t.signature
                        .iter()
                        .map(|s| Felt::from_bytes(s).to_hex())
                        .collect(),
                    Some(Felt::from_bytes(&t.nonce).to_hex()),
                    Some("1".to_string()),
                ),
                RawInvokeTransaction::V3(t) => (
                    Some(Felt::from_bytes(&t.sender_address).to_hex()),
                    t.calldata
                        .iter()
                        .map(|c| Felt::from_bytes(c).to_hex())
                        .collect(),
                    t.signature
                        .iter()
                        .map(|s| Felt::from_bytes(s).to_hex())
                        .collect(),
                    Some(Felt::from_bytes(&t.nonce).to_hex()),
                    Some("3".to_string()),
                ),
            },
            RawTransaction::L1Handler(t) => (
                Some(Felt::from_bytes(&t.contract_address).to_hex()),
                t.calldata
                    .iter()
                    .map(|c| Felt::from_bytes(c).to_hex())
                    .collect(),
                vec![],
                Some(format!("{}", t.nonce)),
                Some(Felt::from_bytes(&t.version).to_hex()),
            ),
            RawTransaction::Declare(tx) => match tx {
                RawDeclareTransaction::V0(t) => (
                    Some(Felt::from_bytes(&t.sender_address).to_hex()),
                    vec![],
                    t.signature
                        .iter()
                        .map(|s| Felt::from_bytes(s).to_hex())
                        .collect(),
                    None,
                    Some("0".to_string()),
                ),
                RawDeclareTransaction::V1(t) => (
                    Some(Felt::from_bytes(&t.sender_address).to_hex()),
                    vec![],
                    t.signature
                        .iter()
                        .map(|s| Felt::from_bytes(s).to_hex())
                        .collect(),
                    Some(Felt::from_bytes(&t.nonce).to_hex()),
                    Some("1".to_string()),
                ),
                RawDeclareTransaction::V2(t) => (
                    Some(Felt::from_bytes(&t.sender_address).to_hex()),
                    vec![],
                    t.signature
                        .iter()
                        .map(|s| Felt::from_bytes(s).to_hex())
                        .collect(),
                    Some(Felt::from_bytes(&t.nonce).to_hex()),
                    Some("2".to_string()),
                ),
                RawDeclareTransaction::V3(t) => (
                    Some(Felt::from_bytes(&t.sender_address).to_hex()),
                    vec![],
                    t.signature
                        .iter()
                        .map(|s| Felt::from_bytes(s).to_hex())
                        .collect(),
                    Some(Felt::from_bytes(&t.nonce).to_hex()),
                    Some("3".to_string()),
                ),
            },
            RawTransaction::Deploy(t) => (
                None,
                t.constructor_calldata
                    .iter()
                    .map(|c| Felt::from_bytes(c).to_hex())
                    .collect(),
                vec![],
                None,
                Some(Felt::from_bytes(&t.version).to_hex()),
            ),
            RawTransaction::DeployAccount(tx) => match tx {
                RawDeployAccountTransaction::V1(t) => (
                    None,
                    t.constructor_calldata
                        .iter()
                        .map(|c| Felt::from_bytes(c).to_hex())
                        .collect(),
                    t.signature
                        .iter()
                        .map(|s| Felt::from_bytes(s).to_hex())
                        .collect(),
                    Some(Felt::from_bytes(&t.nonce).to_hex()),
                    Some("1".to_string()),
                ),
                RawDeployAccountTransaction::V3(t) => (
                    None,
                    t.constructor_calldata
                        .iter()
                        .map(|c| Felt::from_bytes(c).to_hex())
                        .collect(),
                    t.signature
                        .iter()
                        .map(|s| Felt::from_bytes(s).to_hex())
                        .collect(),
                    Some(Felt::from_bytes(&t.nonce).to_hex()),
                    Some("3".to_string()),
                ),
            },
        }
    }
}
