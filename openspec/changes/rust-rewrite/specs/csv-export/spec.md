## ADDED Requirements

### Requirement: CSV file creation with header
The system SHALL create a CSV file named `dns_racing_<timestamp>.csv` at startup with the header: `timestamp,server,domain,latency_ms,status`.

#### Scenario: File creation
- **WHEN** the program starts
- **THEN** a CSV file is created with the header row

### Requirement: Per-query CSV row
The system SHALL append a row to the CSV for each completed query (after warmup) with ISO 8601 timestamp, server IP, domain, latency in ms (2 decimal places), and status (`ok`, `nxdomain`, or `error`). Query tasks SHALL send results via a `tokio::sync::mpsc` channel to a single CSV writer task to avoid concurrent file access.

#### Scenario: OK query logged
- **WHEN** a successful query completes during recording
- **THEN** a CSV row is written with status `ok` and the measured latency

#### Scenario: Warmup queries not logged
- **WHEN** a query completes during the warmup phase
- **THEN** no CSV row is written

#### Scenario: Concurrent writes
- **WHEN** multiple query tasks complete simultaneously
- **THEN** results are sent through the mpsc channel and written sequentially by the writer task

### Requirement: CSV flush on shutdown
The system SHALL flush and close the CSV file during graceful shutdown before exiting.

#### Scenario: Clean file close
- **WHEN** the program shuts down
- **THEN** all buffered CSV data is flushed to disk and the file is closed
