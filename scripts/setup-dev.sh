#!/usr/bin/env bash
set -euo pipefail

echo "================================================"
echo "  VaultMind Development Setup"
echo "================================================"
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

check_command() {
    if command -v "$1" &> /dev/null; then
        echo -e "  ${GREEN}✓${NC} $1 found"
        return 0
    else
        echo -e "  ${RED}✗${NC} $1 not found"
        return 1
    fi
}

echo "1. Checking prerequisites..."
echo ""

MISSING=0

check_command "rustc" || MISSING=1
check_command "cargo" || MISSING=1
check_command "node" || MISSING=1
check_command "pnpm" || MISSING=1

echo ""

if [ "$MISSING" -eq 1 ]; then
    echo -e "${RED}Missing prerequisites. Please install:${NC}"
    echo "  - Rust: https://rustup.rs/"
    echo "  - Node.js 20+: https://nodejs.org/"
    echo "  - pnpm: npm install -g pnpm"
    echo ""
    echo "Also install Tauri system dependencies:"
    echo "  https://v2.tauri.app/start/prerequisites/"
    exit 1
fi

echo -e "${GREEN}All prerequisites found!${NC}"
echo ""

# ── Install Node dependencies ──
echo "2. Installing Node dependencies..."
pnpm install
echo ""

# ── Install Rust components ──
echo "3. Checking Rust toolchain..."
rustup component add rustfmt clippy 2>/dev/null || true
echo -e "  ${GREEN}✓${NC} rustfmt and clippy installed"
echo ""

# ── Setup pre-commit hooks ──
echo "4. Setting up pre-commit hooks..."

mkdir -p .git/hooks 2>/dev/null || true

cat > .git/hooks/pre-commit << 'HOOK'
#!/usr/bin/env bash
set -e

echo "Running pre-commit checks..."

# Rust formatting
echo "  Checking Rust formatting..."
cd src-tauri && cargo fmt --all --check
cd ..

# Rust linting
echo "  Running Clippy..."
cd src-tauri && cargo clippy --workspace -- -D warnings
cd ..

# Frontend formatting
echo "  Checking frontend formatting..."
pnpm format:check

echo "All pre-commit checks passed!"
HOOK

chmod +x .git/hooks/pre-commit 2>/dev/null || true
echo -e "  ${GREEN}✓${NC} Pre-commit hook installed"
echo ""

# ── Create sample vault ──
echo "5. Creating sample development vault..."
mkdir -p dev-vault

cat > dev-vault/welcome.md << 'MD'
# Welcome to VaultMind

This is a sample vault for development. You can use it to test features.

## Getting Started

- Create new notes using the editor
- Link notes using [[wikilinks]]
- Tag notes with #tags
- Search your knowledge base with AI

## Links

- [[setup-guide]]
- [[architecture]]

#welcome #getting-started
MD

cat > dev-vault/setup-guide.md << 'MD'
# Setup Guide

Instructions for setting up VaultMind for development.

## Prerequisites

You need Rust, Node.js, and pnpm installed.

## Building

Run `pnpm tauri dev` to start in development mode.

[[welcome]]

#setup #development
MD

cat > dev-vault/architecture.md << 'MD'
# Architecture

VaultMind uses a layered architecture with RAG as a first-class concern.

## Layers

The system has 6 layers from storage to AI agent.

## Key Design Decisions

- AST-based chunking for semantic boundaries
- Hybrid retrieval (vector + BM25 + graph)
- Local-first with optional cloud LLM

[[welcome]] [[setup-guide]]

#architecture #design
MD

echo -e "  ${GREEN}✓${NC} Sample vault created at ./dev-vault/"
echo ""

# ── Git LFS ──
echo "6. Checking Git LFS..."
if command -v git-lfs &> /dev/null; then
    git lfs install 2>/dev/null || true
    echo -e "  ${GREEN}✓${NC} Git LFS configured"
else
    echo -e "  ${YELLOW}!${NC} Git LFS not installed (optional, needed for ONNX models)"
    echo "    Install: https://git-lfs.github.com/"
fi
echo ""

# ── Models directory ──
echo "7. Preparing models directory..."
touch models/.gitkeep
echo -e "  ${GREEN}✓${NC} models/ ready (add ONNX files here later)"
echo ""

# ── Done ──
echo "================================================"
echo -e "  ${GREEN}Setup complete!${NC}"
echo "================================================"
echo ""
echo "Next steps:"
echo "  1. Start development:  pnpm tauri dev"
echo "  2. Run Rust tests:     cd src-tauri && cargo test --workspace"
echo "  3. Run frontend tests: pnpm test"
echo ""
echo "Sample vault: ./dev-vault/"
echo "Docs: ./docs/"
echo ""
echo "Happy hacking!"
