# Madara DB Visualizer - Development Plan

A web-based visualizer for Madara's RocksDB database.

## Configuration

| Setting | Value |
|---------|-------|
| **Test DB Path** | `/tmp/madara_devnet_poc_v2/db` |
| **Type Reuse** | Import from `mc-db` crate (madara repo) |
| **Authentication** | None required |
| **Real-time Updates** | Not required (some lag acceptable) |
| **Repo Location** | `/Users/mohit/Desktop/karnot/madara-db-visualizer` |

---

## Tech Stack

| Layer | Choice | Rationale |
|-------|--------|-----------|
| **Backend** | Axum | Lightweight, async, Tokio ecosystem |
| **Frontend** | Leptos | Fast, Rust → WASM, fine-grained reactivity |
| **Styling** | TailwindCSS | Utility-first, rapid iteration |
| **DB Access** | RocksDB (read-only) | Direct Madara storage access |
| **Index** | SQLite (Phase 4+) | Complex queries |
| **Build** | Trunk | Standard Rust WASM tooling |
| **Feedback** | agent-browser | Visual verification |

---

## Code Structure

```
madara-db-visualizer/
├── Cargo.toml                    # Workspace root
├── Trunk.toml                    # WASM build config
├── index.html                    # Entry point
├── input.css                     # Tailwind input
├── tailwind.config.js
│
├── crates/
│   ├── db-reader/                # RocksDB access (reuses mc-db types)
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── connection.rs     # Read-only RocksDB
│   │       └── queries.rs        # Query functions
│   │
│   ├── api/                      # Axum HTTP server
│   │   └── src/
│   │       ├── main.rs
│   │       └── routes/
│   │
│   ├── frontend/                 # Leptos WASM
│   │   └── src/
│   │       ├── main.rs
│   │       ├── app.rs
│   │       └── components/
│   │
│   └── types/                    # Shared JSON types
│       └── src/lib.rs
│
└── static/
```

---

## Development Workflow

Each phase follows this cycle:

```
┌─────────────────────────────────────────────────────────────┐
│                     Phase N Workflow                        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  1. IMPLEMENT                                               │
│     └─ Write code for phase requirements                   │
│                                                             │
│  2. VERIFY                                                  │
│     └─ trunk serve (frontend)                              │
│     └─ cargo run -p api (backend)                          │
│     └─ agent-browser snapshot + screenshot                 │
│                                                             │
│  3. SELF-FEEDBACK                                           │
│     └─ Review screenshot: Does it look right?              │
│     └─ Test interactions: Do clicks work?                  │
│     └─ Check data: Is it accurate?                         │
│     └─ Note issues and fix them                            │
│                                                             │
│  4. CHECKPOINT COMMIT                                       │
│     └─ git add -A                                          │
│     └─ git commit -m "phase-N: <description>"              │
│                                                             │
│  5. ITERATE if needed, then move to Phase N+1              │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## Phases

### Phase 0: Project Scaffold
**Commit**: `phase-0: project scaffold with hello world`

**Goal**: Working project structure with basic hello world.

**Deliverables**:
- [ ] Workspace Cargo.toml
- [ ] Basic Axum server at `localhost:3000`
- [ ] Basic Leptos app at `localhost:8080`
- [ ] TailwindCSS integrated
- [ ] `GET /api/health` → `{"status": "ok"}`

**Verification**:
```bash
# Terminal 1
cargo run -p api

# Terminal 2
trunk serve

