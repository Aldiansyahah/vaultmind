# Changelog

All notable changes to VaultMind will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project scaffold with Tauri 2.0 + Svelte 5
- Rust workspace with 5 crates: core-storage, indexer, graph-engine, retriever, agent-runtime
- CI/CD pipelines (lint, test, build, release)
- Contributing guidelines for humans and AI agents
- Issue templates and PR template
- Development setup script
- Sample vault for development
- SQLite database initialization and migration system (P1-05)
- Notes table with CRUD operations and content hash for change detection (P1-05)
- Tags table with CRUD operations (P1-05)
- Note-tags junction table with association operations (P1-05)
- Custom error types with thiserror-style error handling (P1-05)
- File system watcher with 300ms debouncing for vault directory (P1-06)
- Structured WatchEvent types: FileCreated, FileModified, FileDeleted, FileRenamed (P1-06)
- Markdown file filtering: only .md/.markdown, ignores hidden/temp/swap/OS files (P1-06)
- VaultWatcher API with start/stop/recv/try_recv for indexing pipeline integration (P1-06)
