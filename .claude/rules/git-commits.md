<!-- Implementation rules for SOUL.md principles. SOUL owns "why", this file owns "how". -->

# Git Commit Standards

All commits MUST follow [Conventional Commits](https://www.conventionalcommits.org/).

## Format

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

## Required: Type

| Type     | Purpose                      |
|----------|------------------------------|
| feat     | New feature (MINOR version)  |
| fix      | Bug fix (PATCH version)      |
| docs     | Documentation only           |
| style    | Code style (no logic change) |
| refactor | Neither fix nor feature      |
| perf     | Performance improvement      |
| test     | Adding/fixing tests          |
| build    | Build system/dependencies    |
| ci       | CI configuration             |
| chore    | Other non-src/test changes   |

## Required: Description

- Use imperative, present tense ("add" not "added")
- Do NOT capitalize the first letter
- Do NOT end with a period

## Optional: Scope

Common scopes: `config`, `validate`, `metadata`, `report`, `cli`

## Breaking Changes

Indicate with `!` after type/scope: `feat(config)!: rename rules key`

## AI Attribution

Do NOT include in commit messages:
- The "Generated with Claude Code" line
- The "Co-Authored-By: Claude" line
- Any other AI attribution
