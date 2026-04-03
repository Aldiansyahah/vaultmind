# Contributing to VaultMind

Thank you for your interest in contributing to VaultMind! This guide covers everything you need to get started — whether you're a human developer or an AI agent.

## Table of Contents

- [Development Setup](#development-setup)
- [Branch Strategy](#branch-strategy)
- [Making Changes](#making-changes)
- [Code Standards](#code-standards)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)
- [For AI Agent Contributors](#for-ai-agent-contributors)
- [Architecture Overview](#architecture-overview)

---

## Development Setup

### Prerequisites

- **Rust** 1.75+ via [rustup](https://rustup.rs/)
- **Node.js** 20+ via [nvm](https://github.com/nvm-sh/nvm) or direct install
- **pnpm** 9+ (`npm install -g pnpm`)
- **Tauri system dependencies**: follow [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS

### First-Time Setup

```bash
# 1. Fork and clone
git clone https://github.com/Aldiansyahahah/vaultmind.git
cd vaultmind

# 2. Run setup script (installs deps, pre-commit hooks, downloads ONNX model)
chmod +x scripts/setup-dev.sh
./scripts/setup-dev.sh

# 3. Verify everything works
cargo test --workspace
pnpm test
pnpm tauri dev
```

The setup script will:
- Install Rust and Node dependencies
- Configure pre-commit hooks (rustfmt, clippy, eslint, prettier)
- Download the default ONNX embedding model via Git LFS
- Create a sample vault for development

**Target: new contributor productive in < 15 minutes.**

---

## Branch Strategy

| Branch | Purpose | Merges Into | Protection |
|---|---|---|---|
| `main` | Stable releases | — | PR + 1 review + CI green |
| `develop` | Integration | `main` (on release) | PR + CI green |
| `feature/*` | New features | `develop` | None |
| `fix/*` | Bug fixes | `develop` | None |
| `release/*` | Release prep | `main` + `develop` | CI green |

### Creating a Branch

```bash
# Always branch from develop
git checkout develop
git pull origin develop
git checkout -b feature/your-feature-name

# For bug fixes
git checkout -b fix/issue-number-description
```

---

## Making Changes

### 1. Create an Issue First

Before starting work, create or find a GitHub issue describing what you'll do. This prevents duplicate effort and enables discussion before coding.

### 2. Write Code

Follow the module structure. Changes typically touch one of these crates:

| Crate | Location | Responsibility |
|---|---|---|
| `core-storage` | `src-tauri/crates/core-storage/` | File I/O, SQLite, LanceDB |
| `indexer` | `src-tauri/crates/indexer/` | Markdown parsing, chunking, embedding |
| `graph-engine` | `src-tauri/crates/graph-engine/` | Knowledge graph operations |
| `retriever` | `src-tauri/crates/retriever/` | Hybrid search, re-ranking |
| `agent-runtime` | `src-tauri/crates/agent-runtime/` | LLM integration, tool execution |
| Frontend | `src/` | Svelte UI components and views |

### 3. Test Your Changes

```bash
# Rust tests
cargo test --workspace

# Rust linting (must pass with zero warnings)
cargo clippy --workspace -- -D warnings

# Rust formatting
cargo fmt --all --check

# Frontend tests
pnpm test

# Frontend linting
pnpm lint
```

### 4. Commit

Write clear commit messages following [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(indexer): add AST-based markdown chunking
fix(core-storage): handle concurrent SQLite writes
docs(readme): update quick start instructions
test(retriever): add RRF score fusion tests
refactor(graph-engine): extract edge builder into trait
```

---

## Code Standards

### Rust

- **Formatting**: `rustfmt` with default settings (enforced by CI)
- **Linting**: `cargo clippy -- -D warnings` (zero warnings policy)
- **Documentation**: All public APIs must have `///` doc comments
- **Error handling**: Use `thiserror` for library errors, `anyhow` for application code. No `unwrap()` or `panic!()` in library code.
- **Testing**: Use `#[cfg(test)]` modules. Minimum 80% coverage per crate.
- **Unsafe**: No `unsafe` blocks without written justification in a comment

### TypeScript / Svelte

- **Formatting**: Prettier with project config
- **Linting**: ESLint with project config
- **Types**: Strict TypeScript — no `any` unless justified
- **Components**: Svelte 5 runes syntax, small focused components

### General

- No hardcoded secrets, API keys, or credentials
- No commented-out code in PRs
- Inline comments for non-obvious logic
- English for all code, comments, and documentation

---

## Testing

| Level | Framework | What to Test |
|---|---|---|
| Unit | `cargo test` | Individual functions, struct methods |
| Integration | `cargo test` + temp dirs | Cross-crate interactions, SQLite ops |
| E2E | Playwright | Full app workflows |
| Benchmark | `criterion.rs` | Indexing speed, search latency |
| RAG Quality | Custom eval | Retrieval relevance (MRR, Recall@k) |

### Writing Tests

```rust
// In src-tauri/crates/indexer/src/chunker.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_respects_code_block_boundaries() {
        let markdown = "# Title\n\n```rust\nfn main() {\n    println!(\"hello\");\n}\n```\n\nSome text.";
        let chunks = chunk_document(markdown, &ChunkConfig::default());
        
        // Code block should never be split
        let code_chunk = chunks.iter().find(|c| c.content.contains("fn main")).unwrap();
        assert!(code_chunk.content.contains("println!"));
    }
}
```

---

## Pull Request Process

1. **Fill out the PR template** completely
2. **Link the issue** using `Closes #123` or `Fixes #123`
3. **Ensure CI passes** — PR is blocked if lint, test, or build fails
4. **Request review** from a maintainer
5. **Address feedback** — push fixes as new commits (don't force-push during review)
6. **Maintainer merges** after approval

### PR Review Checklist

| Criteria | Requirement |
|---|---|
| Correctness | Code does what the PR claims |
| Tests | New tests added, existing tests pass |
| Performance | No unnecessary allocations |
| Safety | No unsafe without justification |
| Documentation | Public APIs documented |
| Style | Passes rustfmt + clippy |

---

## For AI Agent Contributors

AI agents (Claude Code, Cursor, Aider, Copilot Workspace, etc.) are welcomed contributors. Follow all the guidelines above, plus these additions:

### Additional Rules for AI Agents

1. **Read before writing.** Read this file and relevant module `docs/` before making changes.
2. **Create an issue first.** Use the GitHub API or CLI to create an issue before starting. Reference it in commits.
3. **One task per PR.** Do not bundle unrelated changes.
4. **Explain your reasoning.** Add inline comments for non-obvious design decisions.
5. **Run all checks.** `cargo clippy --workspace -- -D warnings` before submitting.
6. **Test everything.** Add or update tests for every code change.
7. **Complete PR description.** Must include: What changed, Why, How to test, Related issues.
8. **Do not modify these without human approval:**
   - CI/CD configurations (`.github/workflows/`)
   - `LICENSE`
   - Security-sensitive code (crypto, auth, file permissions)
   - `Cargo.toml` dependency versions (suggest in PR, don't change)

### AI Agent Workflow Example

```bash
# 1. Create issue via GitHub CLI
gh issue create --title "feat(indexer): implement AST parser" --body "..."

# 2. Branch from develop
git checkout -b feature/ast-parser develop

# 3. Implement changes
# ... write code ...

# 4. Run all checks
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace

# 5. Commit and push
git add .
git commit -m "feat(indexer): implement AST parser using pulldown-cmark

Closes #12"
git push origin feature/ast-parser

# 6. Create PR
gh pr create --base develop --title "feat(indexer): AST parser" --body "..."
```

---

## Architecture Overview

```
┌─────────────────────────────────────────────┐
│         AI Agent Layer (agent-runtime)       │
├─────────────────────────────────────────────┤
│         Retrieval Layer (retriever)          │
├──────────────────────┬──────────────────────┤
│   Knowledge Graph    │   Indexing Pipeline   │
│   (graph-engine)     │   (indexer)           │
├──────────┬───────────┼──────────────────────┤
│  SQLite  │  LanceDB  │  Tantivy             │
│          │           │  (core-storage)       │
├──────────┴───────────┴──────────────────────┤
│         Tauri 2.0 Shell (Rust + IPC)        │
├─────────────────────────────────────────────┤
│         Svelte 5 Frontend                    │
└─────────────────────────────────────────────┘
```

Each crate has a clear public interface defined as Rust traits. See `docs/` for detailed API documentation and Architecture Decision Records (ADRs).

---

## Questions?

Open a [Discussion](https://github.com/Aldiansyahahah/vaultmind/discussions) or reach out via Issues. We're happy to help you get started.
