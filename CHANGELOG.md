# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2026-07-21

### Added
- Wave 1 `Embedder` trait (`embed_batch`, `model_id`, `dims`, `is_semantic`) in `src/embeddings.rs`
- `NullEmbedder` fail-closed stub; `HashingEmbedder` deterministic local (non-semantic, tests)
- Optional `HttpEmbedder` (`--features http-embedder`) OpenAI-compatible HTTP backend
- `Context` fields: `embedding_model`, `embedding_dims`, `content_hash`; `apply_embedding` helper
- Errors: `EmbedderUnavailable`, `SemanticUnavailable`
- `RagProcessor::with_embedder` — semantic retrieve uses real embedder only

### Changed
- Semantic mode fail closed: `enable_semantic=true` without `is_semantic` embedder returns typed error (no hash pseudo fallback)
- Production path no longer calls `text_to_pseudo_embedding` (quarantined under tests)

### Notes
- Still **not** legitimate RAG (Wave 2 vector store + Wave 3 eval remaining). MCP tools stable for cabal facade.

## [0.2.0] - 2026-07-10

### Changed
- Semver baseline for supportive tooling/helper crate (`Cargo.toml` version `0.2.0`)
- C0 honesty: `RagConfig.enable_semantic` defaults to `false`; pseudo-embeddings warn when forced on
- crates.io keywords ordered with session-memory terms first; `rag` remains aspirational until Wave gates

### Notes
- Not legitimate RAG yet (see `docs/ROADMAP.md`, issues #19–#21 for product epics)
- wgpu optional feature tracked separately (GPU compute still CPU-fallback; issue #20)

## [0.1.6] - 2026-01-10

### Added
- Comprehensive installation script (`install.sh`) for streamlined setup
- Detailed installation documentation (`INSTALL.md`) with platform-specific guides
- Usage examples and scenarios documentation (`USAGE_EXAMPLES.md`)
- Performance assessment report (`ASSESSMENT_REPORT.md`) with full benchmarks
- Automated test suite (`test_mcp_server.py`) for validation and benchmarking
- VS Code MCP configuration examples and troubleshooting guides

### Changed
- Updated README.md with quick start guide and performance metrics
- Status upgraded from "Alpha" to "Production-ready"
- Improved documentation structure with clear navigation

### Performance
- Validated: 7,421 contexts/second sustained throughput
- Validated: Sub-millisecond latency (0.13-0.23ms average)
- 100% test pass rate across all 9 MCP tools (23 tests)

### Documentation
- Added comprehensive installation guides for Linux, macOS, and Windows
- Added VS Code integration examples with multiple configuration options
- Added performance benchmarks with rigorous methodology
- Added troubleshooting section with common issues and solutions

## [0.1.5] - 2026-01-09

### Fixed
- Various bug fixes and stability improvements

## [0.1.4] - 2026-01-08

### Changed
- Performance optimizations
- Documentation improvements

## [0.1.3] - 2026-01-07

### Added
- Additional MCP tool implementations

## [0.1.2] - 2026-01-06

### Fixed
- Bug fixes and improvements

## [0.1.1] - 2026-01-05

### Added
- Initial MCP server implementation

## [0.1.0-alpha.1] - 2026-01-04

### Added
- Initial alpha release
- Basic context storage and retrieval
- Temporal tracking
- In-memory LRU cache
- Optional sled persistence
