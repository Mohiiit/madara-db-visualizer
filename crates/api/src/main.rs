use axum::{
    extract::State,
    http::StatusCode,
    routing::get,
    Json, Router,
};
use clap::Parser;
use db_reader::DbReader;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use visualizer_types::{HealthResponse, StatsResponse};

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

async fn stats(State(state): State<Arc<AppState>>) -> Result<Json<StatsResponse>, (StatusCode, String)> {
    let db_stats = state.db.get_stats();

    Ok(Json(StatsResponse {
        db_path: db_stats.db_path,
        latest_block: db_stats.latest_block,
        column_count: db_stats.column_count,
        columns: db_stats.columns,
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
        .with_state(state)
        .layer(cors);

    let addr = format!("0.0.0.0:{}", args.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("API server running on http://localhost:{}", args.port);
    println!("Database path: {}", args.db_path);
    axum::serve(listener, app).await.unwrap();
}
