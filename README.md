# cargo-depgraph-check

Enforce workspace crate dependency graph rules via allowlist configuration.

[![Crates.io](https://img.shields.io/crates/v/cargo-depgraph-check.svg)](https://crates.io/crates/cargo-depgraph-check)
[![CI](https://github.com/1898andCo/cargo-depgraph-check/actions/workflows/ci.yml/badge.svg)](https://github.com/1898andCo/cargo-depgraph-check/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/cargo-depgraph-check.svg)](LICENSE)

## The Problem

Large Rust workspaces define layered architectures with strict dependency rules:
core crates shouldn't depend on API crates, domain crates shouldn't depend on
binaries, and the dependency graph must stay acyclic. Today these rules live in
architecture docs and are enforced only by code review.

**cargo-depgraph-check** automates this. Define your allowed dependencies in a
TOML config, and the tool validates the actual `cargo metadata` dependency graph
against your rules — in CI, pre-commit hooks, or on demand.

No existing tool does this. `cargo-deny` handles external crate bans.
`cargo-udeps` finds unused dependencies. `cargo-depgraph` visualizes the graph.
None enforce internal workspace dependency boundaries.

## Installation

```bash
# Pre-built binary (fastest)
cargo binstall cargo-depgraph-check

# From source
cargo install cargo-depgraph-check

# From git
cargo install --git https://github.com/1898andCo/cargo-depgraph-check
```

## Quick Start

```bash
# 1. Generate a baseline config from your current workspace
cargo depgraph-check generate > depgraph-rules.toml

# 2. Edit the config to tighten rules (remove deps you want to forbid)
$EDITOR depgraph-rules.toml

# 3. Validate
cargo depgraph-check check
```

## Configuration

Create a `depgraph-rules.toml` in your workspace root:

```toml
[rules]
# Each key is a workspace crate name.
# Value is the list of allowed internal (workspace) dependencies.

# Foundation — no internal dependencies
my-core = []

# Domain crates — explicit allowlists
my-storage = ["my-core"]
my-api = ["my-core", "my-storage"]

# Binary crate — can depend on domain crates
my-server = ["my-core", "my-storage", "my-api"]

[options]
# strict = true: workspace members not in [rules] are errors
# strict = false: workspace members not in [rules] are warnings
strict = true

# Whether to validate dev-dependencies against the allowlist
check_dev_deps = false

# What to do when a config entry names a crate not in the workspace:
# "warn" (default) | "error" | "ignore"
unmatched_config_entries = "warn"
```

### Rules

- Each key under `[rules]` is a workspace crate name
- The value is the **complete list** of allowed internal dependencies (allowlist)
- Any internal dependency not in the list is a violation
- Crates not listed in `[rules]` are flagged based on `strict` mode

### Options

| Option | Default | Description |
|--------|---------|-------------|
| `strict` | `true` | Workspace members not in `[rules]` are errors (`true`) or warnings (`false`) |
| `check_dev_deps` | `false` | Whether to validate dev-dependencies against the allowlist |
| `unmatched_config_entries` | `"warn"` | Behavior when a config entry has no matching workspace member: `"warn"`, `"error"`, or `"ignore"` |

## CLI Reference

### `cargo depgraph-check check`

Validate workspace dependencies against the allowlist config.

```
cargo depgraph-check check [OPTIONS]

Options:
  --manifest-path <PATH>   Path to workspace Cargo.toml [default: auto-detect]
  --config <PATH>          Path to rules config [default: depgraph-rules.toml]
  --format <FORMAT>        Output format: text, json [default: text]
  --color <WHEN>           Color output: auto, always, never [default: auto]
```

### `cargo depgraph-check generate`

Generate a baseline config from the current workspace's dependency graph.

```
cargo depgraph-check generate [OPTIONS]

Options:
  --manifest-path <PATH>   Path to workspace Cargo.toml [default: auto-detect]
  -o, --output <PATH>      Write to file instead of stdout
```

### Exit Codes

| Code | Meaning |
|------|---------|
| `0` | All rules pass |
| `1` | Violations found |
| `2` | Tool error (bad config, metadata failure, no subcommand) |

## Example Output

```
ERROR: my-api depends on my-server, but allowed deps are: [my-core, my-storage]
ERROR: my-storage depends on my-api, but allowed deps are: [my-core]
WARNING: config entry 'my-future-crate' has no matching workspace member

2 violations found across 2 crates
```

## CI Integration

### GitHub Actions

```yaml
check-deps:
  name: Dependency Graph
  runs-on: ubuntu-latest
  timeout-minutes: 5
  steps:
    - uses: actions/checkout@v6
    - uses: dtolnay/rust-toolchain@stable
    - name: Install cargo-depgraph-check
      run: cargo install cargo-depgraph-check --locked
    - name: Check dependency graph rules
      run: cargo depgraph-check check
```

### lefthook (pre-commit)

```yaml
# .lefthook.yml
pre-commit:
  commands:
    depgraph:
      glob: "**/Cargo.toml"
      run: cargo depgraph-check check
```

### justfile

```just
check-deps:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! command -v cargo-depgraph-check &>/dev/null; then
        echo "cargo-depgraph-check not installed. Run: cargo install cargo-depgraph-check"
        exit 1
    fi
    cargo depgraph-check check
```

## How It Works

1. Reads your `depgraph-rules.toml` config
2. Runs `cargo metadata --format-version=1` to get the resolved dependency graph
3. Filters to workspace-internal dependencies only
4. Compares each crate's actual dependencies against its allowlist
5. Reports violations with clear error messages

The tool only checks **direct** internal workspace dependencies. Transitive
dependencies are covered by their own allowlist entries. External dependencies
(from crates.io, git) are ignored — use `cargo-deny` for those.

## License

Licensed under the [MIT License](LICENSE).
