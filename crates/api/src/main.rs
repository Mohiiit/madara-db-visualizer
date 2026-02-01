use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use db_reader::DbReader;
use indexer::Indexer;
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use tower_http::cors::{Any, CorsLayer};
use visualizer_types::{
    BlockDetail, BlockListResponse, BlockSummary, ClassListResponse, ClassResponse,
    ColumnFamilyInfo, ColumnFamilyListResponse, ColumnFamilyStats, ContractListResponse,
    ContractResponse, ContractStorageResponse, ContractStorageDiffInfo, DeclaredClassInfo,
    DeployedContractInfo, EventInfo, FilteredContractsResponse, FilteredTransactionsResponse,
    HealthResponse, IndexedTransactionInfo, IndexStatusResponse, KeyInfo, KeyListResponse,
    MessageInfo, NonceUpdateResponse, ReplacedClassInfo, SearchResponse, StateDiffResponse,
    StatsResponse, StorageDiffEntryInfo, StorageEntryResponse, TransactionDetail,
    TransactionListResponse, TransactionSummary,
};

#[derive(Parser, Debug)]
#[command(name = "madara-db-visualizer-api")]
#[command(about = "API server for Madara DB Visualizer")]
struct Args {
    /// Path to the Madara RocksDB database
    #[arg(long, default_value = "/tmp/madara_devnet_poc_v2/db")]
    db_path: String,

    /// Path to the SQLite index database
    #[arg(long, default_value = "/tmp/madara_visualizer_index.db")]
    index_path: String,

    /// Port to listen on
    #[arg(long, default_value = "3000")]
    port: u16,
}

