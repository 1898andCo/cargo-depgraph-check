# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

@SOUL.md
@.claude/rules/_index.md

## What This Project Is

`cargo-depgraph-check` is a cargo subcommand that enforces workspace crate dependency graph rules.
It reads an allowlist-based TOML config and validates the resolved dependency graph from `cargo metadata`.
Published to crates.io. Any Cargo workspace can use it.

This is a single-crate binary — no workspace, no sub-crates.

## Build & Test

```bash
cargo build                        # Build the binary
cargo test                         # Run all tests (needs fixture workspaces)
cargo test -- test_name            # Specific test
cargo clippy --all-targets -- -D warnings  # Lint (warnings are errors)
cargo +nightly fmt --all           # Format (requires nightly)
cargo run -- check                 # Run against current workspace (if in one)
cargo run -- check --manifest-path /path/to/Cargo.toml --config /path/to/rules.toml
cargo run -- generate              # Generate baseline config from current workspace
```

Rust 1.85+, Edition 2024. Nightly rustfmt required.

## Git Workflow

**Git Flow** — aligned with axiathon.

- Branch from `develop`, PRs target `develop`, never commit directly to `main`
- Branch naming: `feature/short-desc`, `fix/issue-123-desc`
- Conventional commits: `type(scope): description`
- No AI attribution in commit messages

## Module Structure

```
src/
  main.rs        # CLI entry point (clap derive, thin wrapper)
  lib.rs         # Public API re-exports
  config.rs      # TOML config parsing + validation
  metadata.rs    # cargo_metadata wrapper, workspace graph extraction
  validate.rs    # Allowlist validation engine
  report.rs      # Human-readable + JSON error formatting
  error.rs       # Crate-level error types (thiserror)
```

`main.rs` is a thin wrapper — all logic lives in library modules.
`lib.rs` is a pure re-export barrel.

## Testing

- **Unit tests**: `#[cfg(test)] mod tests {}` in each module
- **Integration tests**: `tests/integration_test.rs` — run binary against fixture workspaces
- **Snapshot tests**: `insta` crate for error output regression
- **Fixtures**: `tests/fixtures/` — minimal Cargo workspaces with known dep structures
- Test names as documentation: `config_rejects_empty_crate_name()`, not `test_1()`

## Error Handling

- `thiserror` for all error enums
- `pub type Result<T> = std::result::Result<T, Error>` in `error.rs`
- No `unwrap()` in production code — use `?` or `expect()` with actionable message
- Exit codes: 0 = pass, 1 = violations found, 2 = tool error

## CLI Conventions

- Binary name: `cargo-depgraph-check` (invoked as `cargo depgraph-check`)
- Subcommands: `check` (default), `generate`
- Global flags: `--manifest-path`, `--config`, `--format` (text/json), `--color` (auto/always/never)
- Respect `NO_COLOR` env var
- Pair colors with symbols in output (SOUL.md #9)

## Dependencies

- `cargo_metadata` 0.19+ — workspace dependency graph
- `clap` 4 (derive) — CLI argument parsing
- `toml` 0.8+ — config file parsing
- `serde` 1 — deserialization
- `thiserror` 2 — error types
- `insta` (dev) — snapshot testing
- No async runtime needed — this is a synchronous CLI tool
