# Scratchpad (Codex)

Date: 2026-02-07

This is a lightweight internal note to prevent re-learning the repo structure and to
capture gotchas before making changes.

## What This Repo Is

Web UI + API for inspecting a Madara node's RocksDB (plus a SQLite index for richer queries).

Architecture:
- Frontend: Leptos CSR (Rust -> WASM) served as static assets.
- Backend: Axum API server that reads RocksDB read-only and optionally maintains a SQLite index.
- Schema docs: embedded YAML describing column families (key/value encodings + relationships).

## Repo Map

Rust workspace members (see `Cargo.toml`):
- `crates/api`: Axum server (single large `main.rs`) exposing `/api/...`.
- `crates/db-reader`: RocksDB read-only reader + decoding helpers.
- `crates/indexer`: SQLite indexer for complex queries (syncs from RocksDB via `db-reader`).
- `crates/schema`: Embedded YAML schema definitions, served via API.
- `crates/types`: Shared request/response types used by both API and frontend.
- `crates/frontend`: Leptos WASM app (currently implemented as one large `src/lib.rs`).

Other notable:
- Root `index.html` is a Trunk entrypoint that references `output.css` and the frontend crate.
- `.github/workflows/deploy.yml` builds `crates/frontend/index.html` and publishes `crates/frontend/dist` to GitHub Pages.

## Local Dev (Current Behavior)

Backend:
- `cargo run -p api -- --db-path <rocksdb_dir> --port 3000 --index-path <sqlite_path>`

Frontend (GitHub Pages workflow / local in `crates/frontend`):
- `cd crates/frontend && trunk serve index.html --port 8080`

Tailwind (root tooling):
- `npm run css` generates `output.css` from `input.css` (content includes `./index.html` and `./crates/frontend/src/**/*.rs`)

Note: There are 2 HTML entrypoints:
- `index.html` (root): uses compiled `output.css` + points Trunk at `crates/frontend/Cargo.toml`.
- `crates/frontend/index.html`: uses Tailwind via CDN + points Trunk at the local crate (`data-trunk rel="rust"`).

## Data Flow (High Level)

Frontend (`crates/frontend/src/lib.rs`) fetches JSON from the API:
- `fetch_*()` -> `GET/POST {API_BASE}/api/...`
- Views are selected via an internal `Page` enum (no router).

API (`crates/api/src/main.rs`) handlers call into:
- `DbReader` for direct RocksDB reads.
- `Indexer` (guarded by a `Mutex`) for indexed queries / SQL console.
- `schema` crate for schema docs endpoints.

## API Endpoints -> Main Implementations

Core:
- `GET /api/health` -> `health()`
- `GET /api/stats` -> `DbReader::get_stats()` (`crates/db-reader/src/queries.rs`)

Blocks / transactions:
- `GET /api/blocks` -> `DbReader::get_blocks()` (`crates/db-reader/src/blocks.rs`)
- `GET /api/blocks/{n}` -> `DbReader::get_block_detail()` (`crates/db-reader/src/blocks.rs`)
- `GET /api/blocks/{n}/transactions` -> `DbReader::get_block_transactions()` (`crates/db-reader/src/transactions.rs`)
- `GET /api/blocks/{n}/transactions/{idx}` -> `DbReader::get_transaction_detail()` (`crates/db-reader/src/transactions.rs`)
- `GET /api/transactions/{hash}` -> `DbReader::find_transaction_by_hash()` + `get_transaction_detail()`
- `GET /api/blocks/{n}/state-diff` -> `DbReader::get_state_diff()` (`crates/db-reader/src/state_diff.rs`)
- `GET /api/search?q=...` -> `DbReader::search()` (`crates/db-reader/src/queries.rs`)

Contracts / classes:
- `GET /api/contracts` -> `DbReader::list_contracts()` (`crates/db-reader/src/contracts.rs`)
- `GET /api/contracts/{addr}` -> `DbReader::get_contract()` (`crates/db-reader/src/contracts.rs`)
- `GET /api/contracts/{addr}/storage` -> `DbReader::get_contract_storage()` (`crates/db-reader/src/contracts.rs`)
- `GET /api/classes` -> `DbReader::list_classes()` (`crates/db-reader/src/contracts.rs`)
- `GET /api/classes/{hash}` -> `DbReader::get_class()` (`crates/db-reader/src/contracts.rs`)

