---
name: dependency-bump-cargo-workspace
description: Workflow command scaffold for dependency-bump-cargo-workspace in dprc-autotrader-v2.
allowed_tools: ["Bash", "Read", "Write", "Grep", "Glob"]
---

# /dependency-bump-cargo-workspace

Use this workflow when working on **dependency-bump-cargo-workspace** in `dprc-autotrader-v2`.

## Goal

Updates dependencies across multiple Cargo.toml files in the workspace, typically using dependabot or similar automation.

## Common Files

- `Cargo.toml`
- `Cargo.lock`
- `rig-core/Cargo.toml`
- `rig-lancedb/Cargo.toml`
- `rig-mongodb/Cargo.toml`
- `rig-neo4j/Cargo.toml`

## Suggested Sequence

1. Understand the current state and failure mode before editing.
2. Make the smallest coherent change that satisfies the workflow goal.
3. Run the most relevant verification for touched files.
4. Summarize what changed and what still needs review.

## Typical Commit Signals

- Update version numbers in Cargo.toml files for affected crates.
- Regenerate Cargo.lock to reflect new dependency versions.
- Commit changes to all updated Cargo.toml and Cargo.lock files.

## Notes

- Treat this as a scaffold, not a hard-coded script.
- Update the command if the workflow evolves materially.