# Terminal 3
agent-browser open http://localhost:8080
agent-browser snapshot -i
agent-browser screenshot /tmp/phase0.png
```

**Success**: Page shows "Madara DB Visualizer", API returns health OK.

---

### Phase 1: Database Connection
**Commit**: `phase-1: rocksdb connection and stats endpoint`

**Goal**: Connect to RocksDB, show basic stats.

**Deliverables**:
- [ ] RocksDB read-only connection
- [ ] `GET /api/stats` → column list, latest block, tx count
- [ ] Frontend displays stats

**API**:
```
GET /api/stats
→ { "db_path": "...", "latest_block": 100, "columns": [...] }
```

**UI**:
```
┌─────────────────────────────────────┐
│ Madara DB Visualizer                │
├─────────────────────────────────────┤
│ Database: /tmp/madara_devnet_poc_v2 │
│ Latest Block: 100                   │
│ Columns: 18                         │
└─────────────────────────────────────┘
```

**Success**: Stats from actual DB displayed in UI.

---

### Phase 2: Block Explorer
**Commit**: `phase-2: block list and detail views`

**Goal**: Browse blocks with pagination, view block details.

**Deliverables**:
- [ ] `GET /api/blocks?limit=20&offset=0`
- [ ] `GET /api/blocks/:number`
- [ ] Block list component with pagination
- [ ] Block detail component
- [ ] Navigation: sidebar + prev/next

**UI**:
```
┌────────────┬─────────────────────────────┐
│ Sidebar    │ Block #100                  │
│            │                             │
│ • Blocks ◄ │ Hash: 0x7a8b...             │
│ • Txns     │ Parent: 0x6f5e...           │
│ • State    │ Timestamp: 2024-01-15       │
│            │ Transactions: 45            │
│            │                             │
│            │ [◄ Prev] [Next ►]           │
├────────────┼─────────────────────────────┤
│ Block List │ #100 | 0x7a8b... | 45 txns  │
│            │ #99  | 0x6f5e... | 32 txns  │
│            │ #98  | 0x5e4d... | 28 txns  │
└────────────┴─────────────────────────────┘
```

**Success**: Can browse blocks, click to see details, navigate prev/next.

---

### Phase 3: Transaction Browser
**Commit**: `phase-3: transaction list and details`

**Goal**: View transactions in a block, transaction details.

**Deliverables**:
- [ ] `GET /api/blocks/:number/transactions`
- [ ] `GET /api/transactions/:hash`
- [ ] Transaction list per block
- [ ] Transaction detail (type, status, fee, events)
- [ ] Revert reason for failed txs

**UI**:
```
┌────────────┬─────────────────────────────┐
│ Sidebar    │ Transaction 0x1a2b...       │
│            │                             │
│ • Blocks   │ Block: #100 (index 5)       │
│ • Txns  ◄  │ Type: INVOKE_V3             │
│ • State    │ Status: ✓ Succeeded         │
│            │ Fee: 0.00012 ETH            │
│            │                             │
│            │ Events (3):                 │
│            │ • Transfer(...)             │
│            │ • Approval(...)             │
└────────────┴─────────────────────────────┘
```

**Success**: Can view txs in block, click for details, see events.

---

### Phase 4: Contract & Class Viewer
**Commit**: `phase-4: contract state and class browser`

**Goal**: View contract storage and class information.

**Deliverables**:
- [ ] `GET /api/contracts/:address`
- [ ] `GET /api/contracts/:address/storage`
- [ ] `GET /api/classes/:hash`
- [ ] Contract view (nonce, class hash, storage slots)
- [ ] Class view (type, compiled hash, ABI)

**UI**:
```
┌────────────┬─────────────────────────────┐
│ Sidebar    │ Contract 0x049d...          │
│            │                             │
│ • Blocks   │ Class: 0x07b8...            │
│ • Txns     │ Nonce: 42                   │
│ • State ◄  │                             │
│ • Classes  │ Storage:                    │
│            │ 0x01 → 0x1234...            │
│            │ 0x02 → 0x5678...            │
│            │                             │
│            │ [Search key: ________]      │
└────────────┴─────────────────────────────┘
```

**Success**: Can lookup contract, view storage, view class info.

---

### Phase 5: State Diff & Search
**Commit**: `phase-5: state diff viewer and search`

**Goal**: View state changes per block, global search.

**Deliverables**:
- [ ] `GET /api/blocks/:number/state-diff`
- [ ] `GET /api/search?q=...`
- [ ] State diff view (deployed, storage changes, nonces)
- [ ] Universal search bar
- [ ] Auto-detect search type (block/tx/contract/class)

**UI**:
```
┌─────────────────────────────────────────┐
│ [🔍 Search: 0x049d...              ]   │
├────────────┬────────────────────────────┤
│ Block #100 │ State Diff                 │
│ ├─ Info    │                            │
│ ├─ Txns    │ Deployed (2):              │
│ └─ Diff ◄  │ • 0x049d... → class 0x07b8 │
│            │                            │
│            │ Storage Changes:           │
│            │ ▸ 0x049d... (5 slots)      │
│            │   0x01: 0x00 → 0x1234      │
│            │                            │
│            │ Nonces:                    │
│            │ • 0x049d...: 41 → 42       │
└────────────┴────────────────────────────┘
```

**Success**: Can view state diff, search works across types.

---

### Phase 6: Complex Queries (SQLite Index)
**Commit**: `phase-6: sqlite index and complex queries`

**Goal**: Enable queries RocksDB can't handle efficiently.

**Deliverables**:
- [ ] SQLite schema for transactions, contracts
- [ ] Background indexer (sync from RocksDB)
- [ ] `GET /api/transactions?status=reverted`
- [ ] `GET /api/transactions?sender=0x...`
- [ ] `GET /api/contracts?class_hash=0x...`
- [ ] Index status indicator

**New crate**: `crates/indexer/`

**UI**:
```
┌────────────┬─────────────────────────────┐
│ Sidebar    │ Advanced Filters            │
│            │                             │
│ • Blocks   │ Status: [Failed Only ☑]    │
│ • Txns     │ Sender: [0x...          ]  │
│ • State    │ Block:  [0   ] to [100  ]  │
│ • Advanced◄│                             │
│            │ Results (23 failed txs):    │
│ Index: ✓   │ #100 | 0x1a2b | "Out of gas"|
│ 100/100    │ #98  | 0x3c4d | "Assert"    │
└────────────┴─────────────────────────────┘
```

**Success**: Can query failed txs, filter by sender, index syncs.

---

### Phase 7: Polish & Export
**Commit**: `phase-7: polish, export, responsive design`

**Goal**: Production-ready polish.

**Deliverables**:
- [ ] Loading states / skeletons
- [ ] Error handling / boundaries
- [ ] Export to JSON
- [ ] Responsive design
- [ ] Dark mode toggle
- [ ] Shareable URLs

**Success**: No crashes, works on mobile, can export data.

---

## Commit History (Expected)

```
phase-0: project scaffold with hello world
phase-1: rocksdb connection and stats endpoint
phase-2: block list and detail views
phase-3: transaction list and details
phase-4: contract state and class browser
phase-5: state diff viewer and search
phase-6: sqlite index and complex queries
phase-7: polish, export, responsive design
```

Each phase may have intermediate commits:
```
phase-2: block list and detail views
phase-2: fix pagination bug
phase-2: improve block detail layout
```

---

## Self-Feedback Checklist

After each phase, verify:

### Visual Check
- [ ] Does the layout match the mockup?
- [ ] Are fonts/colors consistent?
- [ ] Is spacing appropriate?

### Functional Check
- [ ] Do all links/buttons work?
- [ ] Does pagination work?
- [ ] Do API calls succeed?

### Data Check
- [ ] Is data from actual DB?
- [ ] Are values formatted correctly?
- [ ] Do hashes display properly?

### Edge Cases
- [ ] Empty states handled?
- [ ] Errors displayed gracefully?
- [ ] Loading states shown?

---

## Dependencies on Madara

The visualizer imports types from `mc-db`:

```toml
[dependencies]
mc-db = { path = "../madara/crates/client/db" }
mp-block = { path = "../madara/crates/primitives/block" }
mp-state-update = { path = "../madara/crates/primitives/state_update" }
# etc.
```

This ensures serialization compatibility without duplicating types.

---

## Quick Reference

```bash
# Start backend
cargo run -p api -- --db-path /tmp/madara_devnet_poc_v2/db

# Start frontend
trunk serve

# Visual check
agent-browser open http://localhost:8080
agent-browser snapshot -i
agent-browser screenshot /tmp/check.png

# Commit checkpoint
git add -A && git commit -m "phase-N: description"
```

---

Ready to begin with Phase 0!
