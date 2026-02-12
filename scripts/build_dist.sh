#!/usr/bin/env bash
set -euo pipefail

# Deterministic asset build for makimono-viz embedding.
# Requires: node/npm, rust toolchain with wasm32 target, trunk, python3.

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

if ! command -v npm >/dev/null 2>&1; then
  echo "error: npm not found" >&2
  exit 1
fi
if ! command -v python3 >/dev/null 2>&1; then
  echo "error: python3 not found" >&2
  exit 1
fi

npm ci
npx tailwindcss -i ./input.css -o ./output.css

# Install trunk if missing (pin for reproducibility)
TRUNK_VERSION="0.20.3"
if ! command -v trunk >/dev/null 2>&1; then
  cargo install trunk --locked --version "$TRUNK_VERSION"
fi

rustup target add wasm32-unknown-unknown >/dev/null 2>&1 || true

trunk build --release
python3 ./scripts/patch_wasm_table.py ./dist

echo "dist ready at ./dist"
