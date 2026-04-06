## Why

DNS Racing measures sub-millisecond DNS resolver latency, but the Node.js runtime introduces non-deterministic noise from GC pauses and event loop contention that contaminates results. Rewriting in Rust eliminates measurement interference and enables distribution as a single static binary — no runtime dependency for the pi-hole/homelab audience.

## What Changes

- **BREAKING**: Replace the entire Node.js implementation (`dns-racing.js`, `package.json`, `node_modules`) with a Rust binary
- Add `clap`-based CLI with flags and env var fallback (replaces dotenv-only config)
- Replace Node.js `dns/promises` with `hickory-resolver` for async DNS lookups
- Add p99 percentile alongside existing p95 in stats output
- Use `Instant`-based monotonic timing instead of `performance.now()` for deterministic latency measurement
- Ship cross-compiled binaries via GitHub Actions + `cargo-dist` for Linux, macOS, and Windows
- Retain identical ANSI escape live table output, domain breakdown, verdict, and CSV logging

## Capabilities

### New Capabilities
- `dns-query`: Async DNS query execution against configurable resolvers with per-query monotonic timing
- `stats-engine`: Rolling window statistics with p50/p95/p99 percentiles, cache hit detection, and blocked domain tracking
- `live-display`: ANSI escape terminal output with live stats table, domain breakdown, and race verdict
- `csv-export`: Timestamped CSV logging of all query results
- `cli-config`: CLI argument parsing with env var fallback and `.env` file support
- `warmup`: Multi-round cache warming before measurement recording begins
- `cross-platform-dist`: GitHub Actions workflow for cross-compiled release binaries

### Modified Capabilities

_(none — no existing specs)_

## Impact

- **Code**: Entire codebase replaced — `dns-racing.js` → `src/main.rs` (+ modules)
- **Dependencies**: Node.js + dotenv → Rust toolchain with tokio, hickory-resolver, clap, chrono, rand
- **Distribution**: `npm install` + Node.js runtime → single static binary via GitHub Releases
- **Config**: `.env` file still supported, but CLI flags take precedence; `.env.example` updated for new format
- **Output**: Identical table format and CSV schema; adds p99 column to stats table
