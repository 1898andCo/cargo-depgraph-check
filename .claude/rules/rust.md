<!-- Implementation rules for SOUL.md principles. SOUL owns "why", this file owns "how". -->

# Rust Coding Rules

Conventions for cargo-depgraph-check.

## Safety

- `#![forbid(unsafe_code)]` in `main.rs`
- No `unwrap()` in production code — use `?` or `expect()` with actionable message
- No async runtime — this is a synchronous CLI tool

## Type Design

- **Newtypes where they prevent bugs:** `CrateName(String)` if it prevents mixing crate names with other strings — apply pragmatically (SOUL #1), not dogmatically
- **Validated constructors at trust boundaries:** config parsing validates crate names, metadata extraction validates package IDs
- **`#[non_exhaustive]` on enums that will grow** — `ViolationKind`, `OutputFormat`
- **Private fields with getters** on types where invariants matter

## Error Handling

- Use `thiserror` for error enums — `ConfigError`, `MetadataError`, `ValidationError`
- Define `pub type Result<T> = std::result::Result<T, Error>` in `error.rs`
- `Display` impl produces user-facing messages directly (CLI tool, no API sanitization layer)

## Module Structure

```
src/
  main.rs        # CLI entry, clap, exit code
  lib.rs         # Re-exports only
  error.rs       # Error types
  config.rs      # Config parsing
  metadata.rs    # cargo_metadata wrapper
  validate.rs    # Validation logic
  report.rs      # Output formatting
```

`lib.rs` is a pure re-export barrel — implementation in domain modules.

## Dependencies

- Edition 2024, MSRV 1.85+
- Key crates: `cargo_metadata` (workspace graph), `clap` (CLI), `toml`/`serde` (config), `thiserror` (errors)
- Use `cargo clippy -- -D warnings` — warnings are errors

## Testing

- Unit: `#[cfg(test)] mod tests {}` in same file
- Integration: `tests/` directory, run binary against fixture workspaces
- Snapshot: `insta` crate for output regression
- Test names as documentation: `validate_catches_forbidden_dep()`, not `test_1()`
- Test boundaries: empty config, missing crate, stale config entry, zero deps, self-dep
