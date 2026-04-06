## Context

DNS Racing is a single-file Node.js CLI tool (~500 LOC in `dns-racing.js`) that benchmarks DNS resolvers by firing parallel lookups and displaying live latency stats. It uses `dotenv` for config, Node's built-in `dns/promises` for queries, and ANSI escape codes for terminal output. The tool targets homelab users running Pi-hole, AdGuard, or Unbound who want to compare their resolver against public DNS.

The Node.js runtime introduces GC pauses and event loop contention that contaminate sub-millisecond latency measurements. Distribution requires users to have Node.js installed and run `npm install`.

## Goals / Non-Goals

**Goals:**
- Deterministic latency measurement using monotonic `Instant` timing with no GC interference
- Single static binary distribution for Linux, macOS, and Windows via GitHub Releases
- Feature parity with the Node.js version: live stats table, domain breakdown, verdict, CSV export
- Enhanced stats: add p99 percentile
- CLI flags with env var fallback and `.env` file support via `clap` + `dotenv`

**Non-Goals:**
- TUI framework (ratatui) — keep the same ANSI escape approach for simplicity
- DNS-over-HTTPS/TLS support (plain UDP/TCP only, matching current behavior)
- GUI or web interface
- Backward compatibility with Node.js config format (clean break)

## Decisions

**1. Async runtime: `tokio`**
Tokio is the standard async runtime for Rust network I/O. It provides multi-core task scheduling, async UDP/TCP sockets, timers, and signal handling — all needed here. Alternative: `async-std` — less ecosystem support, no meaningful advantage for this use case.

**2. DNS library: `hickory-resolver`**
Provides a full async DNS client with protocol handling, retries, and TCP fallback. Alternative: raw `UdpSocket` with `dns-parser` — more control but requires hand-crafting DNS wire format, handling retries, and EDNS. Not worth the complexity for A-record lookups.

**3. Resolver sharing: `Arc<TokioResolver>` per server**
One resolver instance per DNS server, wrapped in `Arc` and cloned into spawned tasks. This avoids the `'static` lifetime issue with hickory-resolver's futures while sharing the internal connection pool. Alternative: create a new resolver per query — wasteful, loses connection reuse.

**4. Config: `clap` derive with `env` feature**
CLI flags with automatic env var fallback. Combined with `dotenv` crate to load `.env` files. This gives users three config layers: CLI flags > env vars > `.env` file > defaults. Alternative: dotenv-only (current approach) — loses CLI ergonomics.

**5. Stats: in-memory rolling window with manual percentile calculation**
Keep the current rolling window approach (`VecDeque` bounded by `--window` size). Sort on demand for percentile extraction. Alternative: `hdrhistogram` crate — overkill for window sizes of 500-1000, adds dependency for minimal benefit.

**6. Output: manual ANSI escape codes and box-drawing to stdout**
Match the current Node.js approach: cursor movement + line clearing for live table overwrites. Manual box-drawing with precise column widths and inline ANSI color codes per cell. Alternative: `comfy-table` — fights you on inline colors, padding, and exact alignment; manual formatting is ~50 lines and gives full control. Alternative: `ratatui` — full TUI framework is unnecessary for a table that refreshes every few seconds.

**7. Project structure: modular from the start**
Split into `main.rs`, `config.rs`, `query.rs`, `stats.rs`, `display.rs`, `csv.rs`. The implementation will exceed 800 LOC and clean module boundaries make each file focused. Alternative: single `main.rs` — would work but becomes unwieldy at this scale.

**8. CSV write concurrency: mpsc channel**
Query tasks send results to a single CSV writer task via `tokio::sync::mpsc`. This avoids mutex contention on the file handle, preserves write ordering, and lets the writer buffer/flush efficiently. The channel also carries stats updates to avoid shared mutable state. Alternative: `Mutex<BufWriter<File>>` — simpler but adds contention under high RPS.

**9. Cross-compilation: `cargo-dist` with GitHub Actions**
`cargo-dist` generates release workflows that produce binaries for all target platforms. Alternative: manual `cross` setup — more maintenance, `cargo-dist` handles the matrix automatically.

## Risks / Trade-offs

- **[hickory-resolver overhead]** → The resolver has internal caching and retry logic that adds slight overhead vs raw UDP. Mitigation: set `attempts: 1` to avoid retries that would skew timing. Leave `cache_size` at default (32) — hickory's internal cache rarely triggers since we cycle 30 domains round-robin, and fighting undocumented `cache_size: 0` behavior isn't worth the risk.
- **[Measurement includes resolver client overhead]** → `Instant` timing wraps the full `resolver.lookup_ip()` call, not just network RTT. Mitigation: this is acceptable — we're measuring "how fast does the resolver respond to a real query from a real client," which is what users care about. The overhead is consistent and sub-microsecond.
- **[Breaking change for existing users]** → Node.js users must switch to the binary. Mitigation: clear migration notes in README, keep `.env` variable names identical where possible.
- **[Binary size]** → Rust binary with tokio + hickory will be ~5-10MB. Mitigation: LTO + strip in release profile. Acceptable for the distribution model.
