## ADDED Requirements

### Requirement: Live stats table with in-place overwrite
The system SHALL print a stats table at the configured `--stats-every` interval using ANSI escape codes to overwrite the previous table in-place. The table SHALL show per-server: OK count, cache hits, blocked, errors, min, avg, p95, p99, and max.

#### Scenario: Periodic stats refresh
- **WHEN** the stats interval elapses and stdout is a TTY
- **THEN** the system moves the cursor up and overwrites the previous table with updated stats

#### Scenario: First stats print
- **WHEN** stats are printed for the first time
- **THEN** the table is printed without cursor movement (no previous table to overwrite)

#### Scenario: Non-TTY output
- **WHEN** stdout is not a TTY (piped or redirected)
- **THEN** the system prints stats sequentially without ANSI cursor movement (append-only)

### Requirement: Startup banner
The system SHALL print a banner at startup showing each configured server (label + IP), the query rate, window size, and output file path.

#### Scenario: Banner display
- **WHEN** the program starts after config parsing
- **THEN** the banner is printed with server labels colored using ANSI codes

### Requirement: Race verdict on shutdown
The system SHALL print a final ranking table on shutdown, sorted by average latency, showing rank, server, avg, p95, min, and diff from fastest.

#### Scenario: Verdict with multiple servers
- **WHEN** the program shuts down with results from 2+ servers
- **THEN** servers are ranked by avg latency with the fastest showing "—" as diff

### Requirement: Per-domain breakdown on shutdown
The system SHALL print a per-domain table on shutdown showing average latency per server for each domain, sorted by the difference (biggest custom DNS advantage first).

#### Scenario: Domain breakdown display
- **WHEN** the program shuts down with per-domain data
- **THEN** a table shows each domain's avg latency per server and the diff column

### Requirement: Graceful shutdown on signal
The system SHALL handle Ctrl+C (SIGINT) via `tokio::signal::ctrl_c()` (cross-platform) and SIGTERM (Unix only) to trigger graceful shutdown: stop queries, print final stats, domain breakdown, verdict, flush CSV, then exit. On Unix with a TTY stdin, the system SHALL also detect the ESC key via raw mode.

#### Scenario: Ctrl+C shutdown
- **WHEN** the user presses Ctrl+C
- **THEN** the system stops the query ticker, prints final output, and exits cleanly

#### Scenario: ESC key shutdown (Unix TTY only)
- **WHEN** the user presses ESC and stdin is a TTY on a Unix system
- **THEN** the system triggers the same graceful shutdown sequence

#### Scenario: Non-TTY stdin
- **WHEN** stdin is not a TTY (piped or non-interactive)
- **THEN** ESC key detection is skipped; only Ctrl+C / SIGTERM trigger shutdown
