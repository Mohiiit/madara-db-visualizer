use clap::Parser;
use db_reader::DbReader;
use directories::ProjectDirs;
use indexer::Indexer;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Parser, Debug)]
#[command(name = "makimono-viz")]
#[command(about = "Makimono toolchain: single-port Madara DB Visualizer (UI + API)")]
struct Args {
    /// Path to the Madara RocksDB database directory
    #[arg(long)]
    db_path: PathBuf,

    /// Path to the SQLite index database (defaults to MAKIMONO_HOME/state/index/<hash>.db)
    #[arg(long)]
    index_path: Option<PathBuf>,

    /// Host/IP to listen on
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Port to listen on
    #[arg(long, default_value_t = 8080)]
    port: u16,

    /// Disable the initial index sync (useful for very large DBs)
    #[arg(long)]
    no_initial_sync: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let db = match DbReader::open(&args.db_path) {
        Ok(db) => db,
        Err(e) => {
            eprintln!(
                "Failed to open database at {}: {}",
                args.db_path.display(),
                e
            );
            std::process::exit(1);
        }
    };

    let index_path = args
        .index_path
        .unwrap_or_else(|| default_index_path(&args.db_path));
    if let Some(parent) = index_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            eprintln!("Failed to create index directory {}: {e}", parent.display());
            std::process::exit(1);
        }
    }

    let indexer = match Indexer::open(&index_path) {
        Ok(idx) => idx,
        Err(e) => {
            eprintln!("Failed to open index at {}: {}", index_path.display(), e);
            std::process::exit(1);
        }
    };

    let state = Arc::new(api::AppState {
        db,
        indexer: Mutex::new(indexer),
    });

    if !args.no_initial_sync {
        let mut idx = state.indexer.lock().unwrap();
        match idx.sync_from_db(&state.db) {
            Ok(count) => eprintln!("Initial index sync: {count} blocks indexed"),
            Err(e) => eprintln!("Warning: initial index sync failed: {e}"),
        }
    }

    // Build API router (no CORS needed for same-origin).
    let api_router = api::build_router(state.clone(), None);

    // Serve embedded static UI for everything else.
    let app = api_router.fallback(axum::routing::get(|uri: axum::http::Uri| async move {
        api::embedded::response_for_uri(&uri)
    }));

    let addr = format!("{}:{}", args.host, args.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    let detected = state.db.detect_madara_db_version();
    let supported = detected
        .version
        .map(|v| api::SUPPORTED_MADARA_DB_VERSIONS.contains(&v));

    eprintln!("makimono-viz: http://{}", addr);
    eprintln!("db: {}", args.db_path.display());
    eprintln!("index: {}", index_path.display());
    if let Some(v) = detected.version {
        let src = detected
            .source_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "(unknown)".to_string());
        eprintln!(
            ".db-version: {v} (from {src}) supported={}",
            supported.unwrap_or(false)
        );
    } else if let Some(err) = detected.error {
        eprintln!(".db-version note: {err}");
    } else {
        eprintln!(".db-version: (not found)");
    }

    axum::serve(listener, app).await.unwrap();
}

fn default_index_path(db_path: &Path) -> PathBuf {
    let home = std::env::var_os("MAKIMONO_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            ProjectDirs::from("xyz", "karnot", "makimono").map(|p| p.data_dir().to_path_buf())
        });

    let home = home.unwrap_or_else(|| PathBuf::from(".makimono"));

    let mut hasher = Sha256::new();
    hasher.update(db_path.to_string_lossy().as_bytes());
    let digest = hasher.finalize();
    let hex = hex::encode(digest);

    home.join("state").join("index").join(format!("{hex}.db"))
}
