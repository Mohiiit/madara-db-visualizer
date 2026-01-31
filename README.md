# Madara DB Visualizer

A web-based tool for exploring and visualizing Madara's RocksDB database.

## Features

- Browse blocks, transactions, and state
- View contract storage and class information
- Explore state diffs per block
- Search across all entity types
- Complex queries (failed txs, by sender, etc.)

## Tech Stack

- **Backend**: Axum (Rust)
- **Frontend**: Leptos (Rust → WASM)
- **Database**: RocksDB (read-only) + SQLite (index)
- **Styling**: TailwindCSS

## Quick Start

```bash
# Start the API server
cargo run -p api -- --db-path /path/to/madara/db

# Start the frontend (in another terminal)
trunk serve

# Open http://localhost:8080
```

## Development

See [PLAN.md](./PLAN.md) for the development roadmap and phase details.

## License

MIT
