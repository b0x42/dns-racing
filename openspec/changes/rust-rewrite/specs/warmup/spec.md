## ADDED Requirements

### Requirement: Cache warmup before recording
The system SHALL execute `--warmup-rounds` full passes through all domains against all resolvers before recording begins. Warmup queries SHALL NOT be recorded in stats or CSV.

#### Scenario: Warmup execution
- **WHEN** the program starts after config parsing
- **THEN** it runs the configured number of warmup rounds, querying all domains on all resolvers

#### Scenario: Warmup progress indication
- **WHEN** warmup is in progress
- **THEN** the system prints a progress message showing total warmup queries per server

#### Scenario: Warmup completion
- **WHEN** all warmup rounds complete
- **THEN** the system prints "done" and begins the recording phase

### Requirement: Unreachable resolver detection during warmup
The system SHALL exit with an error if any resolver produces 100% errors during warmup, indicating the server is unreachable.

#### Scenario: Unreachable resolver
- **WHEN** all warmup queries for a resolver fail with errors
- **THEN** the system exits with an error message suggesting the user check the server IP
