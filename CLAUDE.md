# VaultMind — AI Agent Instructions

## Project Overview

VaultMind is a RAG-optimized personal knowledge management system built with Tauri 2.0 (Rust backend) + Svelte 5 (frontend). Every note is automatically parsed, chunked, embedded, and connected in a knowledge graph for high-quality AI-powered retrieval.

## Key Documents

- `docs/VaultMind-SRS-v1.0.docx` — Full specification, requirements, architecture, and development roadmap
- `CONTRIBUTING.md` — Code standards, branch strategy, and PR process
- `CHANGELOG.md` — Track all changes here

## Current Status

**Phase 0: Complete** — Repository, CI/CD, project structure, branch protection.
**Phase 1: Starting** — Core Note-Taking MVP.

## Architecture

```
src-tauri/
├── Cargo.toml              # Workspace root
├── src/lib.rs              # Tauri entry point + IPC commands
├── src/main.rs             # Binary entry
└── crates/
    ├── core-storage/       # File I/O, SQLite, LanceDB, file watcher
    ├── indexer/            # Markdown AST parser, chunker, embedder
    ├── graph-engine/       # Knowledge graph (petgraph)
    ├── retriever/          # Hybrid search, RRF, re-ranking
    └── agent-runtime/      # LLM integration, tool-calling

src/                        # Svelte 5 frontend
├── App.svelte              # Root component
├── main.ts                 # Entry point
├── lib/components/         # Reusable UI components
├── lib/stores/             # Svelte stores (state management)
└── routes/                 # App views (editor, graph, search, chat, settings)
```

## Tech Stack

| Layer            | Technology                | Version             |
| ---------------- | ------------------------- | ------------------- |
| App Shell        | Tauri                     | 2.x                 |
| Backend          | Rust                      | 1.75+               |
| Frontend         | Svelte                    | 5.x                 |
| Editor           | TipTap                    | 2.x                 |
| Vector DB        | LanceDB                   | 0.4+                |
| Metadata DB      | SQLite (rusqlite)         | 3.45+               |
| Full-Text Search | Tantivy                   | 0.22+               |
| Markdown Parser  | pulldown-cmark            | 0.12+               |
| Graph            | petgraph                  | 0.6+                |
| Embeddings       | Ollama (nomic-embed-text) | via localhost:11434 |
| Graph Viz        | D3.js                     | 7.x                 |
| File Watcher     | notify                    | 7.x                 |

## Rules for AI Agent

### Git Workflow

1. Always branch from `develop`: `git checkout -b feature/P1-XX-description develop`
2. Never push directly to `main`
3. One task per branch and PR
4. Reference task ID in commits: `feat(core-storage): setup SQLite schema (P1-05)`
5. Update CHANGELOG.md for every feature/fix

### Code Quality

1. Rust: `cargo fmt --all` before committing
2. Rust: `cargo clippy --workspace -- -D warnings` must pass with zero warnings
3. Rust: `cargo test --workspace` must pass
4. Frontend: `pnpm lint` and `pnpm format:check` must pass
5. Add `#[cfg(test)]` module with tests for every new public function
6. All public APIs must have `///` doc comments
7. No `unwrap()` or `panic!()` in library code — use `thiserror` for errors
8. No hardcoded paths, secrets, or credentials

### Architecture Rules

1. Heavy computation (indexing, search, graph) happens in Rust crates, not frontend
2. Frontend communicates with backend via Tauri IPC commands (`#[tauri::command]`)
3. Each crate exposes a clean public API via traits
4. SQLite for metadata, LanceDB for vectors, Tantivy for full-text — never mix responsibilities
5. Markdown files on disk are the source of truth — database is a derived index
6. All database operations must handle concurrent access safely

## Development Roadmap

### Phase 1: Core Note-Taking MVP (Current)

Complete these tasks in order. Each task should be a separate branch and PR.

#### P1-05: SQLite Database Setup (START HERE)

**Crate:** `core-storage`
**What to build:**

- Database initialization and migration system
- Tables: `notes` (id, path, title, content_hash, created_at, updated_at), `tags` (id, name), `note_tags` (note_id, tag_id)
- CRUD operations for notes and tags
- Content hash for change detection (avoid re-indexing unchanged files)
  **Acceptance criteria:**
- `cargo test -p core-storage` passes with tests for all CRUD operations
- Database created automatically on first run
- Migration system supports schema evolution

#### P1-06: File System Watcher

**Crate:** `core-storage`
**What to build:**

- Watch vault directory for file create/modify/delete/rename events using `notify` crate
- Debounce rapid changes (300ms)
- Emit structured events that the indexing pipeline can consume
- Handle edge cases: temp files, .DS_Store, non-markdown files
  **Acceptance criteria:**
- Watcher detects changes within 1 second
- Only .md files trigger events
- Debouncing prevents duplicate events from editor save

#### P1-01: File Tree Sidebar

**Module:** Frontend (`src/lib/components/`)
**What to build:**

- Tree view component showing vault directory structure
- Create new note (with title prompt)
- Rename note/folder
- Delete note/folder (with confirmation)
- Drag-and-drop move
  **Backend IPC commands needed:**
