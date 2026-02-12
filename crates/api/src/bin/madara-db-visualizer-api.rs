use clap::Parser;
use db_reader::DbReader;
use indexer::Indexer;
use std::sync::{Arc, Mutex};
use tower_http::cors::{Any, CorsLayer};

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

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let db = match DbReader::open(&args.db_path) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Failed to open database at {}: {}", args.db_path, e);
            std::process::exit(1);
        }
    };

    let indexer = match Indexer::open(&args.index_path) {
        Ok(idx) => idx,
        Err(e) => {
            eprintln!("Failed to open index at {}: {}", args.index_path, e);
            std::process::exit(1);
        }
    };

    let state = Arc::new(api::AppState {
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

    let app = api::build_router(state, Some(cors));

    let addr = format!("0.0.0.0:{}", args.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("API server running on http://localhost:{}", args.port);
    println!("Database path: {}", args.db_path);
    println!("Index path: {}", args.index_path);

    axum::serve(listener, app).await.unwrap();
}
