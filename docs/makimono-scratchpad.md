# Makimono Scratchpad (Maintainer Notes)

This file is a short internal note to avoid footguns when releasing/maintaining Makimono.

## Moving Parts

- `makimono` (bootstrapper): installs toolchains and runs them.
- `makimono-viz` (toolchain): single-port server that embeds the UI and serves `/api/*`.

## Release Tagging

Toolchains are selected by Madara DB schema version:

- Immutable releases: `<db_version>.<patch>.<patch>` (example: `9.0.1`)
- Alias tag: `<db_version>` (example: `9`)

Important: `makimono` downloads toolchains using:

- GitHub release tag: `<tag>`
- Asset name: `makimono-viz-<tag>-<os>-<arch>.(tar.gz|zip)`

So the alias release `9` must contain assets named with `9` in the filename (not `9.0.1`).

## Embedded UI Build

`makimono-viz` embeds the workspace-level `dist/` directory at compile time.

Build `dist/` with:

```bash
./scripts/build_dist.sh
```

Then build the toolchain binary:

```bash
cargo build -p api --release --features embedded-ui --bin makimono-viz
```