- `list_vault_files` — returns directory tree
- `create_note` — creates .md file
- `rename_note` — renames file
- `delete_note` — deletes file (with trash support)
- `move_note` — moves file to new path
  **Acceptance criteria:**
- File tree reflects actual disk state
- Changes on disk (from watcher) update tree automatically

#### P1-02: TipTap Markdown Editor

**Module:** Frontend (`src/lib/components/`)
**What to build:**

- TipTap editor instance with Markdown extensions
- Bidirectional sync: edit in WYSIWYG → valid .md on disk, edit .md on disk → reflected in editor
- Support: headings, bold, italic, code blocks, lists, blockquotes, links, images
- Auto-save on change (debounced 1 second)
  **Dependencies:** `npm install @tiptap/core @tiptap/starter-kit @tiptap/extension-link @tiptap/extension-code-block-lowlight`
  **Acceptance criteria:**
- Edit in editor, open file in external editor → content matches
- Edit file externally, editor updates within 1 second

#### P1-03: Wikilink Support

**Module:** Frontend (editor extension) + Backend (parser)
**What to build:**

- Custom TipTap extension for `[[wikilink]]` syntax
- Autocomplete dropdown when typing `[[` (searches note titles)
- Clicking a wikilink opens the target note
- Backend: extract wikilinks from markdown content for graph edges later
  **Acceptance criteria:**
- Typing `[[` shows autocomplete with existing note titles
- Rendered wikilinks are clickable and navigate to target
- Wikilinks in .md files are standard `[[note-name]]` syntax

#### P1-04: Tag Support

**Module:** Frontend + Backend
**What to build:**

- Parse `#tag` syntax in markdown content
- Tag autocomplete when typing `#`
- Store tags in SQLite (tags + note_tags tables)
- Tag panel/filter in sidebar
  **Acceptance criteria:**
- Tags extracted and stored on note save
- Autocomplete shows existing tags
- Can filter/search notes by tag

#### P1-07: Tantivy Full-Text Search

**Crate:** `retriever`
**What to build:**

- Tantivy index for full-text search over note content
- Index updated incrementally on file change
- Search API: query string → ranked results with snippets
- BM25 scoring
  **Acceptance criteria:**
- Search returns results in < 100ms for 1000 notes
- Results include highlighted snippet showing match context
- Index survives app restart (persisted to disk)

#### P1-08: Search UI

**Module:** Frontend
**What to build:**

- Search input with keyboard shortcut (Ctrl+K / Cmd+K)
- Result cards showing: note title, highlighted snippet, tags, last modified
- Click result → opens note in editor
- Real-time search-as-you-type (debounced 200ms)
  **Acceptance criteria:**
- Search results appear within 300ms of typing
- Keyboard navigation (arrow keys + Enter)

#### P1-09: Settings Panel

**Module:** Frontend + Backend
**What to build:**

- Settings view accessible from sidebar
- Configurable: vault path, theme (light/dark), editor font size
- Settings persisted in SQLite or JSON config file
  **Acceptance criteria:**
- Changing vault path triggers reindex
- Theme switch works without restart

#### P1-10: Cross-Platform Testing

**What to do:**

- Test on macOS, Linux (Ubuntu), Windows
- Fix platform-specific bugs
- Verify CI builds on all 3 platforms
- Tag as v0.1.0-alpha release
  **Acceptance criteria:**
- `pnpm tauri build` succeeds on all 3 platforms
- Core features work identically across platforms

### Phase 2-5: See docs/VaultMind-SRS-v1.0.docx for full details.

## Embedding Configuration

Use Ollama with `nomic-embed-text` model for all embedding operations.
Do NOT use ONNX Runtime — the user already has Ollama running locally.

### Ollama API

- Endpoint: `http://localhost:11434/api/embed`
- Model: `nomic-embed-text:latest`
- Dimensions: 768
- Max context: 8192 tokens

### API Call Example (Rust)

```rust
// POST http://localhost:11434/api/embed
// Body: { "model": "nomic-embed-text", "input": ["chunk text here"] }
// Response: { "embeddings": [[0.1, 0.2, ...]] }

use reqwest::Client;
use serde_json::json;

let client = Client::new();
let response = client
    .post("http://localhost:11434/api/embed")
    .json(&json!({
        "model": "nomic-embed-text",
        "input": ["Your chunk text here"]
    }))
    .send()
    .await?;
```

### Impact on Architecture

- Crate `indexer`: embedding generation via HTTP call to Ollama (reqwest), not ONNX
- LanceDB vector dimension: 768 (not 384)
- No need to bundle ONNX model files — removes ~80-150 MB from binary
- Ollama must be running for indexing; search works offline from stored vectors
- ChunkConfig max_chunk_tokens can be increased up to 8192 (nomic supports long context)
- The `ort` dependency is removed from Cargo.toml — use `reqwest` instead

## Non-Functional Requirements (Key Targets)

- App startup: < 2 seconds
- Search latency: < 200ms for 10k notes
- Indexing: > 50 notes/sec
- Memory (idle): < 150 MB
- Binary size: < 50 MB
- Offline: 100% core features without internet
