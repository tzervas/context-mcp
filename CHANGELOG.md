# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-07-09

### Semver Baseline

This establishes the **v0.2.0 semver baseline** for the context-mcp repo only (disjoint scope).

- **Manifest alignment**: `Cargo.toml` at `version = "0.2.0"` (no change needed; prior tags only reached v0.1.6).
- **Tero-first scoping** (executed: `/root/git/scripts/tero.sh context-mcp text_search "version"`, `"release"`, `"changelog"`, `"0.2"`):
  - Index: `context-mcp/docs/tero-index/index.json`
  - Citations (selected): `install--check-version` (INSTALL.md:144,139; `context-mcp --version`), `contributing--release-process` (CONTRIBUTING.md:117), `changelog--0.1.0-alpha.1` (CHANGELOG.md:60), `context-mcp-plan--version-2026-01-09-post-multi-response-analysis` (context-mcp-plan.md:2), `contributing--development-setup`/`--prerequisites` (CONTRIBUTING.md:5,7), `integrationdeploymentguide--track-all-cves-across-versions`, `benchmarkrealdatareport--*` (release notes refs), `changelog--added-4`.
  - Explain: `candidates_matched:10/2/1`, `candidates_scanned:506`; hits ordered by match score desc (title~ + summary~) then canonical (family/file/line/anchor); query terms resolved; `unresolved_edges:[]`; kind=answer. (Full resolvable citations + EXPLAIN trace per Tero Layer-1.)
- **Cites plan.md**: Core rules (Tero-first via scripts/tero.sh or tero MCP, dev-workflow, branch-guard (working branches only), worktree-guard, append-only docs/AGENTS/CHANGELOG, signed commits (-S)); w2-rollout in_progress, C0.4 "Align version tags with 0.2.0 story (0.2.0 in Cargo; tags later)". See plan.md:6,44,85+ (hygiene-thin-repos, wsfull-wave refs). Tero-grounded.
- **Release preference**: local podman GHCR (no Actions). (Note: CONTRIBUTING.md describes GH Actions automation; LOCAL_CHECKS.md states "GitHub Actions workflows ... manual only (`workflow_dispatch`)" + "Day-to-day quality gates run locally". Preference applied here.)
- **Process**: chore/semver-baseline-v0.2.0 only (worktree: /tmp/semver-context-mcp); hygiene (scripts/check.sh + cargo); signed commit; annotated/signed tag v0.2.0; push branch+tag; gh release v0.2.0 w/ artifacts. Push remotes periodically. Append-only.
- **Other files updated** (append-only, tero cites): AGENTS.md, docs/ROADMAP.md, README.md, docs/LOCAL_CHECKS.md.
- Follows dev-workflow, branch-guard, worktree-guard, append-only, tero cites. (See ROADMAP.md C0.4, AGENTS.md latest.)

### Added
- Semver baseline v0.2.0 tag + dedicated chore branch for version alignment story.

### Changed
- Documentation appends for baseline (CHANGELOG + peers) with Tero/plan.md citations.

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
