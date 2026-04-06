## 1. Project Scaffold

- [ ] 1.1 Create `Cargo.toml` with dependencies: tokio, hickory-resolver, clap (derive+env), chrono, rand, dotenvy. Configure release profile with `lto = true`, `strip = true`
- [ ] 1.2 Create module structure: `src/main.rs`, `src/config.rs`, `src/query.rs`, `src/stats.rs`, `src/display.rs`, `src/csv.rs`
- [ ] 1.3 Update `.gitignore` to include `/target`; commit `Cargo.lock` (binary project)
- [ ] 1.4 Delete Node.js files: `dns-racing.js`, `package.json`, `package-lock.json`; remove `node_modules/` from `.gitignore`

## 2. CLI Config (`src/config.rs`)

- [ ] 2.1 Define clap `Args` struct with all config parameters, env var fallback, and defaults per cli-config spec
- [ ] 2.2 Implement `.env` file loading via `dotenvy::dotenv().ok()` before clap parsing
- [ ] 2.3 Implement extra-dns parsing (comma-separated `ip:label` pairs, IP as label when omitted)
- [ ] 2.4 Add validation: reject duplicate custom/cloudflare IPs, reject non-positive RPS
- [ ] 2.5 Update `.env.example` with comments showing equivalent CLI flags

## 3. DNS Resolver Setup (`src/query.rs`)

- [ ] 3.1 Create `build_resolver(ip, timeout)` function that returns `Arc<TokioResolver>` with `NameServerConfig` pointing at the given IP, `attempts: 1`
- [ ] 3.2 Build server registry: `Vec<Server>` where each `Server` holds label, IP, color, and `Arc<TokioResolver>`

## 4. Query Engine (`src/query.rs`)

- [ ] 4.1 Implement `query(resolver, domain)` async function that times `resolver.lookup_ip()` with `Instant` and returns `(Duration, Status)` where Status is `Ok`/`Nxdomain`/`Error`
- [ ] 4.2 Implement domain list with Fisher-Yates shuffle at startup
- [ ] 4.3 Implement tick loop: `tokio::time::interval(1s / rps)` cycling domains round-robin, spawning parallel queries into a `JoinSet` per tick, draining results into main-task-owned stats and sending to CSV channel

## 5. Stats Engine (`src/stats.rs`)

- [ ] 5.1 Implement `ServerStats` with `VecDeque<QueryResult>` bounded by window size
- [ ] 5.2 Implement `compute_stats()` returning ok count, cache hits, blocked, errors, min, avg, p95, p99, max
- [ ] 5.3 Implement per-domain average tracking: `HashMap<(domain, server_ip), (sum, count)>`

## 6. CSV Export (`src/csv.rs`)

- [ ] 6.1 Create CSV file with timestamped filename and write header at startup
- [ ] 6.2 Spawn CSV writer task receiving results via `tokio::sync::mpsc` channel
- [ ] 6.3 Write per-query rows (timestamp, server, domain, latency_ms, status) after warmup only
- [ ] 6.4 Flush and close CSV on shutdown (drop sender to signal writer task)

## 7. Warmup (`src/query.rs`)

- [ ] 7.1 Implement warmup loop: N rounds × all domains × all servers, with progress output
- [ ] 7.2 Ensure warmup queries are not recorded in stats or CSV
- [ ] 7.3 Detect unreachable resolvers (100% errors during warmup) and exit with error

## 8. Live Display (`src/display.rs`)

- [ ] 8.1 Implement startup banner (server list, rate, window, output file)
- [ ] 8.2 Implement live stats table with manual box-drawing, ANSI cursor movement, and colored server labels (OK, cache, blocked, err, min, avg, p95, p99, max)
- [ ] 8.3 Implement non-TTY fallback: sequential append-only stats output
- [ ] 8.4 Implement per-domain breakdown table on shutdown
- [ ] 8.5 Implement race verdict table on shutdown (ranked by avg, with diff)

## 9. Graceful Shutdown

- [ ] 9.1 Handle Ctrl+C via `tokio::signal::ctrl_c()` (cross-platform) and SIGTERM (Unix only)
- [ ] 9.2 Handle ESC key via raw stdin on Unix TTY only; skip on Windows and non-TTY
- [ ] 9.3 On shutdown: stop tick loop, print final stats, domain breakdown, verdict, flush CSV, exit

## 10. Distribution

- [ ] 10.1 Run `cargo dist init` to generate GitHub Actions release workflow targeting Linux (x86_64, aarch64 musl), macOS (x86_64, aarch64), Windows (x86_64)
- [ ] 10.2 Update README for Rust: installation from releases, `cargo install`, and build-from-source instructions
