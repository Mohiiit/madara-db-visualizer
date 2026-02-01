# Madara DB Visualizer

A web-based **Database Inspector** for Madara's RocksDB - like pgAdmin but for blockchain storage. Understand how data is stored, inspect raw keys/values, and write custom SQL queries.

## Features

### Database Inspector (New!)
- **Schema Browser**: Self-documenting schema for all 27+ RocksDB column families
  - Key encoding details (u64 big-endian, Felt 32-bytes, composite keys)
  - Value serialization (bincode, varint, raw bytes)
  - Field descriptions and relationships between column families
- **Raw Data Browser**: Inspect raw RocksDB key-value pairs
  - Browse all column families with key counts
  - View keys in hex with prefix filtering
  - Pagination and decoded hints
- **SQL Console**: Query the SQLite index directly
  - 7 indexed tables: blocks, transactions, events, storage_updates, deployed_contracts, classes, contracts
  - Table schema sidebar with "Insert SELECT template"
  - Results table with copy-as-JSON

### Block Explorer
- **Block Browser**: Browse blocks with pagination, view block details
- **Transaction Browser**: View transactions, calldata, signatures, events
- **Contract Viewer**: Lookup contracts by address, view storage slots
- **Class Browser**: Browse Sierra/Legacy classes
- **State Diff Viewer**: See all state changes in a block
- **Universal Search**: Search by block number, tx hash, contract address, or class hash
- **Export**: Download data as JSON, copy hashes to clipboard

## Quick Start

### Prerequisites

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install Trunk (WASM bundler)
cargo install trunk
```

### Step 1: Clone and Build

```bash
git clone https://github.com/Mohiiit/madara-db-visualizer.git
cd madara-db-visualizer

# Build the API server
cargo build -p api --release

# Build the frontend
cd crates/frontend
trunk build index.html --release
cd ../..
```

### Step 2: Find Your Madara Database Path

The database is typically located at:
- **Default**: `~/.madara/db`
- **Custom**: Check your Madara node's `--base-path` flag

The directory should contain `.sst` files (RocksDB data files).

```bash
# Verify the path contains SST files
ls ~/.madara/db/*.sst
```

### Step 3: Start the Servers

**Terminal 1 - API Server:**
```bash
# Replace with your actual database path
./target/release/api --db-path ~/.madara/db

# Or with cargo
cargo run -p api --release -- --db-path ~/.madara/db
```

**Terminal 2 - Frontend Server:**
```bash
# Serve the built frontend
cd crates/frontend/dist
python3 -m http.server 8080
```

### Step 4: Open the Visualizer

Open http://localhost:8080 in your browser.

## Configuration

### API Server Options

```bash
./target/release/api --help

Options:
  --db-path <PATH>  Path to Madara RocksDB database (required)
  --port <PORT>     API server port [default: 3000]
```

### Environment Variables

```bash
# Use local target directory (useful if default is on external SSD)
CARGO_TARGET_DIR=target cargo build -p api --release
```

## API Endpoints

### Block Explorer
| Endpoint | Description |
|----------|-------------|
| `GET /api/health` | Health check |
| `GET /api/stats` | Database statistics |
| `GET /api/blocks?offset=0&limit=20` | List blocks |
| `GET /api/blocks/:number` | Block details |
| `GET /api/blocks/:number/transactions` | Block transactions |
| `GET /api/blocks/:number/state-diff` | Block state diff |
| `GET /api/contracts/:address` | Contract details |
| `GET /api/classes/:hash` | Class details |
| `GET /api/search?q=<query>` | Universal search |

### Schema Documentation
| Endpoint | Description |
|----------|-------------|
| `GET /api/schema/categories` | List schema categories |
| `GET /api/schema/column-families` | List all CF schemas |
| `GET /api/schema/column-families/:name` | Detailed CF schema |

### Raw Data Inspection
| Endpoint | Description |
|----------|-------------|
| `GET /api/raw/cf` | List all column families with key counts |
| `GET /api/raw/cf/:name/stats` | CF statistics (first/last key) |
| `GET /api/raw/cf/:name/keys?limit=50&offset=0` | List keys in hex |
| `GET /api/raw/cf/:name/key/:key_hex` | Get value for key |
| `POST /api/raw/cf/:name/keys/batch` | Batch fetch keys |

### SQL Index
| Endpoint | Description |
|----------|-------------|
| `GET /api/index/status` | Index sync status |
| `GET /api/index/tables` | List indexed tables |
| `GET /api/index/tables/:name/schema` | Table schema |
| `POST /api/index/query` | Execute SQL query |

**SQL Query Example:**
```bash
curl -X POST http://localhost:3000/api/index/query \
  -H "Content-Type: application/json" \
  -d '{"sql": "SELECT * FROM blocks ORDER BY block_number DESC LIMIT 5", "params": []}'
```

## Project Structure

```
madara-db-visualizer/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── api/                # Axum HTTP server
│   ├── db-reader/          # RocksDB access layer
│   ├── frontend/           # Leptos WASM frontend
│   │   └── dist/           # Built frontend assets
│   ├── indexer/            # SQLite indexer
│   ├── schema/             # YAML schema definitions
│   └── types/              # Shared types
└── README.md
```

## Troubleshooting

### "Database path does not exist"
Ensure `--db-path` points to the RocksDB directory containing `.sst` files:
```bash
ls /path/to/db/*.sst  # Should list SST files
```

### Port already in use
```bash
# Kill processes on ports 3000 and 8080
lsof -ti:3000,8080 | xargs kill -9
```

### WASM build fails
```bash
rustup target add wasm32-unknown-unknown
```

### Permission denied (external SSD)
```bash
CARGO_TARGET_DIR=target cargo build -p api --release
```

### Stale index data
Delete the SQLite index to rebuild:
```bash
rm /tmp/madara_visualizer_index.db
```

## Development

```bash
# Run tests
cargo test --workspace

# Build all crates
cargo build --release

# Frontend with hot reload
cd crates/frontend
trunk serve index.html
```

## License

MIT
