# VaultMind

**RAG-optimized personal knowledge management system.**

VaultMind is a desktop app built with [Tauri 2.0](https://v2.tauri.app/) where Retrieval-Augmented Generation (RAG) is a first-class architectural concern — not a plugin afterthought. Every note you create is automatically parsed, chunked, embedded, and connected in a knowledge graph for high-quality AI-powered retrieval.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![CI](https://img.shields.io/github/actions/workflow/status/YOUR_USERNAME/vaultmind/ci.yml?branch=develop&label=CI)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey)

---

## Why VaultMind?

| | Obsidian + Plugins | VaultMind |
|---|---|---|
| Chunking | Naive (per-file/heading) | AST-based with hierarchical context |
| Embeddings | Plugin dependency | Built-in (ONNX, local-first) |
| Knowledge Graph | Link graph only | Explicit (wikilinks) + implicit (semantic similarity) |
| AI Agent | Chat-only | Action-capable (create, edit, link, refactor notes) |
| Retrieval | Vector search only | Hybrid (vector + BM25 + graph expansion + re-ranking) |
| Architecture | Plugin bolt-on | RAG as core design principle |

## Features (Roadmap)

- **Phase 1** — Markdown editor with wikilinks, tags, full-text search
- **Phase 2** — Smart chunking pipeline with local embeddings (ONNX)
- **Phase 3** — Knowledge graph + hybrid retrieval + Q&A
- **Phase 4** — AI agent with action capabilities
- **Phase 5** — Multi-modal (PDF, images, audio) + plugin system

## Tech Stack

| Layer | Technology |
|---|---|
| App Shell | Tauri 2.0 (Rust) |
| Frontend | Svelte 5 + TipTap editor |
| Vector DB | LanceDB (embedded) |
| Metadata | SQLite (rusqlite) |
| Full-Text | Tantivy |
| Embeddings | ONNX Runtime (all-MiniLM-L6-v2) |
| Graph | petgraph |

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (1.75+)
- [Node.js](https://nodejs.org/) (20+)
- [pnpm](https://pnpm.io/) (9+)
- System dependencies for Tauri: see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)

### Setup

```bash
# Clone the repository
git clone https://github.com/YOUR_USERNAME/vaultmind.git
cd vaultmind

# Run the setup script (installs dependencies, hooks, etc.)
./scripts/setup-dev.sh

# Start development
pnpm tauri dev
```

### Build

```bash
# Build for your platform
pnpm tauri build
```

## Project Structure

```
vaultmind/
├── .github/               # CI/CD, issue templates, PR template
├── src-tauri/              # Rust backend
│   ├── Cargo.toml          # Workspace root
│   ├── crates/
│   │   ├── core-storage/   # File I/O, SQLite, LanceDB
│   │   ├── indexer/        # AST parser, chunker, embedder
│   │   ├── graph-engine/   # Knowledge graph
│   │   ├── retriever/      # Hybrid search, re-ranking
│   │   └── agent-runtime/  # LLM integration, tool execution
│   └── src/main.rs         # Tauri entry point
├── src/                    # Svelte frontend
│   ├── lib/components/     # Reusable UI components
│   ├── lib/stores/         # State management
│   └── routes/             # App views
├── models/                 # ONNX models (git-lfs)
├── docs/                   # Architecture docs, ADRs
├── tests/                  # Integration & E2E tests
└── scripts/                # Dev scripts, benchmarks
```

## Contributing

We welcome contributions from both humans and AI agents. See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Branch Strategy

| Branch | Purpose |
|---|---|
| `main` | Stable releases |
| `develop` | Integration branch |
| `feature/*` | New features |
| `fix/*` | Bug fixes |

## License

MIT — see [LICENSE](LICENSE) for details.
