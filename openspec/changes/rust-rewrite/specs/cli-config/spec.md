## ADDED Requirements

### Requirement: CLI flags with env var fallback
The system SHALL accept configuration via CLI flags, with automatic fallback to environment variables, then `.env` file values, then defaults. CLI flags SHALL take precedence over env vars.

#### Scenario: CLI flag overrides env var
- **WHEN** `--rps 50` is passed and `RPS=25` is in the environment
- **THEN** the system uses 50 as the RPS value

#### Scenario: Env var used when no flag
- **WHEN** no `--rps` flag is passed and `RPS=25` is in the environment
- **THEN** the system uses 25 as the RPS value

#### Scenario: Default used when neither set
- **WHEN** no `--rps` flag and no `RPS` env var is set
- **THEN** the system uses the default value of 25

### Requirement: Configuration parameters
The system SHALL support these parameters with defaults:
- `--custom-dns` / `CUSTOM_DNS` (default: `192.168.0.5`)
- `--custom-dns-label` / `CUSTOM_DNS_LABEL` (default: `My DNS`)
- `--public-dns` / `CLOUDFLARE` (default: `1.1.1.1`) — primary public resolver (env var kept as `CLOUDFLARE` for backward compat)
- `--extra-dns` / `EXTRA_DNS` (default: empty)
- `--rps` / `RPS` (default: 25)
- `--stats-every` / `STATS_EVERY` (default: 5000ms)
- `--timeout` / `TIMEOUT` (default: 5000ms)
- `--window` / `WINDOW` (default: 500)
- `--warmup-rounds` / `WARMUP_ROUNDS` (default: 2)
- `--cache-hit-ms` / `CACHE_HIT_MS` (default: 1.0)

#### Scenario: All parameters have defaults
- **WHEN** the program is run with no flags and no env vars
- **THEN** all parameters use their documented default values

### Requirement: Extra DNS parsing
The system SHALL parse `--extra-dns` as a comma-separated list of `ip:label` pairs (e.g., `8.8.8.8:Google,9.9.9.9:Quad9`). When the label is omitted (e.g., just `8.8.8.8`), the IP address SHALL be used as the display label.

#### Scenario: Multiple extra resolvers
- **WHEN** `--extra-dns "8.8.8.8:Google,9.9.9.9:Quad9"` is provided
- **THEN** two additional resolvers are configured with the given IPs and labels

#### Scenario: Label omitted
- **WHEN** `--extra-dns "8.8.8.8"` is provided without a `:label` suffix
- **THEN** the resolver is configured with `8.8.8.8` as both the IP and the display label

### Requirement: Validation
The system SHALL exit with an error if `--custom-dns` and `--public-dns` are the same IP, if `--rps` is not positive, or if any DNS IP (custom, public, or extra) is not a valid IPv4/IPv6 address.

#### Scenario: Duplicate IP rejection
- **WHEN** custom DNS and public DNS IPs are identical
- **THEN** the system exits with an error message

#### Scenario: Invalid RPS
- **WHEN** `--rps 0` or a negative value is provided
- **THEN** the system exits with an error message

#### Scenario: Invalid IP address
- **WHEN** a DNS IP cannot be parsed as a valid IPv4 or IPv6 address
- **THEN** the system exits with an error message identifying the invalid IP
