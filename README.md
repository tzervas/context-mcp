# context-mcp

<!-- FLEET-BADGES:BEGIN -->
[![CI](https://github.com/tzervas/context-mcp/actions/workflows/fleet-ci.yml/badge.svg?branch=main)](https://github.com/tzervas/context-mcp/actions/workflows/fleet-ci.yml?query=branch%3Amain)
[![Security](https://github.com/tzervas/context-mcp/actions/workflows/fleet-security.yml/badge.svg?branch=main)](https://github.com/tzervas/context-mcp/actions/workflows/fleet-security.yml?query=branch%3Amain)
<!-- FLEET-BADGES:END -->

[![Crates.io](https://img.shields.io/crates/v/context-mcp.svg)](https://crates.io/crates/context-mcp)
[![Documentation](https://docs.rs/context-mcp/badge.svg)](https://docs.rs/context-mcp/latest/context_mcp/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Session memory for agents.** An MCP server (and Rust library) that stores, queries, and scores context items with temporal metadata. Local-first: stdio for MCP clients, or loopback HTTP.

**Not legitimate vector RAG yet.** Semantic mode is off by default and fails closed without a real embedder. See [docs/CURRENT-STATE.md](docs/CURRENT-STATE.md).

| | |
|--|--|
| **Who it's for** | Agent runtimes, IDE MCP clients, tools that need short-lived session context with TTL/tags |
| **Version** | 0.3.0 (0.x until a human authorizes otherwise) |
| **Status (measured)** | [docs/CURRENT-STATE.md](docs/CURRENT-STATE.md) |
| **History / roadmap** | [docs/DEVELOPMENT-PATH.md](docs/DEVELOPMENT-PATH.md) · [docs/ROADMAP.md](docs/ROADMAP.md) |

---

## Quick start (< 1 minute)

### Build from this repo

```bash
export CARGO_BUILD_JOBS=3   # polite on shared builders
cargo build --features server
./target/debug/context-mcp --version   # context-mcp 0.3.0
```

### Run

```bash
# Stdio (MCP clients). Logs go to stderr only — stdout is pure JSON-RPC.
# Disk persistence is ON by default (./data/context_store); pass --no-persist for memory-only.
./target/debug/context-mcp --stdio

# Explicit storage path (recommended for long-lived agents)
./target/debug/context-mcp --stdio --storage-path ~/.context-mcp/data

# HTTP (loopback)
./target/debug/context-mcp --host 127.0.0.1 --port 3000
curl -s http://127.0.0.1:3000/health
# {"server":"context-mcp","status":"ok","version":"0.3.0"}
```

### Install (optional)

```bash
cargo install context-mcp
# or: curl -fsSL https://raw.githubusercontent.com/tzervas/context-mcp/main/install.sh | bash
```

Platform detail and VS Code wiring: [INSTALL.md](INSTALL.md).

Claude Code example:

```bash
claude mcp add context-mcp -s user -- /absolute/path/to/context-mcp --stdio
claude mcp list
```

---

## What works

- **9 MCP tools:** `store_context`, `get_context`, `delete_context`, `query_contexts`, `retrieve_contexts`, `update_screening`, `get_temporal_stats`, `get_storage_stats`, `cleanup_expired`
- **Storage:** in-memory LRU + sled disk **on by default** (`--no-persist` for memory-only); domain/tag indices **rehydrate from disk** on open so persisted contexts stay findable after restart
- **Retrieval:** metadata, temporal decay, keyword-style scoring; semantic **off by default**
- **Wave 1 embedder surface:** `Embedder` trait, fail-closed semantic path, optional `http-embedder`; CLI `--embedder none|local|http` (see below)
- **Transports:** stdio + HTTP (POST JSON-RPC + health). WebSocket MCP is not fully wired.

## What does not work (yet)

- Legitimate ANN / vector RAG and hybrid eval (Wave 2–3)
- Default local embedding model (issue #19)
- Real GPU similarity path (CPU fallback; issue #20)
- Live security-mcp screening (fields only; issue #21)
- HTTP token auth (roadmap C0.3)

---

## Library (minimal)

```rust
use context_mcp::context::ContextDomain;
use context_mcp::{Context, ContextStore, StorageConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = ContextStore::new(StorageConfig::default())?;
    let id = store
        .store(Context::new("important note", ContextDomain::Code))
        .await?;
    let got = store.get(&id).await?.expect("stored");
    println!("{}", got.content);
    Ok(())
}
```

Runnable in-tree: `cargo run --example basic_usage --features server,persistence`.

### Selecting an embedder

`--embedder` chooses the backend; nothing is constructed by default.

| Flag | Meaning |
|------|---------|
| `--embedder none\|local\|http` | Backend to construct (default `none`) |
| `--embed-model <MODEL>` | Model id/path; required for `http` |
| `--embed-dims <N>` | Vector dimensionality; required for `http`, local defaults to 384 |
| `--embed-base-url <URL>` | API root for `http`, e.g. `https://api.openai.com/v1` |
| `$CONTEXT_MCP_EMBED_API_KEY` | Bearer token for `http`. Env only — a flag would leak it into `ps`/argv |

```bash
# Non-semantic local hashing backend (deterministic; wiring/dev only)
context-mcp --stdio --embedder local --embed-dims 384

# Real semantic embeddings — requires a build with the `http-embedder` feature
cargo build --release --features http-embedder
CONTEXT_MCP_EMBED_API_KEY=... context-mcp --stdio \
  --embedder http --embed-base-url https://api.openai.com/v1 \
  --embed-model text-embedding-3-small --embed-dims 1536 \
  --enable-semantic
```

**`--enable-semantic` aborts at startup** unless the selected backend reports
`is_semantic() == true`. It never falls back to the hashing stub. In the default build
(`server`, `persistence`, `ternary-embeddings`) `--embedder http` is unbuildable, and the
error names the missing cargo feature:

```
Error: Configuration error: --embedder http requires the `http-embedder` cargo feature,
which is NOT in the default feature set ... Rebuild with:
cargo build --release --features http-embedder
```

Vectors are still **not persisted or indexed**, and retrieval scoring does not yet use them —
that is Wave 2 (`docs/ROADMAP.md` C2.1–C2.2). Selecting an embedder makes the backend
reachable; it does not by itself make this legitimate RAG.

---

## Development

```bash
./scripts/check.sh --quick   # fmt, clippy -D warnings, doc, build, test
# or: just check
```

Details: [docs/LOCAL_CHECKS.md](docs/LOCAL_CHECKS.md), [CONTRIBUTING.md](CONTRIBUTING.md).

---

## Documentation map

| Doc | Contents |
|-----|----------|
| [docs/README.md](docs/README.md) | Index of all docs |
| [docs/CURRENT-STATE.md](docs/CURRENT-STATE.md) | Measured capabilities (VERIFIED / UNVERIFIED) |
| [docs/DEVELOPMENT-PATH.md](docs/DEVELOPMENT-PATH.md) | How the project evolved |
| [docs/ROADMAP.md](docs/ROADMAP.md) | Waves, unblocks, open issues |
| [INSTALL.md](INSTALL.md) · [USAGE_EXAMPLES.md](USAGE_EXAMPLES.md) | Install & scenarios |
| [docs.rs](https://docs.rs/context-mcp) | Rust API |

Historical benchmark writeups under the repo root (`ASSESSMENT_REPORT.md`, etc.) are **not** current measurements; do not treat their throughput numbers as live.

---

## Versioning & license

Conventional Commits + [Commitizen](https://commitizen-tools.github.io/commitizen/); version in `.cz.toml` / `Cargo.toml`. **0.x.x** until a human authorizes otherwise.

MIT — see [LICENSE](LICENSE).