struct AppState {
    db: DbReader,
    indexer: Mutex<Indexer>,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

async fn stats(State(state): State<Arc<AppState>>) -> Json<StatsResponse> {
    let db_stats = state.db.get_stats();

    Json(StatsResponse {
        db_path: db_stats.db_path,
        latest_block: db_stats.latest_block,
        column_count: db_stats.column_count,
        columns: db_stats.columns,
    })
}

#[derive(Deserialize)]
struct BlocksQuery {
    #[serde(default = "default_limit")]
    limit: u64,
    #[serde(default)]
    offset: u64,
}

fn default_limit() -> u64 {
    20
}

async fn blocks(
    State(state): State<Arc<AppState>>,
    Query(query): Query<BlocksQuery>,
) -> Json<BlockListResponse> {
    let blocks: Vec<BlockSummary> = state
        .db
        .get_blocks(query.offset, query.limit)
        .into_iter()
        .map(|b| BlockSummary {
            block_number: b.block_number,
            block_hash: b.block_hash,
            parent_hash: b.parent_hash,
            timestamp: b.timestamp,
            transaction_count: b.transaction_count,
        })
        .collect();

    let total = state.db.get_latest_block_number().map(|n| n + 1).unwrap_or(0);

    Json(BlockListResponse {
        blocks,
        total,
        offset: query.offset,
        limit: query.limit,
    })
}

async fn block_detail(
    State(state): State<Arc<AppState>>,
    Path(block_number): Path<u64>,
) -> Result<Json<BlockDetail>, (StatusCode, String)> {
    let block = state
        .db
        .get_block_detail(block_number)
        .ok_or((StatusCode::NOT_FOUND, format!("Block {} not found", block_number)))?;

    Ok(Json(BlockDetail {
        block_number: block.block_number,
        block_hash: block.block_hash,
        parent_hash: block.parent_hash,
        state_root: block.state_root,
        sequencer_address: block.sequencer_address,
        timestamp: block.timestamp,
        transaction_count: block.transaction_count,
        event_count: block.event_count,
        l2_gas_used: block.l2_gas_used,
        tx_hashes: block.tx_hashes,
    }))
}

async fn block_transactions(
    State(state): State<Arc<AppState>>,
    Path(block_number): Path<u64>,
) -> Result<Json<TransactionListResponse>, (StatusCode, String)> {
    let transactions = state.db.get_block_transactions(block_number);
    let total = transactions.len();

    let txs: Vec<TransactionSummary> = transactions
        .into_iter()
        .map(|tx| {
            let (status, revert_reason) = match tx.status {
                db_reader::ExecutionStatus::Succeeded => ("SUCCEEDED".to_string(), None),
                db_reader::ExecutionStatus::Reverted(reason) => ("REVERTED".to_string(), Some(reason)),
            };
            TransactionSummary {
                tx_hash: tx.tx_hash,
                tx_type: tx.tx_type.to_string(),
                status,
                revert_reason,
                block_number: tx.block_number,
                tx_index: tx.tx_index,
            }
        })
        .collect();

    Ok(Json(TransactionListResponse {
        transactions: txs,
        block_number,
        total,
    }))
}

/// Get transaction detail by block number and tx index
async fn transaction_detail_by_index(
    State(state): State<Arc<AppState>>,
    Path((block_number, tx_index)): Path<(u64, u64)>,
) -> Result<Json<TransactionDetail>, (StatusCode, String)> {
    let tx = state
        .db
        .get_transaction_detail(block_number, tx_index)
        .ok_or((StatusCode::NOT_FOUND, format!("Transaction at block {} index {} not found", block_number, tx_index)))?;

    let (status, revert_reason) = match tx.status {
        db_reader::ExecutionStatus::Succeeded => ("SUCCEEDED".to_string(), None),
        db_reader::ExecutionStatus::Reverted(reason) => ("REVERTED".to_string(), Some(reason)),
    };

    Ok(Json(TransactionDetail {
        tx_hash: tx.tx_hash,
        tx_type: tx.tx_type.to_string(),
        status,
        revert_reason,
        block_number: tx.block_number,
        tx_index: tx.tx_index,
        actual_fee: tx.actual_fee,
        fee_unit: tx.fee_unit,
        events: tx.events.into_iter().map(|e| EventInfo {
            from_address: e.from_address,
            keys: e.keys,
            data: e.data,
        }).collect(),
        messages_sent: tx.messages_sent.into_iter().map(|m| MessageInfo {
            from_address: m.from_address,
            to_address: m.to_address,
            payload: m.payload,
        }).collect(),
        sender_address: tx.sender_address,
        calldata: tx.calldata,
        signature: tx.signature,
        nonce: tx.nonce,
        version: tx.version,
    }))
}

async fn transaction_detail(
    State(state): State<Arc<AppState>>,
    Path(tx_hash): Path<String>,
) -> Result<Json<TransactionDetail>, (StatusCode, String)> {
    // Find the transaction by hash
    let (block_n, tx_index) = state
        .db
        .find_transaction_by_hash(&tx_hash)
        .ok_or((StatusCode::NOT_FOUND, format!("Transaction {} not found (hash lookup failed)", tx_hash)))?;

    let tx = state
        .db
        .get_transaction_detail(block_n, tx_index)
        .ok_or((StatusCode::NOT_FOUND, format!("Transaction {} not found (detail lookup failed)", tx_hash)))?;

    let (status, revert_reason) = match tx.status {
        db_reader::ExecutionStatus::Succeeded => ("SUCCEEDED".to_string(), None),
        db_reader::ExecutionStatus::Reverted(reason) => ("REVERTED".to_string(), Some(reason)),
    };

    Ok(Json(TransactionDetail {
        tx_hash: tx.tx_hash,
        tx_type: tx.tx_type.to_string(),
        status,
        revert_reason,
        block_number: tx.block_number,
        tx_index: tx.tx_index,
        actual_fee: tx.actual_fee,
        fee_unit: tx.fee_unit,
        events: tx.events.into_iter().map(|e| EventInfo {
            from_address: e.from_address,
            keys: e.keys,
            data: e.data,
        }).collect(),
        messages_sent: tx.messages_sent.into_iter().map(|m| MessageInfo {
            from_address: m.from_address,
            to_address: m.to_address,
            payload: m.payload,
        }).collect(),
        sender_address: tx.sender_address,
        calldata: tx.calldata,
        signature: tx.signature,
        nonce: tx.nonce,
        version: tx.version,
    }))
}

// Contract endpoints

#[derive(Deserialize)]
struct LimitQuery {
    #[serde(default = "default_limit_usize")]
    limit: usize,
}

fn default_limit_usize() -> usize {
    20
}

async fn contracts(
    State(state): State<Arc<AppState>>,
    Query(query): Query<LimitQuery>,
) -> Json<ContractListResponse> {
    let contracts: Vec<ContractResponse> = state
        .db
        .list_contracts(query.limit)
        .into_iter()
        .map(|c| ContractResponse {
            address: c.address,
            class_hash: c.class_hash,
            nonce: c.nonce,
        })
        .collect();

    let total = contracts.len();

    Json(ContractListResponse { contracts, total })
}

async fn contract_detail(
    State(state): State<Arc<AppState>>,
    Path(address): Path<String>,
) -> Result<Json<ContractResponse>, (StatusCode, String)> {
    let contract = state
        .db
        .get_contract(&address)
        .ok_or((StatusCode::NOT_FOUND, format!("Contract {} not found", address)))?;

    Ok(Json(ContractResponse {
        address: contract.address,
        class_hash: contract.class_hash,
        nonce: contract.nonce,
    }))
}

async fn contract_storage(
    State(state): State<Arc<AppState>>,
    Path(address): Path<String>,
    Query(query): Query<LimitQuery>,
) -> Json<ContractStorageResponse> {
    let entries: Vec<StorageEntryResponse> = state
        .db
        .get_contract_storage(&address, query.limit)
        .into_iter()
        .map(|e| StorageEntryResponse {
            key: e.key,
            value: e.value,
        })
        .collect();

    let total = entries.len();

    Json(ContractStorageResponse {
        address,
        entries,
        total,
    })
}

async fn classes(
    State(state): State<Arc<AppState>>,
    Query(query): Query<LimitQuery>,
) -> Json<ClassListResponse> {
    let classes: Vec<ClassResponse> = state
        .db
        .list_classes(query.limit)
        .into_iter()
        .map(|c| ClassResponse {
            class_hash: c.class_hash,
            class_type: c.class_type.to_string(),
            compiled_class_hash: c.compiled_class_hash,
        })
        .collect();

    let total = classes.len();

    Json(ClassListResponse { classes, total })
}

async fn class_detail(
    State(state): State<Arc<AppState>>,
    Path(class_hash): Path<String>,
) -> Result<Json<ClassResponse>, (StatusCode, String)> {
    let class = state
        .db
        .get_class(&class_hash)
        .ok_or((StatusCode::NOT_FOUND, format!("Class {} not found", class_hash)))?;

    Ok(Json(ClassResponse {
        class_hash: class.class_hash,
        class_type: class.class_type.to_string(),
        compiled_class_hash: class.compiled_class_hash,
    }))
}

// State diff endpoint

async fn block_state_diff(
    State(state): State<Arc<AppState>>,
    Path(block_number): Path<u64>,
) -> Result<Json<StateDiffResponse>, (StatusCode, String)> {
    let diff = state
        .db
        .get_state_diff(block_number)
        .ok_or((StatusCode::NOT_FOUND, format!("State diff for block {} not found", block_number)))?;

    Ok(Json(StateDiffResponse {
        block_number,
        deployed_contracts: diff
            .deployed_contracts
            .into_iter()
            .map(|d| DeployedContractInfo {
                address: d.address,
                class_hash: d.class_hash,
            })
            .collect(),
        storage_diffs: diff
            .storage_diffs
            .into_iter()
            .map(|s| ContractStorageDiffInfo {
                address: s.address,
                storage_entries: s
                    .storage_entries
                    .into_iter()
                    .map(|e| StorageDiffEntryInfo {
                        key: e.key,
                        value: e.value,
                    })
                    .collect(),
            })
            .collect(),
        declared_classes: diff
            .declared_classes
            .into_iter()
            .map(|d| DeclaredClassInfo {
                class_hash: d.class_hash,
                compiled_class_hash: d.compiled_class_hash,
            })
            .collect(),
        nonces: diff
            .nonces
            .into_iter()
            .map(|n| NonceUpdateResponse {
                contract_address: n.contract_address,
                nonce: n.nonce,
            })
            .collect(),
        replaced_classes: diff
            .replaced_classes
            .into_iter()
            .map(|r| ReplacedClassInfo {
                contract_address: r.contract_address,
                class_hash: r.class_hash,
            })
            .collect(),
    }))
}

// Search endpoint

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
}

