# context-mcp

<!-- FLEET-BADGES:BEGIN -->
[![CI](https://github.com/tzervas/context-mcp/actions/workflows/fleet-ci.yml/badge.svg?branch=main)](https://github.com/tzervas/context-mcp/actions/workflows/fleet-ci.yml?query=branch%3Amain)
[![Security](https://github.com/tzervas/context-mcp/actions/workflows/fleet-security.yml/badge.svg?branch=main)](https://github.com/tzervas/context-mcp/actions/workflows/fleet-security.yml?query=branch%3Amain)
<!-- FLEET-BADGES:END -->

[![Crates.io](https://img.shields.io/crates/v/context-mcp.svg)](https://crates.io/crates/context-mcp)
[![Documentation](https://docs.rs/context-mcp/badge.svg)](https://docs.rs/context-mcp/latest/context_mcp/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

MCP server for context management, temporal metadata, and lightweight (non-semantic) retrieval plumbing.

This crate provides a "memory service" for agents: store/retrieve context items with timestamps and metadata, supporting basic query/retrieval patterns via text matching and filtering.

## Quick Start

### Install

```bash
curl -fsSL https://raw.githubusercontent.com/tzervas/context-mcp/main/install.sh | bash
```

Or via cargo:
```bash
cargo install context-mcp
```

**See [INSTALL.md](INSTALL.md) for detailed installation instructions and VS Code configuration.**

### Run

```bash
# Stdio transport (for MCP clients like VS Code)
context-mcp --stdio

# HTTP server
context-mcp --host 127.0.0.1 --port 3000
```

Under `--stdio`, **stdout carries the JSON-RPC stream and nothing else** — all logs go to
stderr. Redirect stderr (`2>/tmp/context-mcp.log`) to inspect them without disturbing the
transport; anything written to stdout by a future change will break every MCP client.

Registering with Claude Code:

```bash
# -s user makes it available in every project; use -s project for one repo
claude mcp add context-mcp -s user -- /absolute/path/to/context-mcp --stdio
claude mcp list   # expect: context-mcp: … - ✔ Connected
```

## Features

- **Multi-tier Storage**: In-memory LRU cache with optional sled-based disk persistence
- **Temporal Tracking**: Timestamps, age tracking, and time-based filtering for context relevance
- **CPU-Optimized Retrieval**: Parallel processing with rayon for text-based context queries
- **MCP Protocol Support**: JSON-RPC server implementation with HTTP (POST + SSE) and stdio transports (WebSocket not yet fully wired for MCP)
- **Screening Status Fields**: Built-in fields for tracking security screening state (integration not included)

## Performance

Validated through comprehensive benchmarking:
- **7,421 contexts/second** sustained throughput
- **Sub-millisecond latency** (0.13-0.23ms average)
- **100% test pass rate** across all 9 MCP tools

See [ASSESSMENT_REPORT.md](ASSESSMENT_REPORT.md) for detailed performance analysis.

## Documentation

- **[INSTALL.md](INSTALL.md)** - Installation and setup guide
- **[USAGE_EXAMPLES.md](USAGE_EXAMPLES.md)** - Usage examples and scenarios
- **[ASSESSMENT_REPORT.md](ASSESSMENT_REPORT.md)** - Performance benchmarks and validation
- **[API Documentation](https://docs.rs/context-mcp)** - Rust API reference

## Status

Production-ready for context management, temporal tracking, and metadata query (session memory MCP).
**No legitimate RAG yet** (no ANN vector store / eval). Wave 1 adds a pluggable `Embedder` trait
(`NullEmbedder` fail-closed, `HashingEmbedder` test-only non-semantic, optional `http-embedder`).
Semantic mode stays **off by default** and **fails closed** without a real (`is_semantic`) embedder —
hash pseudo is quarantined from production paths. See docs/ROADMAP.md (C0 + Wave 1).

**W2 / Facade**: Part of common memory vision (tero L1 citations + context-mcp session/RAG + memory-gate domains). See dev-docs/schemas/ for W2 StructuredResponse + facade adapters (used in cabal-devmelopner). Context-mcp integrates as session backend today; real RAG post-Wave 1-2. PR #29 + tero reindex capture updates. (wsfull wave 2026-07-09)

**Post-merge W2/C0 + wsfull state (2026-07-09):** PR #29 (feature/ctx-c0-honesty) merged. C0 honesty gate landed (semantic=false default), tero reindexed. References wsfull-wave-2026-07-09-compact.md + common memory facade (cabal integration). See docs/ROADMAP.md, AGENTS.md. Verification + hygiene complete.

## Usage

### As a Library

```rust
use context_mcp::{ContextStore, StorageConfig, Context, ContextDomain};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create storage configuration
    let config = StorageConfig {
        memory_cache_size: 1000,
        enable_persistence: true,
    };

    // Create context store
    let store = ContextStore::new(config)?;

    // Store some context
    let ctx = Context::new("This is some important information", ContextDomain::Code);
    let id = store.store(ctx).await?;

    // Retrieve it
    let retrieved = store.get(&id).await?;
    println!("Retrieved: {}", retrieved.content);

    Ok(())
}
```

### As an MCP Server

Run as HTTP server:
```bash
context-mcp --host 127.0.0.1 --port 3000
```

Run as stdio transport:
```bash
context-mcp --stdio
```

## What It Does (Verified by Code/Tests)

- **JSON-RPC MCP Server**: Runs over HTTP/WebSocket or stdio transport
- **Context Storage**: Store/retrieve contexts with IDs, domains, timestamps, tags, and custom metadata
- **Tiered Storage**: In-memory LRU cache (always) + optional sled disk persistence
- **Text-Based Queries**: Query by text content, domain, tags, time ranges with simple text matching
- **Temporal Filtering**: Filter contexts by creation time, last access, age, and expiration
- **Parallel Processing**: CPU-optimized retrieval using rayon for performance
- **MCP Tools**: 9 tools including store_context, get_context, query_contexts, retrieve_contexts, delete_context, update_screening, get_temporal_stats, get_storage_stats, cleanup_expired

## What It Does Not Do (Yet)

- **Vector embeddings**: Mock implementation only - no real embedding generation or similarity search
- **Semantic search**: Text matching is literal, not semantic
- **GPU acceleration**: `gpu-acceleration` feature flag exists but the compute shader path is a placeholder; similarity computation always falls back to CPU
- **External integrations**: No active security-mcp or other service integrations (only status fields)
- **Chunking/citations**: No automatic document chunking or citation tracking
- **Distributed storage**: Single-node only, no replication or clustering

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   MCP Client    │    │  JSON-RPC Server │    │ Storage Layer   │
│                 │    │                  │    │                 │
│ • HTTP/WS       │◄──►│ • Store/Retrieve │◄──►│ • In-Memory LRU │
│ • stdio         │    │ • Query/Filter   │    │ • Sled (opt)    │
│ • curl/tools    │    │ • Temporal Stats │    │ • Domain Index  │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                              │
                              ▼
                       ┌──────────────┐
                       │ RAG Processor│
                       │ • Text Match │
                       │ • Parallel   │
                       │ • Scoring    │
                       └──────────────┘
```

## Development

### Setup

1. Install development dependencies:
   ```bash
   ./setup-dev.sh
   ```

2. Run all quality checks:
   ```bash
   just check
   ```

### Available Commands

This project uses [just](https://github.com/casey/just) for development tasks:

```bash
just                    # Show all available tasks
just check             # Run all quality checks (fmt, clippy, test, security, docs)
just test              # Run tests
just bench             # Run benchmarks
just docs              # Generate documentation
just security          # Run security checks
just audit             # Run security audit
just licenses          # Check licenses
just build             # Build all targets
just dev               # Full development cycle
just pre-commit        # Pre-commit checks
just ci                # Simulate CI pipeline locally
```

### Code Quality

- **Formatting**: `cargo fmt` (enforced)
- **Linting**: `cargo clippy` with warnings as errors
- **Testing**: 100% test coverage target with `cargo tarpaulin`
- **Security**: `cargo audit` and `cargo deny` for vulnerabilities and license compliance
- **Documentation**: `cargo doc` with warning checks
- **Dependencies**: Unused dependency detection with `cargo udeps`

### Security Scanning

The project includes comprehensive security scanning:

- **Vulnerability scanning**: `cargo audit` checks for known security issues
- **License compliance**: `cargo deny` ensures only approved licenses
- **Dependency analysis**: Checks for unused and outdated dependencies
- **Pre-commit hooks**: Automatic quality checks before commits

### Benchmarking

Performance benchmarks are included for critical paths:

```bash
just bench             # Run all benchmarks
just bench-flamegraph  # Generate flamegraphs (requires cargo-flamegraph)
```

## Versioning

This project follows [Conventional Commits](https://www.conventionalcommits.org/) and uses [Commitizen](https://commitizen-tools.github.io/commitizen/) for release versioning. Version is tracked in `.cz.toml` and `Cargo.toml`. Before release, dispatch the **Commitizen** workflow (Actions → Commitizen → Run workflow) to verify commits on the current branch.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and contribution guidelines.

## License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.

## Status & roadmap

- [Assessment & gaps](docs/ASSESSMENT.md)
- [Product roadmap & API plans](docs/ROADMAP.md)
