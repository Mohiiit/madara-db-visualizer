# API

Makimono serves the API under `/api/*`.

If you run the standalone API server (`madara-db-visualizer-api`), the API is served at the server root.

## Endpoints

### Health / Stats

- `GET /api/health`
- `GET /api/stats`

### Block Explorer

- `GET /api/blocks?offset=0&limit=20`
- `GET /api/blocks/:number`
- `GET /api/blocks/:number/transactions`
- `GET /api/blocks/:number/state-diff`
- `GET /api/contracts/:address`
- `GET /api/classes/:hash`
- `GET /api/search?q=<query>`

### Schema Documentation

- `GET /api/schema/categories`
- `GET /api/schema/column-families`
- `GET /api/schema/column-families/:name`

### Raw Data Inspection

- `GET /api/raw/cf`
- `GET /api/raw/cf/:name/stats`
- `GET /api/raw/cf/:name/keys?limit=50&offset=0`
- `GET /api/raw/cf/:name/key/:key_hex`
- `POST /api/raw/cf/:name/keys/batch`

### SQL Index

- `GET /api/index/status`
- `GET /api/index/tables`
- `GET /api/index/tables/:name/schema`
- `POST /api/index/query`

Example:
```bash
curl -X POST http://127.0.0.1:8080/api/index/query \
  -H "Content-Type: application/json" \
  -d '{"sql": "SELECT * FROM blocks ORDER BY block_number DESC LIMIT 5", "params": []}'
```
