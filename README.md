# Madara DB Visualizer

A web-based tool for exploring and visualizing Madara's RocksDB database.

## Features

- **Block Explorer**: Browse blocks with pagination, view block details including hash, parent hash, state root, timestamps
- **Transaction Browser**: View all transactions in a block, transaction details including calldata, signature, events
- **Contract Viewer**: Lookup contracts by address, view class hash, nonce, and storage slots
- **Class Browser**: Browse declared classes, view class type (Sierra/Legacy) and compiled class hash
- **State Diff Viewer**: See all state changes in a block (deployed contracts, storage changes, nonce updates)
- **Universal Search**: Search by block number, transaction hash, contract address, or class hash
- **Advanced Filters**: Filter transactions by status (succeeded/reverted) and block range using SQLite index
- **Export**: Download block and transaction data as JSON
- **Copy to Clipboard**: One-click copy for all hash fields

## Tech Stack

- **Backend**: Axum (Rust) - lightweight async HTTP server
- **Frontend**: Leptos (Rust → WASM) - reactive UI framework
- **Database**: RocksDB (read-only) + SQLite (index for complex queries)
- **Styling**: TailwindCSS via CDN
- **Build**: Trunk for WASM compilation

## Prerequisites

- Rust toolchain (1.70+)
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- Trunk: `cargo install trunk`
- A Madara node database (RocksDB)

## Quick Start

### 1. Build the Frontend

```bash
cd /path/to/madara-db-visualizer

# Development build
trunk build crates/frontend/index.html

# Production build (optimized)
trunk build crates/frontend/index.html --release
```

### 2. Start the API Server

```bash
# Point to your Madara RocksDB database
cargo run -p api --release -- --db-path /path/to/madara/db

# Example with a local devnet database
cargo run -p api --release -- --db-path ~/.madara/db
```

The API server runs on `http://localhost:3000` by default.

### 3. Serve the Frontend

```bash
# Option A: Use trunk serve (development with hot reload)
trunk serve crates/frontend/index.html

# Option B: Serve the built files (production)
cd dist && python3 -m http.server 8080
```

### 4. Open the Visualizer

Navigate to `http://localhost:8080` in your browser.

## Configuration

### API Server Options

```bash
cargo run -p api -- --help

Options:
  --db-path <PATH>  Path to the Madara RocksDB database (required)
  --port <PORT>     API server port (default: 3000)
```

### Environment Variables

- `CARGO_TARGET_DIR`: Override the cargo target directory (useful for external SSDs)

## API Endpoints

| Endpoint | Description |
|----------|-------------|
| `GET /api/health` | Health check |
| `GET /api/stats` | Database statistics (latest block, column count) |
| `GET /api/blocks?offset=0&limit=20` | List blocks with pagination |
| `GET /api/blocks/:number` | Get block details |
| `GET /api/blocks/:number/transactions` | List transactions in a block |
| `GET /api/blocks/:number/transactions/:index` | Get transaction details |
| `GET /api/blocks/:number/state-diff` | Get state diff for a block |
| `GET /api/contracts?limit=50` | List contracts |
| `GET /api/contracts/:address` | Get contract details |
| `GET /api/contracts/:address/storage?limit=50` | Get contract storage |
| `GET /api/classes?limit=50` | List classes |
| `GET /api/classes/:hash` | Get class details |
| `GET /api/search?q=<query>` | Universal search |
| `GET /api/index/status` | SQLite index status |
| `GET /api/index/transactions?status=&block_from=&block_to=` | Filtered transactions |
| `POST /api/index/sync` | Trigger index sync |

## Project Structure

```
madara-db-visualizer/
├── Cargo.toml              # Workspace root
├── PLAN.md                 # Development roadmap
├── crates/
│   ├── api/                # Axum HTTP server
│   ├── db-reader/          # RocksDB access layer
│   ├── frontend/           # Leptos WASM frontend
│   ├── indexer/            # SQLite indexer for complex queries
│   └── types/              # Shared JSON types
└── dist/                   # Built frontend assets
```

## Development

See [PLAN.md](./PLAN.md) for the development roadmap and phase details.

### Running Tests

```bash
cargo test --workspace
```

### Building for Release

```bash
# Build everything
cargo build --release

# Build frontend with optimizations
trunk build crates/frontend/index.html --release
```

## Troubleshooting

### "Database path does not exist"
Ensure the `--db-path` points to the actual RocksDB directory (usually contains `.sst` files).

### Port already in use
Kill existing processes: `lsof -ti:3000 | xargs kill -9`

### WASM build fails
Ensure you have the WASM target: `rustup target add wasm32-unknown-unknown`

### Index shows stale data
The SQLite index persists at `/tmp/madara_visualizer_index.db`. Delete it to rebuild from scratch, or use the `/api/index/sync` endpoint.

## License

MIT
