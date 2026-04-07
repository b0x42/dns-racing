## ADDED Requirements

### Requirement: Rolling window per server
The system SHALL maintain a bounded `VecDeque` of query results per server, capped at the configured `--window` size. When the window is full, the oldest result SHALL be evicted.

#### Scenario: Window overflow
- **WHEN** a new result arrives and the window is at capacity
- **THEN** the oldest result is removed before the new result is appended

### Requirement: Percentile calculation
The system SHALL compute min, avg, p95, p99, and max from the OK results in each server's rolling window by sorting and indexing.

#### Scenario: Stats computation
- **WHEN** stats are requested for a server with OK results
- **THEN** the system returns min, avg, p95, p99, and max latency values

#### Scenario: No OK results
- **WHEN** stats are requested for a server with zero OK results
- **THEN** the system returns no stats for that server

### Requirement: Cache hit detection
The system SHALL flag any OK response with latency below the configured `--cache-hit-ms` threshold as a cache hit.

#### Scenario: Sub-threshold response
- **WHEN** a successful query completes in less than `--cache-hit-ms`
- **THEN** the result is counted as a cache hit in stats

### Requirement: Blocked domain tracking
The system SHALL count NXDOMAIN responses separately from errors, treating them as blocked domains.

#### Scenario: NXDOMAIN counted as blocked
- **WHEN** a resolver returns NXDOMAIN
- **THEN** the blocked counter increments (not the error counter)

### Requirement: Per-domain average tracking
The system SHALL track cumulative sum and count of OK latencies per (domain, server) pair for the domain breakdown table.

#### Scenario: Domain average accumulation
- **WHEN** an OK result arrives for a domain/server pair
- **THEN** the latency is added to the running sum and count for that pair
