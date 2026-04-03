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
- TipTap WYSIWYG markdown editor with StarterKit (headings, bold, italic, code, lists, blockquotes, links) (P1-02)
- Auto-save with 1s debounce and bidirectional content sync (P1-02)
- Wikilink support: [[target]] syntax with autocomplete dropdown (P1-03)
- Wikilink extraction parser with deduplication (P1-03)
- Tag extraction parser for #tag syntax (P1-03)
- Tauri IPC commands for wikilink and tag extraction (P1-03)
- Tag sync operations: sync_tags_for_note, list_all_tags_with_counts (P1-04)
- Tag autocomplete when typing # in editor with keyboard navigation (P1-04)
- TagPanel in sidebar with usage counts and filter by tag (P1-04)
- Tauri IPC commands: sync_note_tags, get_all_tags, get_notes_for_tag (P1-04)
- File tree sidebar with recursive vault listing and note selection (P1-01)
- New note creation with modal dialog and validation (P1-01)
- Note editor with auto-save (1s debounce) and loading/error states (P1-01)
- Tauri IPC commands: set_vault_path, list_vault_files, create_note, rename_note, delete_note, move_note, read/write content (P1-01)
- Svelte stores for vault state management with action helpers (P1-01)
