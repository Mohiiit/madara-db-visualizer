# Development

## Prerequisites

- Rust (stable)
- `wasm32-unknown-unknown` target
- Python 3 (used for a WASM patch step)
- Optional: Node.js (only needed to rebuild Tailwind CSS)

```bash
rustup update
rustup target add wasm32-unknown-unknown
```

## Build

Standalone API server:
```bash
cargo build -p api --release --bin madara-db-visualizer-api
```

Single-port server (UI + API) with embedded assets:
```bash
./scripts/build_dist.sh
cargo build -p api --release --features embedded-ui --bin makimono-viz
```

## Run

```bash
# Single-port server
cargo run -p api --release --features embedded-ui --bin makimono-viz -- --db-path ./sample-db

# Standalone API server
cargo run -p api --release --bin madara-db-visualizer-api -- --db-path ./sample-db --index-path /tmp/madara_visualizer_index.db --port 3000
```

## Troubleshooting

### RocksDB path

The RocksDB directory should contain a `CURRENT` file (and typically `*.sst` files):
```bash
ls /path/to/db/CURRENT
ls /path/to/db/*.sst  # optional
```

### Port already in use

```bash
lsof -ti:3000,8080 | xargs kill -9
```

### Stale index data

```bash
rm /tmp/madara_visualizer_index.db
```

If you started via Docker Compose, the index is stored in a Docker volume:
```bash
docker compose -f compose.yaml down -v
```
