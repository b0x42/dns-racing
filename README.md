# DNS Latency Tracker

[![Node.js](https://img.shields.io/badge/Node.js-16.4+-green.svg)](https://nodejs.org/)
[![dotenv](https://img.shields.io/badge/config-dotenv-yellow.svg)](https://github.com/motdotla/dotenv)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

Benchmarks your local AdGuard DNS server against Cloudflare (`1.1.1.1`) in real time. Fires parallel A-record lookups at both servers, prints live stats every 5 seconds, and saves every result to a timestamped CSV.

- Live min / avg / p95 / max latency per server
- Error classification: `ok`, `nxdomain`, `error`
- Round-robin across 30 real-world domains
- Timestamped CSV output for offline analysis
- Zero dependencies — uses only Node.js built-ins

## Quick Start

```bash
npm install
cp .env.example .env   # edit CUSTOM_DNS to your AdGuard IP
node dns-tracker.js
```

Press `Ctrl+C` to stop. Final stats are printed and the CSV is flushed on exit.

## Configuration

Set values in your `.env` file (copy from `.env.example`):

| Key | Default | Description |
|---|---|---|
| `CUSTOM_DNS` | `10.0.1.15` | Your AdGuard server IP |
| `CLOUDFLARE` | `1.1.1.1` | Cloudflare resolver |
| `RPS` | `25` | Queries per second per server |
| `STATS_EVERY` | `5000` | ms between live stat prints |
| `TIMEOUT` | `5000` | DNS query timeout in ms |

## Output

Live stats print to the terminal on the configured interval:

```
── Stats after 10s ───────────────────────────────────────
  AdGuard     ok=  250  err=  0  min=    1.2ms  avg=    3.4ms  p95=    8.1ms  max=   22.3ms
  Cloudflare  ok=  250  err=  0  min=    8.5ms  avg=   12.1ms  p95=   18.4ms  max=   35.6ms
```

A CSV file named `dns_latency_<timestamp>.csv` is written to the current directory:

```
timestamp,server,domain,latency_ms,status
2026-03-15T15:12:28.000Z,10.0.1.15,google.com,2.31,ok
```

## Prerequisites

- Node.js >= 16.4
- `npm install` (installs dotenv)

## License

MIT License - see [LICENSE](LICENSE) for details.
