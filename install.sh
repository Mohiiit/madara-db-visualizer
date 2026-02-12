#!/usr/bin/env bash
set -euo pipefail

REPO="${MAKIMONO_REPO:-Mohiiit/makimono}"
TAG="${MAKIMONO_BOOTSTRAPPER_TAG:-makimono}"
PREFIX="${MAKIMONO_INSTALL_PREFIX:-$HOME/.local/bin}"

mkdir -p "$PREFIX"

os=""
case "$(uname -s)" in
  Darwin) os="macos";;
  Linux) os="linux";;
  *) echo "unsupported OS" >&2; exit 1;;
 esac

arch=""
case "$(uname -m)" in
  arm64|aarch64) arch="arm64";;
  x86_64|amd64) arch="x64";;
  *) echo "unsupported arch" >&2; exit 1;;
 esac

asset="makimono-${TAG}-${os}-${arch}.tar.gz"
base="https://github.com/${REPO}/releases/download/${TAG}"
url="${base}/${asset}"
sums_url="${base}/SHA256SUMS"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

curl -fsSL "$sums_url" -o "$work/SHA256SUMS"
expected="$(awk -v a="$asset" '$2==a{print $1}' "$work/SHA256SUMS")"
if [[ -z "${expected}" ]]; then
  echo "SHA256SUMS missing entry for ${asset}" >&2
  exit 1
fi

curl -fsSL "$url" -o "$work/${asset}"
if command -v sha256sum >/dev/null 2>&1; then
  actual="$(sha256sum "$work/${asset}" | awk '{print $1}')"
elif command -v shasum >/dev/null 2>&1; then
  actual="$(shasum -a 256 "$work/${asset}" | awk '{print $1}')"
elif command -v openssl >/dev/null 2>&1; then
  actual="$(openssl dgst -sha256 "$work/${asset}" | awk '{print $2}')"
else
  echo "need sha256sum, shasum, or openssl for checksum verification" >&2
  exit 1
fi
if [[ "$actual" != "$expected" ]]; then
  echo "checksum mismatch for ${asset}" >&2
  exit 1
fi

tar -xzf "$work/${asset}" -C "$work"
if [[ ! -f "$work/makimono" ]]; then
  echo "archive did not contain makimono" >&2
  exit 1
fi

install -m 0755 "$work/makimono" "$PREFIX/makimono"

echo "installed: $PREFIX/makimono"
