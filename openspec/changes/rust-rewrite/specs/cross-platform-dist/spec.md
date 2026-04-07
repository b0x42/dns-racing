## ADDED Requirements

### Requirement: Release binary via cargo-dist
The system SHALL use `cargo-dist` to generate a GitHub Actions workflow that builds release binaries for Linux (x86_64, aarch64), macOS (x86_64, aarch64), and Windows (x86_64).

#### Scenario: Tagged release triggers build
- **WHEN** a git tag matching `v*` is pushed
- **THEN** GitHub Actions builds and uploads binaries for all target platforms to a GitHub Release

### Requirement: Optimized release profile
The Cargo.toml SHALL configure the release profile with `lto = true` and `strip = true` to minimize binary size.

#### Scenario: Release build size
- **WHEN** the binary is built in release mode
- **THEN** LTO and symbol stripping are applied

### Requirement: Static binary where possible
The system SHALL produce statically-linked binaries on Linux (using musl) so they run without shared library dependencies.

#### Scenario: Linux binary portability
- **WHEN** the Linux binary is copied to a minimal system without glibc
- **THEN** the binary executes without missing library errors
