#!/usr/bin/env python3
"""
Patch wasm externref table limits to avoid runtime failures like:
  WebAssembly.Table.grow(): failed to grow table by 4

We do a minimal in-place edit of the Table section (id=4):
- For each table of type externref (0x6f) that has an explicit maximum equal to its initial,
  rewrite the maximum to the largest value that fits in the same LEB128 byte width.

This keeps section sizes unchanged (no re-encoding of the module required).

Usage:
  scripts/patch_wasm_table.py /path/to/file.wasm
  scripts/patch_wasm_table.py /path/to/dir   # patches all *_bg.wasm under it
"""

from __future__ import annotations

import argparse
import os
import sys
from pathlib import Path


def _read_varuint(data: bytes, off: int) -> tuple[int, int]:
    """Return (value, nbytes)."""
    res = 0
    shift = 0
    n = 0
    while True:
        if off + n >= len(data):
            raise ValueError("unexpected EOF while reading varuint")
        b = data[off + n]
        n += 1
        res |= (b & 0x7F) << shift
        if (b & 0x80) == 0:
            return res, n
        shift += 7
        if shift > 35:
            raise ValueError("varuint too large")


def _encode_varuint_fixed_width(value: int, width: int) -> bytes:
    """LEB128 encode into exactly `width` bytes (assumes value fits)."""
    out = bytearray()
    for i in range(width):
        byte = value & 0x7F
        value >>= 7
        if i != width - 1:
            byte |= 0x80
        out.append(byte)
    if value != 0:
        raise ValueError("value does not fit in fixed width")
    return bytes(out)


def patch_wasm_file(path: Path) -> bool:
    data = path.read_bytes()
    if len(data) < 8 or data[:4] != b"\x00asm":
        raise ValueError(f"not a wasm module: {path}")

    # Skip magic + version
    off = 8
    patched = False
    externref_table_index: int | None = None

    while off < len(data):
        section_id = data[off]
        off += 1
        section_size, n = _read_varuint(data, off)
        off += n
        payload_off = off
        payload_end = payload_off + section_size
        if payload_end > len(data):
            raise ValueError(f"invalid section size in {path}")

        if section_id not in (4, 7):
            off = payload_end
            continue

        mutable = bytearray(data)

        if section_id == 4:
            # Table section payload:
            # vec(table)
            p = payload_off
            table_count, n = _read_varuint(data, p)
            p += n

            for table_idx in range(table_count):
                if p >= payload_end:
                    raise ValueError("table section truncated")
                elem_type = data[p]
                p += 1

                flags, n = _read_varuint(data, p)
                p += n
                initial, n = _read_varuint(data, p)
                p += n

                has_max = (flags & 0x01) != 0
                if has_max:
                    max_off = p
                    max_val, max_n = _read_varuint(data, p)
                    p += max_n
                else:
                    max_off = None
                    max_val = None
                    max_n = None

                # externref table element type is 0x6f.
                if elem_type == 0x6F and externref_table_index is None:
                    externref_table_index = table_idx

                # Best-effort: ensure an explicit maximum allows wasm-bindgen's initial `grow(4)`.
                if (
                    elem_type == 0x6F
                    and has_max
                    and max_val is not None
                    and max_n is not None
                    and max_off is not None
                    and max_val < initial + 4
                ):
                    # Pick the largest value representable with the same LEB width.
                    new_max = (1 << (7 * max_n)) - 1
                    enc = _encode_varuint_fixed_width(new_max, max_n)
                    mutable[max_off : max_off + max_n] = enc
                    patched = True

        if section_id == 7:
            # Export section payload:
            # vec(export) where export = name + kind + index
            if externref_table_index is None:
                # Can't fix exports without knowing which table is externref.
                off = payload_end
                continue

            p = payload_off
            export_count, n = _read_varuint(data, p)
            p += n

            for _ in range(export_count):
                name_len, n = _read_varuint(data, p)
                p += n
                name = data[p : p + name_len].decode("utf-8", "replace")
                p += name_len
                kind = data[p]
                p += 1

                idx_off = p
                idx_val, idx_n = _read_varuint(data, p)
                p += idx_n

                # We want `__wbindgen_externrefs` to point to the externref table.
                # Some builds end up exporting the funcref table under that name,
                # which causes runtime failures in browsers.
                if name == "__wbindgen_externrefs" and kind == 1 and idx_val != externref_table_index:
                    # Only patch in-place when the varuint width stays the same.
                    enc = _encode_varuint_fixed_width(externref_table_index, idx_n)
                    mutable[idx_off : idx_off + idx_n] = enc
                    patched = True

        if patched:
            data = bytes(mutable)
            path.write_bytes(data)

        off = payload_end

    return patched


def iter_wasm_candidates(p: Path):
    if p.is_file():
        yield p
        return
    for root, _dirs, files in os.walk(p):
        for f in files:
            if f.endswith("_bg.wasm"):
                yield Path(root) / f


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("path", type=Path)
    args = ap.parse_args()

    any_patched = False
    for wasm in iter_wasm_candidates(args.path):
        try:
            did = patch_wasm_file(wasm)
            if did:
                print(f"patched: {wasm}")
                any_patched = True
        except Exception as e:
            print(f"error: {wasm}: {e}", file=sys.stderr)
            return 2

    if not any_patched:
        print("no changes needed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
