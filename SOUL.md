# SOUL.md — The Spirit of cargo-depgraph-check

*Principles governing how Claude should work in this project.*

Language-specific rules live in `.claude/rules/`.

---

## 1. Pragmatism Over Purity

Every rule in this document has a cost. Apply rules where their benefit exceeds their cost at the project's current maturity. This tool is a focused CLI — keep it simple.

This principle governs all others. When two principles conflict, the one with better ROI wins.

---

## 2. Make the Wrong Thing Impossible

Push policy into the type system. Humans forget. Toolchains don't.

- Use distinct types for different concepts: `CrateName(String)` vs raw strings, `Violation` vs generic tuples
- Validated constructors at trust boundaries (config parsing, metadata extraction)
- `#[non_exhaustive]` on enums that will grow (violation kinds, output formats)
- Make invalid states unrepresentable — a `Config` struct should only exist if parsing succeeded

---

## 3. The Spec Is More Important Than the Code

The story spec (`1-26-workspace-dependency-graph-enforcement.md` in the axiathon repo) is the single source of truth for requirements. The README is the single source of truth for user-facing behavior.

- If the spec is wrong, fix the spec first, then implement. Never silently deviate.
- If a bug reveals a spec gap, update the appropriate file:

| Gap type | Fix it in |
|----------|-----------|
| Universal principle | SOUL.md |
| Project-specific instruction | CLAUDE.md or `.claude/rules/` |
| User-facing behavior | README.md |
| Under-specified requirement | Story spec (axiathon repo) |

---

## 4. Silent Failures Are the Enemy

This tool's entire purpose is catching things that would otherwise fail silently (bad deps slipping past review). Apply the same rigor internally:

- If `cargo metadata` fails, report the error — never silently produce an empty graph
- If config parsing fails, report exactly which field and why — never default to "no rules"
- If a workspace member isn't in the config, that's information — never silently skip it
- Tests must fail explicitly when the underlying tool errors — a vacuously true test is worse than no test

---

## 5. Errors Are Domain Knowledge, Not Strings

Errors should be structured, semantic, and domain-specific.

- Define error enums with `thiserror` — `ConfigError`, `MetadataError`, `ValidationError`
- Each variant carries the data needed to produce a clear message
- `Display` impls produce user-facing messages directly (no API layer to sanitize for — this is a CLI tool)

---

## 6. Explain Why, Not Just What

PR descriptions lead with motivation. Comments explain design constraints, not just behavior.

Good: `// Allowlist approach chosen over denylist — new crates must be explicitly permitted, matching the architecture doc pattern`
Bad: `// Check the allowlist`

---

## 7. Test Boundaries, Name Tests Like Documentation

Test names are executable documentation: `config_rejects_empty_crate_name()` tells you the contract. Not `test_1()`.

Test boundaries explicitly: empty config, missing crate, extra crate in config, crate with zero deps, self-dependency. Boundaries are where bugs live.

---

## 8. Standards Over Invention

- Follow cargo subcommand conventions (naming, `--manifest-path` flag, exit codes)
- Use `cargo_metadata` crate — don't parse `cargo metadata` JSON manually
- Use `clap` derive API — don't roll custom arg parsing
- Use `insta` for snapshot tests — don't hand-write expected output strings
- Follow crates.io publishing conventions (metadata fields, MIT license, exclude patterns)

---

## 9. Accessibility in CLI Output

No color-only information. Always pair colors with symbols:

- `ERROR:` prefix + red (not just red text)
- `OK` / checkmark + green (not just green text)
- Respect `NO_COLOR` env var and `--color` flag
- Ensure output is readable when piped to a file or another tool

---

## 10. Behavior Is Configuration; Policy Is Types

The allowlist rules live in a TOML config file — they change per-workspace without recompiling the tool. The validation logic, error types, and output formatting live in Rust types — they define the tool's contract.

| Configuration (TOML, per-workspace) | Code (Rust types, compile-time) |
|---|---|
| Allowed dependency lists per crate | Violation detection logic |
| Strict vs permissive mode | Error type structure |
| Whether to check dev-deps | Config parsing and validation |
| Config file path | CLI argument definitions |
| Output format (text/json) | Report formatting |
