## ADDED Requirements

### Requirement: Async DNS A-record lookup per resolver
The system SHALL perform async DNS A-record lookups against each configured resolver using `hickory-resolver` with `TokioResolver`. Each resolver SHALL be represented by an `Arc<TokioResolver>` created at startup with `attempts: 1`.

#### Scenario: Successful lookup
- **WHEN** a query is dispatched for a domain against a resolver
- **THEN** the system records the result as `ok` with the elapsed time from `Instant::now()` to response

#### Scenario: NXDOMAIN response
- **WHEN** a resolver returns NXDOMAIN for a domain
- **THEN** the system records the result as `nxdomain` (blocked)

#### Scenario: Query error or timeout
- **WHEN** a resolver fails to respond within the configured timeout
- **THEN** the system records the result as `error`

### Requirement: Parallel query dispatch across all resolvers
The system SHALL fire queries for the same domain to all configured resolvers concurrently using a `tokio::task::JoinSet`. The main task SHALL drain completed results from the `JoinSet` and update stats directly. Domains SHALL cycle round-robin from a shuffled list of 30 hardcoded domains.

#### Scenario: Concurrent resolution
- **WHEN** a tick fires at the configured RPS interval
- **THEN** the main task spawns one query per resolver into a JoinSet for the current domain, all executing in parallel

#### Scenario: Result collection
- **WHEN** spawned queries complete
- **THEN** the main task collects results from the JoinSet, updates stats, and sends results to the CSV writer channel

### Requirement: Monotonic timing per query
The system SHALL measure query latency using `std::time::Instant` (monotonic clock) to avoid wall-clock drift. Timing SHALL wrap only the `resolver.lookup_ip()` call.

#### Scenario: Timing precision
- **WHEN** a query completes
- **THEN** the recorded latency reflects only DNS resolution time with no GC or event loop interference