Raw RocksDB browsing:
- `GET /api/raw/cf` -> `DbReader::list_column_families()` (`crates/db-reader/src/raw.rs`)
- `GET /api/raw/cf/{name}/stats` -> `DbReader::get_cf_stats()`
- `GET /api/raw/cf/{name}/keys` -> `DbReader::list_keys()` + `count_keys[_with_prefix]()`
- `GET /api/raw/cf/{name}/key/{key_hex}` -> `DbReader::get_raw_value()` + `decode_value_hint()`
- `POST /api/raw/cf/{name}/keys/batch` -> `DbReader::get_key_value_pairs()` + `decode_value_hint()`

Schema docs:
- `GET /api/schema/categories` -> `schema::load_all_schemas()` grouped by `category`
- `GET /api/schema/column-families` -> `schema::load_all_schemas()` or `load_schemas_by_category()`
- `GET /api/schema/column-families/{name}` -> `schema::get_schema_by_name()`

SQLite index:
- `GET /api/index/status` -> `Indexer::get_status()`
- `POST /api/index/sync` -> `Indexer::sync_from_db(&DbReader)`
- `GET /api/index/transactions` -> `Indexer::query_transactions(...)`
- `GET /api/index/contracts` -> `Indexer::query_contracts(...)`
- `GET /api/index/tables` -> `Indexer::list_tables()`
- `GET /api/index/tables/{name}/schema` -> `Indexer::get_table_schema()`
- `POST /api/index/query` -> `Indexer::execute_raw_query_with_params()` (SELECT-only, limited)

## RocksDB Assumptions / Encodings (Where It Matters)

Latest block:
- `DbReader::get_latest_block_number()` prefers `meta` CF key `CHAIN_TIP` (custom bincode varint-ish decode).
- Fallback scans the last key of `block_info` CF (handles 4-byte or 8-byte keys).

Block info:
- Reads from `block_info` CF: key = `u32` big-endian, value = bincode `MadaraBlockInfo`-like struct.

Transactions:
- Reads from `block_transactions` CF: key = `block_n(u32 BE) + tx_index(u16 BE)`, value = bincode `TransactionWithReceipt`-like struct.
- Tx hash lookup: `tx_hash_to_index` CF: key = 32-byte hash, value = bincode `(u32,u16)`.

Contracts:
- Class hash: `contract_class_hashes` CF, key = 32-byte address, value = bincode bytes (Felt).
- Nonce: `contract_nonces` CF, key = 32-byte address, value = bincode u64 (varint encoding).
- Storage: `contract_storage` CF, iteration currently prefixes only by address.

## Indexer Notes (SQLite)

- `Indexer::sync_from_db()` batches inserts in one SQLite transaction.
- It indexes:
  - blocks (basic header fields)
  - transactions (best-effort; uses `DbReader::get_transaction_detail()`)
  - events (from tx receipts)
  - storage_updates, deployed_contracts, declared classes (from state diffs)
  - contracts and classes (full scan up to hardcoded limit 10k)
- SQL console enforces:
  - SELECT-only (string-based keyword filtering)
  - timeout via `busy_timeout`
  - max rows = 1000

## Frontend Notes

API base URL selection order (`crates/frontend/src/lib.rs`):
1. `?api=...` query param (persisted to `localStorage["api_url"]`)
2. `localStorage["api_url"]`
3. If hostname is `localhost`/`127.0.0.1`, use `http://localhost:3000`
4. Fall back to `DEFAULT_API_URL` (Render)

Navigation:
- `Page` enum drives all views in `App()`.
- No URL routing; browser refresh always lands on the default `Page::BlockList`.

## Gotchas / Known Gaps (Before Editing Anything)

- `DbReader::get_block_transactions()` currently returns tx summaries from `block_info.tx_hashes` and hardcodes tx type/status to INVOKE/SUCCEEDED.
  - The API's block tx list view may therefore be misleading until this is wired to full receipt parsing.
- `DbReader::get_contract_storage()` iterates by address prefix only and does not dedupe by storage key.
  - Because Madara storage keys are versioned by block, this can return multiple versions of the same slot.
- Docker Compose packaging:
  - `.db-version` lives in Madara base-path (parent of `db/`), so the compose setup mounts the DB *parent* to keep that file visible.
  - The compose file uses `DB_DIR_NAME` to point `DB_PATH` at the actual RocksDB directory name under that mount (default `db`).
  - `./scripts/up.sh <rocksdb_or_base_path>` is the intended UX; it auto-detects base-path vs `db/` and sets `DB_DIR_NAME` correctly.
- Trunk / Tailwind entrypoint split:
  - Root `index.html` expects `output.css` (built via Tailwind CLI).
  - `crates/frontend/index.html` uses Tailwind CDN (workflow builds this file).
- `crates/frontend/dist` is tracked and changes whenever `trunk build` runs (hash-churn).
