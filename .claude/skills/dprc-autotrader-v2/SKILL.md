---
name: dprc-autotrader-v2-conventions
description: Development conventions and patterns for dprc-autotrader-v2. Rust Rust project with mixed commits.
---

# Dprc Autotrader V2 Conventions

> Generated from [affaan-m/dprc-autotrader-v2](https://github.com/affaan-m/dprc-autotrader-v2) on 2026-03-21

## Overview

This skill teaches Claude the development patterns and conventions used in dprc-autotrader-v2.

## Tech Stack

- **Primary Language**: Rust
- **Framework**: Rust
- **Architecture**: hybrid module organization
- **Test Location**: separate

## When to Use This Skill

Activate this skill when:
- Making changes to this repository
- Adding new features following established patterns
- Writing tests that match project conventions
- Creating commits with proper message format

## Commit Conventions

Follow these commit message conventions based on 357 analyzed commits.

### Commit Style: Mixed Style

### Prefixes Used

- `fix`
- `docs`
- `feat`
- `refactor`
- `misc`
- `chore`
- `devops`

### Message Guidelines

- Average message length: ~45 characters
- Keep first line concise and descriptive
- Use imperative mood ("Add feature" not "Added feature")


*Commit message example*

```text
chore(deps): bump the cargo group across 1 directory with 8 updates
```

*Commit message example*

```text
fix(rig-sqlite): Use tokio-rusqlite from crates.io instead of custom crate (#158)
```

*Commit message example*

```text
test(rig-sqlite): Add integration test (#202)
```

*Commit message example*

```text
docs(README): add SQLite as a supported vector store (#201)
```

*Commit message example*

```text
feat: Add `from_url` method to Gemini client (#194)
```

*Commit message example*

```text
swarm implementation
```

*Commit message example*

```text
fixed errors and birdeye working again
```

*Commit message example*

```text
error handling and better console logs
```

## Architecture

### Project Structure: Single Package

This project uses **hybrid** module organization.

### Source Layout

```
src/
├── database/
├── personality/
├── strategy/
```

### Configuration Files

- `rig-solana-trader/docker-compose.yml`

### Guidelines

- This project uses a hybrid organization
- Follow existing patterns when adding new code

## Code Style

### Language: Rust

### Naming Conventions

| Element | Convention |
|---------|------------|
| Files | camelCase |
| Functions | camelCase |
| Classes | PascalCase |
| Constants | SCREAMING_SNAKE_CASE |

### Import Style: Relative Imports

### Export Style: Named Exports


*Preferred import style*

```typescript
// Use relative imports
import { Button } from '../components/Button'
import { useAuth } from './hooks/useAuth'
```

*Preferred export style*

```typescript
// Use named exports
export function calculateTotal() { ... }
export const TAX_RATE = 0.1
export interface Order { ... }
```

## Common Workflows

These workflows were detected from analyzing commit patterns.

### Feature Development

Standard feature implementation workflow

**Frequency**: ~14 times per month

**Steps**:
1. Add feature implementation
2. Add tests for feature
3. Update documentation

**Files typically involved**:
- `**/*.test.*`

**Example commit sequence**:
```
feat: embeddings API overhaul (#120)
misc: Add `rig-derive` missing manifest fields (#129)
feat: Improve `InMemoryVectorStore` API (#130)
```

### Refactoring

Code refactoring and cleanup workflow

**Frequency**: ~4 times per month

**Steps**:
1. Ensure tests pass before refactor
2. Refactor code structure
3. Verify tests still pass

**Files typically involved**:
- `src/**/*`

**Example commit sequence**:
```
feat: embeddings API overhaul (#120)
misc: Add `rig-derive` missing manifest fields (#129)
feat: Improve `InMemoryVectorStore` API (#130)
```

### Dependency Bump Cargo Workspace

Updates dependencies across multiple Cargo.toml files in the workspace, typically using dependabot or similar automation.

**Frequency**: ~2 times per month

**Steps**:
1. Update version numbers in Cargo.toml files for affected crates.
2. Regenerate Cargo.lock to reflect new dependency versions.
3. Commit changes to all updated Cargo.toml and Cargo.lock files.

**Files typically involved**:
- `Cargo.toml`
- `Cargo.lock`
- `rig-core/Cargo.toml`
- `rig-lancedb/Cargo.toml`
- `rig-mongodb/Cargo.toml`
- `rig-neo4j/Cargo.toml`
- `rig-qdrant/Cargo.toml`
- `rig-sqlite/Cargo.toml`

**Example commit sequence**:
```
Update version numbers in Cargo.toml files for affected crates.
Regenerate Cargo.lock to reflect new dependency versions.
Commit changes to all updated Cargo.toml and Cargo.lock files.
```

### Release Version Workspace

Prepares and publishes a new release by updating CHANGELOG.md and Cargo.toml version fields for all crates, and updating Cargo.lock.

**Frequency**: ~2 times per month

**Steps**:
1. Update version numbers in Cargo.toml for all workspace crates.
2. Update CHANGELOG.md for each crate to reflect new changes.
3. Update Cargo.lock to match new versions.
4. Commit all version and changelog updates together.

**Files typically involved**:
- `Cargo.lock`
- `rig-core/CHANGELOG.md`
- `rig-core/Cargo.toml`
- `rig-lancedb/CHANGELOG.md`
- `rig-lancedb/Cargo.toml`
- `rig-mongodb/CHANGELOG.md`
- `rig-mongodb/Cargo.toml`
- `rig-neo4j/CHANGELOG.md`
- `rig-neo4j/Cargo.toml`
- `rig-qdrant/CHANGELOG.md`
- `rig-qdrant/Cargo.toml`
- `rig-sqlite/CHANGELOG.md`
- `rig-sqlite/Cargo.toml`

**Example commit sequence**:
```
Update version numbers in Cargo.toml for all workspace crates.
Update CHANGELOG.md for each crate to reflect new changes.
Update Cargo.lock to match new versions.
Commit all version and changelog updates together.
```

### Add Or Update Integration Test For Vector Store

Adds or updates integration tests for vector store crates (MongoDB, Neo4j, Qdrant, LanceDB, SQLite), often including example files and test fixtures.

**Frequency**: ~2 times per month

**Steps**:
1. Create or update integration test file in tests/integration_tests.rs.
2. Update or add example files demonstrating usage.
3. Modify src/lib.rs or related implementation files as needed.
4. Update Cargo.toml to include new dependencies or features for testing.
5. Commit all related changes together.

**Files typically involved**:
- `rig-mongodb/tests/integration_tests.rs`
- `rig-mongodb/examples/vector_search_mongodb.rs`
- `rig-mongodb/src/lib.rs`
- `rig-mongodb/Cargo.toml`
- `rig-neo4j/tests/integration_tests.rs`
- `rig-neo4j/examples/vector_search_simple.rs`
- `rig-neo4j/src/lib.rs`
- `rig-neo4j/Cargo.toml`
- `rig-qdrant/tests/integration_tests.rs`
- `rig-qdrant/examples/qdrant_vector_search.rs`
- `rig-qdrant/src/lib.rs`
- `rig-qdrant/Cargo.toml`
- `rig-lancedb/tests/integration_tests.rs`
- `rig-lancedb/examples/vector_search_local_ann.rs`
- `rig-lancedb/src/lib.rs`
- `rig-lancedb/Cargo.toml`
- `rig-sqlite/tests/integration_test.rs`
- `rig-sqlite/examples/vector_search_sqlite.rs`
- `rig-sqlite/src/lib.rs`
- `rig-sqlite/Cargo.toml`

**Example commit sequence**:
```
Create or update integration test file in tests/integration_tests.rs.
Update or add example files demonstrating usage.
Modify src/lib.rs or related implementation files as needed.
Update Cargo.toml to include new dependencies or features for testing.
Commit all related changes together.
```

### Add New Vector Store Backend

Implements support for a new vector store backend (e.g., SQLite, MongoDB, Neo4j, Qdrant, LanceDB), including code, examples, and documentation.

**Frequency**: ~1 times per month

**Steps**:
1. Create new crate directory (e.g., rig-sqlite/).
2. Add src/lib.rs with implementation.
3. Add README.md and LICENSE.
4. Add example files demonstrating usage.
5. Update workspace Cargo.toml and Cargo.lock.
6. Add integration tests.
7. Commit all new files and updates.

**Files typically involved**:
- `rig-sqlite/Cargo.toml`
- `rig-sqlite/LICENSE`
- `rig-sqlite/README.md`
- `rig-sqlite/examples/vector_search_sqlite.rs`
- `rig-sqlite/src/lib.rs`
- `rig-sqlite/tests/integration_test.rs`
- `Cargo.toml`
- `Cargo.lock`

**Example commit sequence**:
```
Create new crate directory (e.g., rig-sqlite/).
Add src/lib.rs with implementation.
Add README.md and LICENSE.
Add example files demonstrating usage.
Update workspace Cargo.toml and Cargo.lock.
Add integration tests.
Commit all new files and updates.
```

### Docs Readme Or Example Update

Updates documentation, README files, or example files to reflect new features, fix typos, or add usage instructions.

**Frequency**: ~3 times per month

**Steps**:
1. Edit README.md or crate-specific README.md files.
2. Edit or add example files in examples/ directory.
3. Commit documentation changes (sometimes with code changes).

**Files typically involved**:
- `README.md`
- `rig-core/README.md`
- `rig-core/examples/*.rs`
- `rig-lancedb/examples/*.rs`
- `rig-neo4j/examples/*.rs`

**Example commit sequence**:
```
Edit README.md or crate-specific README.md files.
Edit or add example files in examples/ directory.
Commit documentation changes (sometimes with code changes).
```


## Best Practices

Based on analysis of the codebase, follow these practices:

### Do

- Use camelCase for file names
- Prefer named exports

### Don't

- Don't deviate from established patterns without discussion

---

*This skill was auto-generated by [ECC Tools](https://ecc.tools). Review and customize as needed for your team.*
