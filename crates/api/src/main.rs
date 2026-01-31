use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use clap::Parser;
use db_reader::DbReader;
use serde::Deserialize;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use visualizer_types::{
    BlockDetail, BlockListResponse, BlockSummary, HealthResponse, StatsResponse,
};

#[derive(Parser, Debug)]
#[command(name = "madara-db-visualizer-api")]
#[command(about = "API server for Madara DB Visualizer")]
struct Args {
    /// Path to the Madara RocksDB database
    #[arg(long, default_value = "/tmp/madara_devnet_poc_v2/db")]
    db_path: String,

    /// Port to listen on
    #[arg(long, default_value = "3000")]
    port: u16,
}

struct AppState {
    db: DbReader,
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

    let state = Arc::new(AppState { db });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/stats", get(stats))
        .route("/api/blocks", get(blocks))
        .route("/api/blocks/{block_number}", get(block_detail))
        .with_state(state)
        .layer(cors);

    let addr = format!("0.0.0.0:{}", args.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("API server running on http://localhost:{}", args.port);
    println!("Database path: {}", args.db_path);
    axum::serve(listener, app).await.unwrap();
}
