# DNS Racing

[![Node.js](https://img.shields.io/badge/Node.js-16.4+-green.svg)](https://nodejs.org/)
[![dotenv](https://img.shields.io/badge/config-dotenv-yellow.svg)](https://github.com/motdotla/dotenv)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

Ever wondered how your DNS server compares to cloud DNS? This tool fires parallel lookups at your custom server (AdGuard, Pi-hole, Unbound, etc.) and public resolvers simultaneously, showing live latency stats so you can see exactly how they stack up.

- Compare your avg response time to one or more cloud DNS servers (Cloudflare, Google, Quad9, etc.)
- Smart measurement — warms up caches before recording, separates blocked domains from errors, and flags cache hits so results are always meaningful
- Instant results with a detailed breakdown per domain
- Auto CSV logging so you can analyse results with any tool you like

## Quick Start

```bash
git clone https://github.com/b0x42/dns-racing.git && cd dns-racing
npm install     # requires Node.js >= 16.4
cp .env.example .env   # edit CUSTOM_DNS to your server's IP
npm start
```

Press `Ctrl+C` to stop and flush final stats, breakdown, verdict, and CSV.

## Configuration

| Key | Default | Description |
|---|---|---|
| `CUSTOM_DNS` | `192.168.0.5` | Your DNS server IP |
| `CUSTOM_DNS_LABEL` | `My DNS` | Display name (e.g. `AdGuard`, `Pi-hole`) |
| `CLOUDFLARE` | `1.1.1.1` | Primary public resolver |
| `EXTRA_DNS` | _(empty)_ | Additional resolvers, e.g. `8.8.8.8:Google,9.9.9.9:Quad9` |
| `RPS` | `25` | Queries per second per server |
| `STATS_EVERY` | `5000` | ms between live stat prints |
| `TIMEOUT` | `5000` | Query timeout in ms |
| `WINDOW` | `500` | Rolling window size (results per server) |
| `WARMUP_ROUNDS` | `2` | Domain passes before recording starts |
| `CACHE_HIT_MS` | `1.0` | Threshold in ms for cache hit detection |

## Output

```
  Warming up cache (60 queries per server)... done
```

```
Stats after 10s
┌──────────────┬────────┬────────┬─────────┬───────┬───────────┬───────────┬───────────┬───────────┐
│       Server │     OK │  Cache │ Blocked │   Err │       Min │       Avg │       p95 │       Max │
├──────────────┼────────┼────────┼─────────┼───────┼───────────┼───────────┼───────────┼───────────┤
│       My DNS │    232 │     18 │       9 │     0 │     0.3ms │     3.4ms │     8.1ms │    22.3ms │
│   Cloudflare │    250 │      0 │       0 │     0 │     8.5ms │    12.1ms │    18.4ms │    35.6ms │
└──────────────┴────────┴────────┴─────────┴───────┴───────────┴───────────┴───────────┴───────────┘
```

```
Per-domain breakdown (My DNS vs Cloudflare)
┌────────────────────┬───────────┬───────────┬────────────┐
│             Domain │    My DNS │ Cloudflare │       Diff │
├────────────────────┼───────────┼───────────┼────────────┤
│         github.com │     1.1ms │    14.3ms │     +13.2ms│
│      wikipedia.org │     2.4ms │    11.8ms │      +9.4ms│
│         amazon.com │     9.8ms │    10.2ms │      +0.4ms│
│        nytimes.com │    13.1ms │     9.7ms │      -3.4ms│
└────────────────────┴───────────┴───────────┴────────────┘
```

```
Race Results
  My DNS  avg 3.4ms  p95 8.1ms  min 0.3ms
  vs Cloudflare: My DNS wins  3.4ms vs 12.1ms  (71.9% faster)
```

CSV: `dns_racing_<timestamp>.csv`

```
timestamp,server,domain,latency_ms,status
2026-03-15T15:12:28.000Z,192.168.0.5,google.com,2.31,ok
2026-03-15T15:12:28.001Z,192.168.0.5,doubleclick.net,1.10,nxdomain
```

## License

MIT — see [LICENSE](LICENSE).
