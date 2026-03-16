#!/usr/bin/env node
// DNS Racing — custom DNS vs public resolvers
// Requires: Node.js >= 16.4

require('dotenv').config();
const dns = require('node:dns/promises');
const fs = require('node:fs');
const { performance } = require('node:perf_hooks');

// ── Config ────────────────────────────────────────────────────────────────────
// Returns the numeric env var if set and valid, otherwise the default.
// Uses NaN check instead of || so that explicitly setting a value to 0 is respected.
const envNum = (key, def) => { const v = Number(process.env[key]); return isNaN(v) ? def : v; };

const rps = envNum('RPS', 25);
if (rps <= 0) { console.error('Error: RPS must be a positive number. Check your .env file.'); process.exit(1); }

const CONFIG = {
  CUSTOM_DNS:       process.env.CUSTOM_DNS       ?? '192.168.0.5', // ← your DNS server IP
  CUSTOM_DNS_LABEL: process.env.CUSTOM_DNS_LABEL ?? 'My DNS',     // display name for your DNS server
  CLOUDFLARE:    process.env.CLOUDFLARE ?? '1.1.1.1',
  EXTRA_DNS:     process.env.EXTRA_DNS  ?? '',            // e.g. "8.8.8.8:Google,9.9.9.9:Quad9"
  RPS:           rps,                                      // requests per second per server
  STATS_EVERY:   envNum('STATS_EVERY', 5000), // ms between live stat prints
  TIMEOUT:       envNum('TIMEOUT',     5000), // DNS query timeout in ms
  OUTPUT:        `dns_racing_${new Date().toISOString().slice(0, 19).replace(/[:.]/g, '-')}.csv`,
  WINDOW:        envNum('WINDOW',        500), // rolling window size — keeps memory bounded for long runs
  WARMUP_ROUNDS: envNum('WARMUP_ROUNDS',   2), // full passes through DOMAINS before recording starts
  CACHE_HIT_MS:  envNum('CACHE_HIT_MS', 1.0), // responses faster than this are flagged as cache hits
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
const GREEN = '\x1b[32m', CYAN = '\x1b[36m', YELLOW = '\x1b[33m', MAGENTA = '\x1b[35m', BLUE = '\x1b[34m';
const CLEAR_LINE = '\x1b[2K';
let tableLineCount = 0; // tracks how many lines the live table occupies so we can overwrite it
let shuttingDown  = false;

// Colors cycled through for extra public resolvers
const EXTRA_COLORS = [MAGENTA, BLUE, YELLOW];

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

// Extracts stats from a rolling window of results for one server
function computeStats(results) {
  const okMs = [];
  let blocked = 0, errors = 0, cacheHits = 0;
  for (const r of results) {
    if (r.ok)           { okMs.push(r.ms); if (r.cacheHit) cacheHits++; }
    else if (r.blocked) { blocked++; }
    else                { errors++; }
  }
  if (!okMs.length) return null;
  const sorted = [...okMs].sort((a, b) => a - b);
  const avg    = okMs.reduce((a, b) => a + b, 0) / okMs.length;
  return { ok: okMs.length, blocked, errors, cacheHits, min: sorted[0], avg, p95: percentile(sorted, 95), max: sorted.at(-1) };
}

function printStats(store, elapsed) {
  const fmtMs = v => `${v.toFixed(1)}ms`;
  const cols  = { server: 12, ok: 6, cache: 6, blocked: 7, err: 5, min: 9, avg: 9, p95: 9, max: 9 };
  const hr    = (l, m, r) => l + Object.values(cols).map(w => '─'.repeat(w + 2)).join(m) + r;
  const cell  = (v, w) => ` ${String(v).padStart(w)} `;

  const lines = [];
  lines.push('');
  lines.push(`${BOLD}Stats after ${(elapsed / 1000).toFixed(0)}s${RESET}`);
  lines.push(hr('┌', '┬', '┐'));
  lines.push(
    '│' + cell('Server',  cols.server)  +
    '│' + cell('OK',      cols.ok)      +
    '│' + cell('Cache',   cols.cache)   +
    '│' + cell('Blocked', cols.blocked) +
    '│' + cell('Err',     cols.err)     +
    '│' + cell('Min',     cols.min)     +
    '│' + cell('Avg',     cols.avg)     +
    '│' + cell('p95',     cols.p95)     +
    '│' + cell('Max',     cols.max)     + '│'
  );
  lines.push(hr('├', '┼', '┤'));

  for (const [ip, { label, color }] of Object.entries(SERVERS)) {
    const s = computeStats(store[ip]);
    if (!s) {
      lines.push('│' + ` ${color}${BOLD}${label.padEnd(cols.server)}${RESET}` + ' │' + ' (no results yet)');
      continue;
    }
    lines.push(
      '│' + ` ${color}${BOLD}${label.padEnd(cols.server)}${RESET} ` +
      '│' + cell(s.ok,           cols.ok)      +
      '│' + cell(s.cacheHits,    cols.cache)   +
      '│' + cell(s.blocked,      cols.blocked)  +
      '│' + cell(s.errors,       cols.err)      +
      '│' + cell(fmtMs(s.min),   cols.min)      +
      '│' + cell(fmtMs(s.avg),   cols.avg)      +
      '│' + cell(fmtMs(s.p95),   cols.p95)      +
      '│' + cell(fmtMs(s.max),   cols.max)      + '│'
    );
  }
  lines.push(hr('└', '┴', '┘'));

  // Move cursor up to overwrite the previous table on all updates after the first
  if (tableLineCount > 0) process.stdout.write(`\x1b[${tableLineCount}A`);
  for (const line of lines) process.stdout.write(`\r${CLEAR_LINE}${line}\n`);
  tableLineCount = lines.length;
}

// Ranks all servers by avg latency and prints a podium table
function printVerdict(store) {
  const fmtMs = v => `${v.toFixed(1)}ms`;
  const ranks = ['1st', '2nd', '3rd', '4th', '5th', '6th', '7th', '8th'];

  // Build ranked list of all servers with stats
  const ranked = Object.keys(SERVERS).map(ip => {
    const s = computeStats(store[ip]);
    if (!s) return null;
    const { label, color } = SERVERS[ip];
    return { label, color, avg: s.avg, p95: s.p95, min: s.min };
  }).filter(Boolean).sort((a, b) => a.avg - b.avg);

  if (!ranked.length) return;

  const cols = { rank: 4, server: 12, avg: 9, p95: 9, min: 9, diff: 10 };
  const hr   = (l, m, r) => l + Object.values(cols).map(w => '─'.repeat(w + 2)).join(m) + r;
  const cell = (v, w) => ` ${String(v).padStart(w)} `;

  console.log(`\n${BOLD}Race Results${RESET}`);
  console.log(hr('┌', '┬', '┐'));
  console.log(
    '│' + cell('Rank', cols.rank) +
    '│' + cell('Server', cols.server) +
    '│' + cell('Avg', cols.avg) +
    '│' + cell('p95', cols.p95) +
    '│' + cell('Min', cols.min) +
    '│' + cell('Diff', cols.diff) + '│'
  );
  console.log(hr('├', '┼', '┤'));

  const fastest = ranked[0].avg;
  for (let i = 0; i < ranked.length; i++) {
    const { label, color, avg, p95, min } = ranked[i];
    const diff = avg - fastest;
    const diffStr = i === 0 ? '—' : `+${fmtMs(diff)}`;
    console.log(
      '│' + cell(ranks[i] || `${i + 1}th`, cols.rank) +
      '│' + ` ${color}${BOLD}${label.padEnd(cols.server)}${RESET} ` +
      '│' + cell(fmtMs(avg), cols.avg) +
      '│' + cell(fmtMs(p95), cols.p95) +
      '│' + cell(fmtMs(min), cols.min) +
      '│' + cell(diffStr, cols.diff) + '│'
    );
  }
  console.log(hr('└', '┴', '┘'));
}

// Prints per-domain avg latency for custom DNS vs the first public resolver,
// sorted by the biggest win/loss for your custom server.
function printDomainBreakdown() {
  const [customIp, publicIp] = Object.keys(SERVERS);
  const { label: customLabel, color: customColor } = SERVERS[customIp];
  const { label: publicLabel, color: publicColor  } = SERVERS[publicIp];

  const rows = DOMAINS.map(domain => {
    const c = domainStore[domain]?.[customIp];
    const p = domainStore[domain]?.[publicIp];
    if (!c || !p) return null;
    const customAvg = c.sum / c.count;
    const publicAvg = p.sum / p.count;
    return { domain, customAvg, publicAvg, diff: publicAvg - customAvg };
  }).filter(Boolean).sort((a, b) => b.diff - a.diff); // biggest custom-DNS wins first

  if (!rows.length) return;

  const fmtMs  = v => `${v.toFixed(1)}ms`;
  const dCol   = 18, msCol = 9, diffCol = 10;
  const hr     = (l, m, r) => `${l}${'─'.repeat(dCol + 2)}${m}${'─'.repeat(msCol + 2)}${m}${'─'.repeat(msCol + 2)}${m}${'─'.repeat(diffCol + 2)}${r}`;
  const cell   = (v, w) => ` ${String(v).padStart(w)} `;

  console.log(`\n${BOLD}Per-domain breakdown${RESET} (${customColor}${customLabel}${RESET} vs ${publicColor}${publicLabel}${RESET})`);
  console.log(hr('┌', '┬', '┐'));
  console.log(`│${cell('Domain', dCol)}│${cell(customLabel, msCol)}│${cell(publicLabel, msCol)}│${cell('Diff', diffCol)}│`);
  console.log(hr('├', '┼', '┤'));

  for (const { domain, customAvg, publicAvg, diff } of rows) {
    const diffStr  = (diff >= 0 ? '+' : '') + diff.toFixed(1) + 'ms';
    const diffColor = diff > 0.5 ? customColor : diff < -0.5 ? publicColor : YELLOW;
    console.log(
      `│ ${domain.padEnd(dCol)} ` +
      `│${cell(fmtMs(customAvg), msCol)}` +
      `│${cell(fmtMs(publicAvg), msCol)}` +
      `│ ${diffColor}${diffStr.padStart(diffCol)}${RESET} │`
    );
  }
  console.log(hr('└', '┴', '┘'));
}

// ── Setup ─────────────────────────────────────────────────────────────────────
let SERVERS;
try {
  SERVERS = {
    [CONFIG.CUSTOM_DNS]: { label: CONFIG.CUSTOM_DNS_LABEL, color: CYAN,  resolver: createResolver(CONFIG.CUSTOM_DNS) },
    [CONFIG.CLOUDFLARE]: { label: 'Cloudflare', color: GREEN, resolver: createResolver(CONFIG.CLOUDFLARE) },
  };

  // Parse extra public resolvers from EXTRA_DNS="8.8.8.8:Google,9.9.9.9:Quad9"
  CONFIG.EXTRA_DNS.split(',').filter(Boolean).forEach((entry, i) => {
    const [ip, label = ip] = entry.trim().split(':');
    SERVERS[ip] = { label, color: EXTRA_COLORS[i % EXTRA_COLORS.length], resolver: createResolver(ip) };
  });
} catch (err) {
  console.error(`Error: Could not create DNS resolver — ${err.message}. Check CUSTOM_DNS, CLOUDFLARE, and EXTRA_DNS in your .env file.`);
  process.exit(1);
}

if (CONFIG.CUSTOM_DNS === CONFIG.CLOUDFLARE) {
  console.error('Error: CUSTOM_DNS and CLOUDFLARE are the same IP. Set them to different servers in your .env file.');
  process.exit(1);
}

const serverIps = Object.keys(SERVERS);
const store       = Object.fromEntries(serverIps.map(ip => [ip, []])); // rolling window per server
const domainStore = {}; // per-domain avg tracking: { domain: { [ip]: { sum, count } } }
const csv         = fs.createWriteStream(CONFIG.OUTPUT); // stream stays open for the run's duration
csv.write('timestamp,server,domain,latency_ms,status\n'); // CSV header

// ── Query ─────────────────────────────────────────────────────────────────────
// record=false during warmup — queries run but results are not stored
async function query(ip, domain, record = true) {
  const { resolver } = SERVERS[ip];
  const t0 = performance.now(); // start high-res timer before the query
  let status = 'ok';

  try {
    await resolver.resolve4(domain); // A-record lookup; result is discarded — we only care about latency
  } catch (err) {
    // distinguish "domain doesn't exist" from a real resolver error
    status = err.code === 'ENOTFOUND' ? 'nxdomain' : 'error';
  }

  if (!record) return; // warmup query — don't store or write

  const ms      = performance.now() - t0; // wall-clock latency in milliseconds
  const blocked  = status === 'nxdomain';        // NXDOMAIN from custom DNS = blocked domain (ad/tracker)
  const ok       = status === 'ok';
  const cacheHit = ok && ms < CONFIG.CACHE_HIT_MS; // sub-1ms response = served from resolver's cache

  // keep the rolling window bounded
  if (store[ip].length >= CONFIG.WINDOW) store[ip].shift();
  store[ip].push({ ms, ok, blocked, cacheHit });

  // accumulate per-domain stats for the breakdown table on exit
  if (ok) {
    if (!domainStore[domain]) domainStore[domain] = {};
    if (!domainStore[domain][ip]) domainStore[domain][ip] = { sum: 0, count: 0 };
    domainStore[domain][ip].sum   += ms;
    domainStore[domain][ip].count += 1;
  }

  if (!shuttingDown) csv.write(`${new Date().toISOString()},${ip},${domain},${ms.toFixed(2)},${status}\n`); // append row to CSV
}

// ── Warmup ────────────────────────────────────────────────────────────────────
// Fire CONFIG.WARMUP_ROUNDS full passes through DOMAINS at all servers so that
// all resolvers have a warm cache before any results are recorded.
async function warmup() {
  const total = DOMAINS.length * CONFIG.WARMUP_ROUNDS;
  process.stdout.write(`  Warming up cache (${total} queries per server)...`);
  for (let round = 0; round < CONFIG.WARMUP_ROUNDS; round++) {
    for (const domain of DOMAINS) {
      await Promise.all(serverIps.map(ip => query(ip, domain, false)));
    }
  }
  process.stdout.write(` ${GREEN}done${RESET}\n\n`);
}

// ── Main ──────────────────────────────────────────────────────────────────────
console.log(`\n${BOLD}DNS Racing${RESET}`);
for (const [ip, { label, color }] of Object.entries(SERVERS)) {
  console.log(`  ${color}${label.padEnd(12)}${RESET} ${ip}`);
}
console.log(`  ${'Rate'.padEnd(12)} ${CONFIG.RPS} req/s per server  →  ${CONFIG.RPS * serverIps.length} total`);
console.log(`  ${'Window'.padEnd(12)} last ${CONFIG.WINDOW} results per server`);
console.log(`  ${'Output'.padEnd(12)} ${CONFIG.OUTPUT}`);
console.log(`  Press ${BOLD}Ctrl+C${RESET} to stop\n`);

warmup().catch(err => {
  console.error(`\n${RESET}Error: Warmup failed — ${err.message}. Check that your DNS servers are reachable.`);
  process.exit(1);
}).then(() => {
  let domainIdx = 0;
  const startTime = Date.now();
  let lastStats   = startTime;

  // fire all servers in parallel on each tick; interval derived from target RPS
  const ticker = setInterval(() => {
    const domain = DOMAINS[domainIdx++ % DOMAINS.length]; // cycle through domains round-robin
    serverIps.forEach(ip => query(ip, domain)); // fire all queries without awaiting — intentionally parallel

    const now = Date.now();
    if (now - lastStats >= CONFIG.STATS_EVERY) { // print live stats on the configured interval
      printStats(store, now - startTime);
      lastStats = now;
    }
  }, 1000 / CONFIG.RPS);

  // graceful shutdown on Ctrl+C or kill — print final stats, verdict, and flush CSV
  function shutdown() {
    if (shuttingDown) return;
    shuttingDown = true;
    clearInterval(ticker);
    printStats(store, Date.now() - startTime);
    tableLineCount = 0; // prevent further overwrites — breakdown and verdict print below the final table
    printDomainBreakdown();
    printVerdict(store);
    csv.end(() => { // wait for the write stream to flush before exiting
      console.log(`\n${GREEN}Results saved → ${CONFIG.OUTPUT}${RESET}\n`);
      process.exit(0);
    });
  }

  process.on('SIGINT',  shutdown);
  process.on('SIGTERM', shutdown); // handle kill / docker stop / systemd stop
});
