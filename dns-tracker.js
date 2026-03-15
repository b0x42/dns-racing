#!/usr/bin/env node
// DNS Latency Tracker — AdGuard vs. Cloudflare
// Requires: Node.js >= 16.4  |  No npm install needed

require('dotenv').config();
const dns        = require('node:dns/promises');
const fs         = require('node:fs');
const { performance } = require('node:perf_hooks');

// ── Config ────────────────────────────────────────────────────────────────────
const CONFIG = {
  CUSTOM_DNS:    process.env.CUSTOM_DNS  ?? '10.0.1.15',  // ← your AdGuard LXC IP
  CLOUDFLARE:    process.env.CLOUDFLARE  ?? '1.1.1.1',
  RPS:           Number(process.env.RPS)          || 25,   // requests per second per server
  STATS_EVERY:   Number(process.env.STATS_EVERY)  || 5000, // ms between live stat prints
  TIMEOUT:       Number(process.env.TIMEOUT)       || 5000, // DNS query timeout in ms
  OUTPUT:        `dns_latency_${new Date().toISOString().slice(0, 19).replace(/[:.]/g, '-')}.csv`,
};
// ─────────────────────────────────────────────────────────────────────────────

const DOMAINS = [
  'google.com',     'youtube.com',    'facebook.com',   'twitter.com',
  'instagram.com',  'reddit.com',     'github.com',     'stackoverflow.com',
  'amazon.com',     'netflix.com',    'wikipedia.org',  'cloudflare.com',
  'apple.com',      'microsoft.com',  'linkedin.com',   'twitch.tv',
  'discord.com',    'spotify.com',    'tiktok.com',     'whatsapp.com',
  'zoom.us',        'dropbox.com',    'slack.com',      'heise.de',
  'spiegel.de',     'bbc.com',        'nytimes.com',    'reuters.com',
  'theguardian.com','medium.com',
];

// ── Helpers ───────────────────────────────────────────────────────────────────
const RESET = '\x1b[0m', BOLD = '\x1b[1m';
const GREEN = '\x1b[32m', CYAN = '\x1b[36m', YELLOW = '\x1b[33m';

// Creates a DNS resolver pointed at a specific server IP
function createResolver(ip) {
  const r = new dns.Resolver({ timeout: CONFIG.TIMEOUT });
  r.setServers([ip]);
  return r;
}

// Returns the p-th percentile value from a pre-sorted array
function percentile(sorted, p) {
  if (!sorted.length) return 0;
  return sorted[Math.max(0, Math.ceil(sorted.length * p / 100) - 1)];
}

function printStats(store, elapsed) {
  console.log(`\n${BOLD}── Stats after ${(elapsed / 1000).toFixed(0)}s ${'─'.repeat(35)}${RESET}`);
  for (const [ip, { label, color }] of Object.entries(SERVERS)) {
    const ok     = store[ip].filter(r => r.ok).map(r => r.ms);
    const errors = store[ip].length - ok.length;
    if (!ok.length) { console.log(`  ${color}${label}${RESET}  no results yet`); continue; }
    const sorted = [...ok].sort((a, b) => a - b);
    const avg    = ok.reduce((a, b) => a + b, 0) / ok.length;
    console.log(
      `  ${color}${BOLD}${label}${RESET}` +
      `  ok=${String(ok.length).padStart(5)}  err=${String(errors).padStart(3)}` +
      `  min=${sorted[0].toFixed(1).padStart(7)}ms` +
      `  avg=${avg.toFixed(1).padStart(7)}ms` +
      `  p95=${percentile(sorted, 95).toFixed(1).padStart(7)}ms` +
      `  max=${sorted.at(-1).toFixed(1).padStart(7)}ms`
    );
  }
}

// ── Setup ─────────────────────────────────────────────────────────────────────
const SERVERS = {
  [CONFIG.CUSTOM_DNS]: { label: 'AdGuard   ', color: CYAN,  resolver: createResolver(CONFIG.CUSTOM_DNS) },
  [CONFIG.CLOUDFLARE]: { label: 'Cloudflare', color: GREEN, resolver: createResolver(CONFIG.CLOUDFLARE) },
};

const store    = { [CONFIG.CUSTOM_DNS]: [], [CONFIG.CLOUDFLARE]: [] }; // in-memory results per server
const csv      = fs.createWriteStream(CONFIG.OUTPUT); // stream stays open for the run's duration
csv.write('timestamp,server,domain,latency_ms,status\n'); // CSV header

// ── Query ─────────────────────────────────────────────────────────────────────
async function query(ip, domain) {
  const { resolver } = SERVERS[ip];
  const t0 = performance.now(); // start high-res timer before the query
  let status = 'ok';

  try {
    await resolver.resolve4(domain); // A-record lookup; result is discarded — we only care about latency
  } catch (err) {
    // distinguish "domain doesn't exist" from a real resolver error
    status = err.code === 'ENOTFOUND' ? 'nxdomain' : 'error';
  }

  const ms = performance.now() - t0; // wall-clock latency in milliseconds
  store[ip].push({ ms, ok: status === 'ok' }); // accumulate for live stats
  csv.write(`${new Date().toISOString()},${ip},${domain},${ms.toFixed(2)},${status}\n`); // append row to CSV
}

// ── Main loop ─────────────────────────────────────────────────────────────────
let domainIdx  = 0;
const startTime = Date.now();
let lastStats   = startTime;

console.log(`\n${BOLD}DNS Latency Tracker${RESET}`);
console.log(`  AdGuard    ${CYAN}${CONFIG.CUSTOM_DNS}${RESET}`);
console.log(`  Cloudflare ${GREEN}${CONFIG.CLOUDFLARE}${RESET}`);
console.log(`  Rate       ${CONFIG.RPS} req/s per server  →  ${CONFIG.RPS * 2} total`);
console.log(`  Output     ${CONFIG.OUTPUT}`);
console.log(`  Press ${BOLD}Ctrl+C${RESET} to stop\n`);

// setInterval fires every 40ms (= 25/s); both servers are queried in parallel per tick
const ticker = setInterval(() => {
  const domain = DOMAINS[domainIdx++ % DOMAINS.length]; // cycle through domains round-robin
  query(CONFIG.CUSTOM_DNS, domain); // fire both queries without awaiting — intentionally parallel
  query(CONFIG.CLOUDFLARE,  domain);

  const now = Date.now();
  if (now - lastStats >= CONFIG.STATS_EVERY) { // print live stats on the configured interval
    printStats(store, now - startTime);
    lastStats = now;
  }
}, 1000 / CONFIG.RPS); // interval in ms derived from target RPS

// Ctrl+C handler: stop the ticker, print final stats, flush and close the CSV
process.on('SIGINT', () => {
  clearInterval(ticker);
  printStats(store, Date.now() - startTime);
  csv.end(() => { // wait for the write stream to flush before exiting
    console.log(`\n${GREEN}Results saved → ${CONFIG.OUTPUT}${RESET}\n`);
    process.exit(0);
  });
});

