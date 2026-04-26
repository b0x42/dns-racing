<p align="center">
  <img src="icon.jpg" width="256" alt="DNS Racing icon">
  <br>
  <strong><h1 align="center">DNS Racing</h1></strong>
</p>

<p align="center">
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-1.70+-orange.svg" alt="Rust"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-green.svg" alt="License: MIT"></a>
</p>

Ever wondered how your DNS server compares to cloud DNS? This tool fires parallel lookups at your custom server (AdGuard, Pi-hole, Unbound, etc.) and public resolvers simultaneously, showing live latency stats so you can see exactly how they stack up.

- 🏁 **Race your DNS** — Compare avg response time to one or more cloud resolvers (Cloudflare, Google, Quad9, etc.)
- 🎯 **Deterministic measurement** — Rust binary with monotonic timing, no GC pauses, no event loop noise
- 🧠 **Smart measurement** — Warms up caches before recording, separates blocked domains from errors, and flags cache hits
- 📊 **Instant results** — Detailed breakdown per domain with live latency stats
- 📁 **Auto CSV logging** — Analyse results with any tool you like
- 📦 **Single static binary** — No runtime dependencies

## Quick Start

### Download a release binary

```bash
# macOS / Linux
curl -LO https://github.com/b0x42/dns-racing/releases/latest/download/dns-racing
chmod +x dns-racing
./dns-racing --custom-dns 192.168.0.5
```

### Install via cargo

```bash
cargo install --git https://github.com/b0x42/dns-racing.git
```

### Build from source

```bash
git clone https://github.com/b0x42/dns-racing.git && cd dns-racing
cargo build --release
cp target/release/dns-racing .
```

Press `Ctrl+C` or `ESC` to stop and flush final stats, breakdown, verdict, and CSV.

## Usage

Race your Pi-hole against Cloudflare (default):

```bash
dns-racing --custom-dns 192.168.1.53 --custom-dns-label Pi-hole
```

Race AdGuard against Cloudflare, Google, and Quad9 at once:

```bash
dns-racing --custom-dns 192.168.1.53 --custom-dns-label AdGuard \
  --extra-dns "8.8.8.8:Google,9.9.9.9:Quad9"
```

High-frequency test with a larger rolling window and faster stats:

```bash
dns-racing --custom-dns 192.168.1.53 --rps 50 --window 1000 --stats-every 2000
```

## Configuration

All options can be set via CLI flags, environment variables, or a `.env` file. CLI flags take precedence over env vars.

| Flag | Env Var | Default | Description |
|---|---|---|---|
| `--custom-dns` | `CUSTOM_DNS` | `192.168.0.5` | Your DNS server IP |
| `--custom-dns-label` | `CUSTOM_DNS_LABEL` | `My DNS` | Display name (e.g. `AdGuard`, `Pi-hole`) |
| `--public-dns` | `CLOUDFLARE` | `1.1.1.1` | Primary public resolver |
| `--extra-dns` | `EXTRA_DNS` | _(empty)_ | Additional resolvers, e.g. `8.8.8.8:Google,9.9.9.9:Quad9` |
| `--rps` | `RPS` | `25` | Queries per second per server |
| `--stats-every` | `STATS_EVERY` | `1000` | ms between live stat prints |
| `--timeout` | `TIMEOUT` | `5000` | Query timeout in ms |
| `--window` | `WINDOW` | `500` | Rolling window size (results per server) |
| `--warmup-rounds` | `WARMUP_ROUNDS` | `2` | Domain passes before recording starts |
| `--cache-hit-ms` | `CACHE_HIT_MS` | `1.0` | Threshold in ms for cache hit detection |

You can also use a `.env` file:

```bash
cp .env.example .env   # edit to your liking
./dns-racing
```

## Output

```
Stats after 10s
┌──────────────┬─────┬─────────┬─────┬────────┬────────┬────────┬────────┬────────┐
│       Server │  OK │ Blocked │ Err │    Min │    Avg │    p95 │    p99 │    Max │
├──────────────┼─────┼─────────┼─────┼────────┼────────┼────────┼────────┼────────┤
│       My DNS │ 232 │       9 │   0 │  3.9ms │  6.1ms │  8.6ms │ 12.4ms │ 22.3ms │
│   Cloudflare │ 250 │       0 │   0 │  6.2ms │  8.3ms │ 11.7ms │ 16.4ms │ 19.9ms │
│       Google │ 250 │       0 │   0 │ 15.0ms │ 19.6ms │ 23.9ms │ 28.8ms │ 38.5ms │
│        Quad9 │ 250 │       0 │   0 │  5.8ms │  7.9ms │ 10.4ms │ 16.2ms │ 32.1ms │
└──────────────┴─────┴─────────┴─────┴────────┴────────┴────────┴────────┴────────┘
  Stop the race with ESC or Ctrl+C
```

```
Per-domain breakdown (My DNS vs Cloudflare vs Google vs Quad9)
┌───────────────┬──────────┬────────────┬─────────┬─────────┬─────────┐
│ Domain        │   My DNS │ Cloudflare │  Google │   Quad9 │    Diff │
├───────────────┼──────────┼────────────┼─────────┼─────────┼─────────┤
│ github.com    │    6.0ms │      8.5ms │  21.6ms │   8.1ms │ +15.6ms │
│ wikipedia.org │    6.1ms │      7.8ms │  20.4ms │   7.6ms │ +14.3ms │
│ amazon.com    │    6.3ms │      8.0ms │  18.5ms │   7.7ms │ +12.2ms │
│ nytimes.com   │    6.0ms │      9.1ms │  19.1ms │   7.5ms │ +13.1ms │
└───────────────┴──────────┴────────────┴─────────┴─────────┴─────────┘
```

```
Race Results
┌──────┬────────────┬────────┬────────┬────────┬─────────┐
│ Rank │     Server │    Avg │    p95 │    Min │    Diff │
├──────┼────────────┼────────┼────────┼────────┼─────────┤
│  1st │     My DNS │  6.1ms │  8.6ms │  3.9ms │       — │
│  2nd │      Quad9 │  7.9ms │ 10.4ms │  5.8ms │  +1.8ms │
│  3rd │ Cloudflare │  8.3ms │ 11.7ms │  6.2ms │  +2.2ms │
│  4th │     Google │ 19.6ms │ 23.9ms │ 15.0ms │ +13.5ms │
└──────┴────────────┴────────┴────────┴────────┴─────────┘
```

CSV: `dns_racing_<timestamp>.csv`

```
timestamp,server,domain,latency_ms,status
2026-03-15T15:12:28.000Z,192.168.0.5,google.com,2.31,ok
2026-03-15T15:12:28.001Z,192.168.0.5,doubleclick.net,1.10,nxdomain
```

## License

MIT — see [LICENSE](LICENSE).
