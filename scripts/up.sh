#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  scripts/up.sh [ROCKSDB_PATH] [-- <docker compose up args...>]

Examples:
  scripts/up.sh ~/.madara/db
  scripts/up.sh ./sample-db
  scripts/up.sh ~/.madara/db -- -d
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
compose_file="$repo_root/compose.yaml"

# Load a minimal subset of .env into the script environment (no eval).
# This keeps "single command" UX while matching docker compose defaults.
dotenv_file="$repo_root/.env"
if [[ -f "$dotenv_file" ]]; then
  while IFS= read -r line; do
    line="${line%$'\r'}"
    [[ -z "$line" || "${line:0:1}" == "#" ]] && continue

    # Support "export KEY=VALUE" (common in local env files)
    if [[ "$line" == export\ * ]]; then
      line="${line#export }"
    fi

    case "$line" in
      API_PORT=*|WEB_PORT=*|ROCKSDB_PATH=*)
        key="${line%%=*}"
        val="${line#*=}"
        # Strip surrounding single/double quotes (simple dotenv compat)
        if [[ ${#val} -ge 2 ]]; then
          first="${val:0:1}"
          last="${val: -1}"
          if [[ "$first" == "\"" && "$last" == "\"" ]]; then
            val="${val:1:${#val}-2}"
          elif [[ "$first" == "'" && "$last" == "'" ]]; then
            val="${val:1:${#val}-2}"
          fi
        fi
        if [[ -z "${!key:-}" && -n "$val" ]]; then
          export "${key}=${val}"
        fi
        ;;
    esac
  done < "$dotenv_file"
fi

user_db_path=""
if [[ $# -ge 1 && "${1:-}" != "--" ]]; then
  user_db_path="$1"
  shift
fi

if [[ "${1:-}" == "--" ]]; then
  shift
fi

db_path="${user_db_path:-${ROCKSDB_PATH:-./sample-db}}"

if ! command -v docker >/dev/null 2>&1; then
  echo "error: docker not found in PATH" >&2
  exit 1
fi

if ! docker info >/dev/null 2>&1; then
  echo "error: Docker does not seem to be running. Start Docker Desktop and retry." >&2
  exit 1
fi

compose_bin=()
if docker compose version >/dev/null 2>&1; then
  compose_bin=(docker compose)
elif command -v docker-compose >/dev/null 2>&1; then
  compose_bin=(docker-compose)
else
  echo "error: docker compose is not available (tried 'docker compose' and 'docker-compose')." >&2
  exit 1
fi

abs_db_path=""
db_path_expanded="$db_path"
if [[ "$db_path_expanded" != /* && "$db_path_expanded" != "~"* ]]; then
  # Interpret relative paths relative to the repo root so the script can be run from anywhere.
  db_path_expanded="$repo_root/$db_path_expanded"
fi
if command -v python3 >/dev/null 2>&1; then
  abs_db_path="$(python3 -c 'import os,sys; print(os.path.abspath(os.path.expanduser(sys.argv[1])))' "$db_path_expanded")"
elif command -v realpath >/dev/null 2>&1; then
  abs_db_path="$(realpath "$db_path_expanded")"
else
  # Best-effort: compose can still handle relative paths if we run from repo root.
  abs_db_path="$db_path_expanded"
fi

if [[ ! -d "$abs_db_path" ]]; then
  echo "error: RocksDB path does not exist or is not a directory: $abs_db_path" >&2
  usage >&2
  exit 1
fi

looks_like_rocksdb_dir() {
  local p="$1"
  [[ -f "$p/CURRENT" ]] || ls "$p"/*.sst >/dev/null 2>&1
}

if ! looks_like_rocksdb_dir "$abs_db_path"; then
  # Common mistake: pass Madara base-path (contains `db/`) instead of the RocksDB dir itself.
  if looks_like_rocksdb_dir "$abs_db_path/db"; then
    abs_db_path="$abs_db_path/db"
  else
    echo "error: $abs_db_path doesn't look like a RocksDB directory (no CURRENT file / *.sst)." >&2
    usage >&2
    exit 1
  fi
fi

if [[ ! -f "$compose_file" ]]; then
  echo "error: compose.yaml not found at repo root: $compose_file" >&2
  exit 1
fi

cd "$repo_root"

db_dir_name="$(basename "$abs_db_path")"

echo "Using RocksDB: $abs_db_path"
echo "Starting services (web: http://localhost:${WEB_PORT:-8080}, api: http://localhost:${API_PORT:-3000})"

ROCKSDB_PATH="$abs_db_path" DB_DIR_NAME="$db_dir_name" "${compose_bin[@]}" -f "$compose_file" up --build "$@"
