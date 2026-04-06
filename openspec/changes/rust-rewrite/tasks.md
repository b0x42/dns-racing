## 1. Project Scaffold

- [x] 1.1 Create `Cargo.toml` with dependencies: tokio, hickory-resolver, clap (derive+env), chrono, rand, dotenvy. Configure release profile with `lto = true`, `strip = true`
- [x] 1.2 Create module structure: `src/main.rs`, `src/config.rs`, `src/query.rs`, `src/stats.rs`, `src/display.rs`, `src/csv.rs`
- [x] 1.3 Update `.gitignore` to include `/target`; commit `Cargo.lock` (binary project)
- [x] 1.4 Delete Node.js files: `dns-racing.js`, `package.json`, `package-lock.json`; remove `node_modules/` from `.gitignore`

## 2. CLI Config (`src/config.rs`)

- [x] 2.1 Define clap `Args` struct with all config parameters, env var fallback, and defaults per cli-config spec
- [x] 2.2 Implement `.env` file loading via `dotenvy::dotenv().ok()` before clap parsing
- [x] 2.3 Implement extra-dns parsing (comma-separated `ip:label` pairs, IP as label when omitted)
- [x] 2.4 Add validation: reject duplicate custom/public-dns IPs, reject non-positive RPS, reject invalid IPs

## 3. DNS Resolver Setup (`src/query.rs`)

- [x] 3.1 Create `build_resolver(ip, timeout)` function that returns `Arc<TokioResolver>` with `NameServerConfig` pointing at the given IP, `attempts: 1`
- [x] 3.2 Build server registry: `Vec<Server>` where each `Server` holds label, IP, color, and `Arc<TokioResolver>`

## 4. Query Engine (`src/query.rs`)

- [x] 4.1 Implement `query(resolver, domain)` async function that times `resolver.lookup_ip()` with `Instant` and returns `(Duration, Status)` where Status is `Ok`/`Nxdomain`/`Error`
- [x] 4.2 Implement domain list with Fisher-Yates shuffle at startup
- [x] 4.3 Implement tick loop: `tokio::time::interval(1s / rps)` cycling domains round-robin, spawning parallel queries into a `JoinSet` per tick, draining results into main-task-owned stats and sending to CSV channel

## 5. Stats Engine (`src/stats.rs`)

- [x] 5.1 Implement `ServerStats` with `VecDeque<QueryResult>` bounded by window size
- [x] 5.2 Implement `compute_stats()` returning ok count, cache hits, blocked, errors, min, avg, p95, p99, max
- [x] 5.3 Implement per-domain average tracking: `HashMap<(domain, server_ip), (sum, count)>`

## 6. CSV Export (`src/csv.rs`)

- [x] 6.1 Create CSV file with timestamped filename and write header at startup
- [x] 6.2 Spawn CSV writer task receiving results via `tokio::sync::mpsc` channel
- [x] 6.3 Write per-query rows (timestamp, server, domain, latency_ms, status) after warmup only
- [x] 6.4 Flush and close CSV on shutdown (drop sender to signal writer task)

## 7. Warmup (`src/query.rs`)

- [x] 7.1 Implement warmup loop: N rounds × all domains × all servers, with progress output
- [x] 7.2 Ensure warmup queries are not recorded in stats or CSV
- [x] 7.3 Detect unreachable resolvers (100% errors during warmup) and exit with error

## 8. Live Display (`src/display.rs`)

- [x] 8.1 Implement startup banner (server list, rate, window, output file)
- [x] 8.2 Implement live stats table with manual box-drawing, ANSI cursor movement, and colored server labels (OK, cache, blocked, err, min, avg, p95, p99, max)
- [x] 8.3 Implement non-TTY fallback: sequential append-only stats output
- [x] 8.4 Implement per-domain breakdown table on shutdown
- [x] 8.5 Implement race verdict table on shutdown (ranked by avg, with diff)

## 9. Graceful Shutdown

- [x] 9.1 Handle Ctrl+C via `tokio::signal::ctrl_c()` (cross-platform) and SIGTERM (Unix only)
- [x] 9.2 Handle ESC key via raw stdin on Unix TTY only; skip on Windows and non-TTY
- [x] 9.3 On shutdown: stop tick loop, print final stats, domain breakdown, verdict, flush CSV, exit

## 10. Documentation

- [x] 10.1 Update README: quick start with binary download / `cargo install` / build-from-source, updated config table with CLI flags, updated output examples with p99 column
- [x] 10.2 Update `.env.example` with comments showing equivalent CLI flags

## 11. Distribution

- [x] 11.1 Run `cargo dist init` to generate GitHub Actions release workflow targeting Linux (x86_64, aarch64 musl), macOS (x86_64, aarch64), Windows (x86_64)