async fn search(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Json<SearchResponse> {
    use db_reader::SearchResult;

    match state.db.search(&query.q) {
        SearchResult::Block(block_n) => Json(SearchResponse {
            result_type: "block".to_string(),
            block_number: Some(block_n),
            tx_index: None,
            address: None,
            class_hash: None,
        }),
        SearchResult::Transaction { block_n, tx_index } => Json(SearchResponse {
            result_type: "transaction".to_string(),
            block_number: Some(block_n),
            tx_index: Some(tx_index),
            address: None,
            class_hash: None,
        }),
        SearchResult::Contract(address) => Json(SearchResponse {
            result_type: "contract".to_string(),
            block_number: None,
            tx_index: None,
            address: Some(address),
            class_hash: None,
        }),
        SearchResult::Class(class_hash) => Json(SearchResponse {
            result_type: "class".to_string(),
            block_number: None,
            tx_index: None,
            address: None,
            class_hash: Some(class_hash),
        }),
        SearchResult::NotFound => Json(SearchResponse {
            result_type: "not_found".to_string(),
            block_number: None,
            tx_index: None,
            address: None,
            class_hash: None,
        }),
    }
}

// Index endpoints

async fn index_status(State(state): State<Arc<AppState>>) -> Json<IndexStatusResponse> {
    let indexer = state.indexer.lock().unwrap();
    match indexer.get_status() {
        Ok(status) => Json(IndexStatusResponse {
            indexed_blocks: status.indexed_blocks,
            latest_block: status.latest_block,
            is_synced: status.is_synced,
            total_transactions: status.total_transactions,
            failed_transactions: status.failed_transactions,
        }),
        Err(_) => Json(IndexStatusResponse {
            indexed_blocks: 0,
            latest_block: 0,
            is_synced: false,
            total_transactions: 0,
            failed_transactions: 0,
        }),
    }
}

async fn index_sync(State(state): State<Arc<AppState>>) -> Result<Json<IndexStatusResponse>, (StatusCode, String)> {
    let mut indexer = state.indexer.lock().unwrap();
    match indexer.sync_from_db(&state.db) {
        Ok(count) => {
            println!("Indexed {} blocks", count);
            match indexer.get_status() {
                Ok(status) => Ok(Json(IndexStatusResponse {
                    indexed_blocks: status.indexed_blocks,
                    latest_block: status.latest_block,
                    is_synced: status.is_synced,
                    total_transactions: status.total_transactions,
                    failed_transactions: status.failed_transactions,
                })),
                Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
            }
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

#[derive(Deserialize)]
struct TransactionFilterQuery {
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    sender: Option<String>,
    #[serde(default)]
    block_from: Option<u64>,
    #[serde(default)]
    block_to: Option<u64>,
    #[serde(default = "default_limit_usize")]
    limit: usize,
}

async fn filtered_transactions(
    State(state): State<Arc<AppState>>,
    Query(query): Query<TransactionFilterQuery>,
) -> Json<FilteredTransactionsResponse> {
    let indexer = state.indexer.lock().unwrap();
    let transactions = indexer
        .query_transactions(
            query.status.as_deref(),
            query.sender.as_deref(),
            query.block_from,
            query.block_to,
            query.limit,
        )
        .unwrap_or_default();

    let total = transactions.len();
    let txs: Vec<IndexedTransactionInfo> = transactions
        .into_iter()
        .map(|t| IndexedTransactionInfo {
            tx_hash: t.tx_hash,
            block_number: t.block_number,
            tx_index: t.tx_index,
            tx_type: t.tx_type,
            status: t.status,
            revert_reason: t.revert_reason,
            sender_address: t.sender_address,
        })
        .collect();

    Json(FilteredTransactionsResponse {
        transactions: txs,
        total,
    })
}

#[derive(Deserialize)]
struct ContractFilterQuery {
    #[serde(default)]
    class_hash: Option<String>,
    #[serde(default = "default_limit_usize")]
    limit: usize,
}

async fn filtered_contracts(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ContractFilterQuery>,
) -> Json<FilteredContractsResponse> {
    let indexer = state.indexer.lock().unwrap();
    let contracts = indexer
        .query_contracts(query.class_hash.as_deref(), query.limit)
        .unwrap_or_default();

    let total = contracts.len();
    let results: Vec<ContractResponse> = contracts
        .into_iter()
        .map(|c| ContractResponse {
            address: c.address,
            class_hash: c.class_hash,
            nonce: c.nonce,
        })
        .collect();

    Json(FilteredContractsResponse {
        contracts: results,
        total,
    })
}

// Raw column family browsing endpoints

/// List all column families
async fn raw_list_column_families(
    State(state): State<Arc<AppState>>,
) -> Json<ColumnFamilyListResponse> {
    let cf_names = state.db.list_column_families();

    let column_families: Vec<ColumnFamilyInfo> = cf_names
        .into_iter()
        .map(|name| {
            let key_count = state.db.count_keys(&name);
            ColumnFamilyInfo { name, key_count }
        })
        .collect();

    Json(ColumnFamilyListResponse { column_families })
}

/// Get statistics for a specific column family
async fn raw_cf_stats(
    State(state): State<Arc<AppState>>,
    Path(cf_name): Path<String>,
) -> Result<Json<ColumnFamilyStats>, (StatusCode, String)> {
    let stats = state
        .db
        .get_cf_stats(&cf_name)
        .ok_or((StatusCode::NOT_FOUND, format!("Column family '{}' not found", cf_name)))?;

    Ok(Json(ColumnFamilyStats {
        name: stats.name,
        key_count: stats.key_count,
        first_key_hex: stats.first_key_hex,
        last_key_hex: stats.last_key_hex,
    }))
}

#[derive(Deserialize)]
struct KeysQuery {
    #[serde(default = "default_keys_limit")]
    limit: usize,
    #[serde(default)]
    offset: usize,
    #[serde(default)]
    prefix: Option<String>,
}

fn default_keys_limit() -> usize {
    50
}

/// List keys in a column family with pagination
async fn raw_list_keys(
    State(state): State<Arc<AppState>>,
    Path(cf_name): Path<String>,
    Query(query): Query<KeysQuery>,
) -> Result<Json<KeyListResponse>, (StatusCode, String)> {
    // Verify the column family exists
    let cf_names = state.db.list_column_families();
    if !cf_names.contains(&cf_name) {
        return Err((StatusCode::NOT_FOUND, format!("Column family '{}' not found", cf_name)));
    }

    // Parse prefix if provided (expects hex string with optional 0x prefix)
    let prefix_bytes: Option<Vec<u8>> = query.prefix.as_ref().and_then(|p| {
        let hex_str = p.strip_prefix("0x").unwrap_or(p);
        hex::decode(hex_str).ok()
    });

    // Get total count
    let total = if let Some(ref prefix) = prefix_bytes {
        state.db.count_keys_with_prefix(&cf_name, prefix)
    } else {
        state.db.count_keys(&cf_name)
    };

    // Get keys with pagination
    let keys_raw = state.db.list_keys(
        &cf_name,
        query.limit,
        query.offset,
        prefix_bytes.as_deref(),
    );

    let keys: Vec<KeyInfo> = keys_raw
        .into_iter()
        .map(|k| KeyInfo {
            raw_hex: format!("0x{}", hex::encode(&k)),
        })
        .collect();

    let has_more = query.offset + keys.len() < total;

    Ok(Json(KeyListResponse {
        cf_name,
        keys,
        total,
        offset: query.offset,
        limit: query.limit,
        has_more,
    }))
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Open database
    let db = match DbReader::open(&args.db_path) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Failed to open database at {}: {}", args.db_path, e);
            std::process::exit(1);
        }
    };

    // Open indexer
    let indexer = match Indexer::open(&args.index_path) {
        Ok(idx) => idx,
        Err(e) => {
            eprintln!("Failed to open index at {}: {}", args.index_path, e);
            std::process::exit(1);
        }
    };

    let state = Arc::new(AppState {
        db,
        indexer: Mutex::new(indexer),
    });

    // Initial sync
    {
        let mut idx = state.indexer.lock().unwrap();
        match idx.sync_from_db(&state.db) {
            Ok(count) => println!("Initial index sync: {} blocks indexed", count),
            Err(e) => eprintln!("Warning: Initial index sync failed: {}", e),
        }
    }

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/stats", get(stats))
        .route("/api/blocks", get(blocks))
        .route("/api/blocks/{block_number}", get(block_detail))
        .route("/api/blocks/{block_number}/transactions", get(block_transactions))
        .route("/api/blocks/{block_number}/transactions/{tx_index}", get(transaction_detail_by_index))
        .route("/api/transactions/{tx_hash}", get(transaction_detail))
        .route("/api/contracts", get(contracts))
        .route("/api/contracts/{address}", get(contract_detail))
        .route("/api/contracts/{address}/storage", get(contract_storage))
        .route("/api/classes", get(classes))
        .route("/api/classes/{class_hash}", get(class_detail))
        .route("/api/blocks/{block_number}/state-diff", get(block_state_diff))
        .route("/api/search", get(search))
        // Index endpoints
        .route("/api/index/status", get(index_status))
        .route("/api/index/sync", post(index_sync))
        .route("/api/index/transactions", get(filtered_transactions))
        .route("/api/index/contracts", get(filtered_contracts))
        // Raw column family browsing endpoints
        .route("/api/raw/cf", get(raw_list_column_families))
        .route("/api/raw/cf/{name}/stats", get(raw_cf_stats))
        .route("/api/raw/cf/{name}/keys", get(raw_list_keys))
        .with_state(state)
        .layer(cors);

    let addr = format!("0.0.0.0:{}", args.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("API server running on http://localhost:{}", args.port);
    println!("Database path: {}", args.db_path);
    println!("Index path: {}", args.index_path);
    axum::serve(listener, app).await.unwrap();
}
