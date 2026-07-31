# Current state — context-mcp

**Back-layer, MEASURED.** What works today, with evidence. Prefer this file over older assessment reports when they disagree.

| Field | Value |
|-------|--------|
| **Measured at (UTC)** | 2026-07-25 |
| **Commit** | `2d07878c2091a706e1f3c5e5e02ddda9e96cc980` (`main` tip at measure time) |
| **Crate / cz version** | 0.3.0 |
| **Default branch** | `main` |

---

## Capability matrix

| Capability | Status | Notes |
|------------|--------|--------|
| Build (default + all features) | **VERIFIED** | `cargo build --all-features` exit 0 |
| Unit tests | **VERIFIED** | 40 passed, 0 failed (`cargo test --all-features`) |
| Integration tests | **VERIFIED** | 4 passed (delete/index consistency) |
| `cargo fmt --check` | **VERIFIED** | exit 0 |
| `cargo clippy --all-targets --all-features -- -D warnings` | **VERIFIED** | exit 0 |
| `cargo doc --all-features --no-deps` (`RUSTDOCFLAGS=-D warnings`) | **VERIFIED** | exit 0 |
| CLI `--help` / `--version` | **VERIFIED** | prints 0.3.0 and option list |
| HTTP listen + `/health` | **VERIFIED** | `{"server":"context-mcp","status":"ok","version":"0.3.0"}` |
| HTTP JSON-RPC `tools/list` | **VERIFIED** | 9 tools returned (see list below) |
| stdio JSON-RPC `initialize` | **VERIFIED** | camelCase wire (`serverInfo`, `protocolVersion`); logs on stderr only |
| Library example `examples/basic_usage.rs` | **VERIFIED** | store / get / query ok |
| MCP tools store/get/delete/query (unit + integration) | **VERIFIED** | covered by Rust tests + HTTP tools list |
| Text/metadata/temporal retrieve (`retrieve_contexts`, semantic off) | **VERIFIED** | unit tests + tool description honesty |
| Semantic mode fail-closed (no embedder / hash / null) | **VERIFIED** | unit tests in `rag::tests` |
| `HashingEmbedder` / `NullEmbedder` / trait surface | **VERIFIED** | unit tests in `embeddings::tests` |
| `HttpEmbedder` live call to a real model endpoint | **UNVERIFIED** | feature compiles; no network embed call run in this docs pass |
| Persistent sled store under load / restart | **UNVERIFIED** | feature builds; no multi-process restart durability test run here |
| Python `test_mcp_server.py` full harness | **UNVERIFIED** | not executed in this pass (stdio smoke was Rust binary only) |
| Throughput “7,421 contexts/s” / sub-ms latencies in README | **UNVERIFIED** | historical `ASSESSMENT_REPORT.md` (server **0.1.5**, 2026-01-10); not re-benchmarked |
| Legitimate vector RAG / ANN store | **UNVERIFIED** (absent) | not implemented; open Wave 2 |
| Local GGUF/ONNX/candle embedder | **UNVERIFIED** (absent) | issue #19 / #45 |
| GPU similarity shaders (non-CPU path) | **UNVERIFIED** | `gpu-acceleration` builds; tests exercise CPU fallback (`test_gpu_compute_fallback`) |
| HTTP auth / token gate | **UNVERIFIED** (absent) | roadmap C0.3 scoped/deferred |
| WebSocket MCP fully wired | **UNVERIFIED** (incomplete) | docs/code still warn WS not fully wired |
| security-mcp live screening | **UNVERIFIED** (absent) | status fields only; issue #21 |
| crates.io install of published 0.3.0 artifact | **UNVERIFIED** | not re-fetched via `cargo install` in this pass; local workspace binary used |

### MCP tools present on the wire (HTTP `tools/list`, measured)

1. `store_context`
2. `get_context`
3. `delete_context`
4. `query_contexts`
5. `retrieve_contexts` (metadata/temporal/keyword; semantic off by default)
6. `update_screening`
7. `get_temporal_stats`
8. `get_storage_stats`
9. `cleanup_expired`

---

## How this was measured

Resource limit: `export CARGO_BUILD_JOBS=3` for all cargo invocations.

### Format

```text
$ cargo fmt --check
# exit 0 (no output)
```

### Clippy

```text
$ cargo clippy --all-targets --all-features -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 04s
# exit 0
```

### Build

```text
$ cargo build --all-features
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 06s
# exit 0
```

### Tests (real summary lines)

```text
$ cargo test --all-features

running 40 tests
...
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.61s

running 0 tests   # bin crate
test result: ok. 0 passed; 0 failed; ...

running 4 tests   # tests/integration_test.rs
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

running 0 tests   # doc-tests
test result: ok. 0 passed; 0 failed; ...
```

### Docs

```text
$ RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps
   Generated .../target/doc/context_mcp/index.html
# exit 0
```

### CLI

```text
$ cargo run --features server -- --version
context-mcp 0.3.0

$ cargo run --features server -- --help
# shows --stdio, --host, --port, --storage-path, --cache-size, --persist,
# --threads, --no-decay, --enable-semantic
```

### HTTP smoke

```text
$ ./target/debug/context-mcp --host 127.0.0.1 --port 18765
$ curl -sS http://127.0.0.1:18765/health
{"server":"context-mcp","status":"ok","version":"0.3.0"}

$ curl -sS -X POST http://127.0.0.1:18765/mcp \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'
# result.tools: 9 tools (store_context … cleanup_expired)
```

### stdio smoke

```text
$ printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"docs-measure","version":"0.0.1"}}}' \
  | ./target/debug/context-mcp --stdio
{"jsonrpc":"2.0","id":1,"result":{"capabilities":{"tools":{"listChanged":true}},"protocolVersion":"2024-11-05","serverInfo":{"name":"context-mcp","version":"0.3.0"}}}
# stderr: Starting MCP Context Server in stdio mode
```

### Library example

```text
$ cargo run --example basic_usage --features server,persistence
Stored context with ID: j1tUQr64Wm5TEf10QGS2cA==
Retrieved: This is some important information
Found 1 matching contexts
# exit 0
```

### CI status (GitHub Actions API)

Command:

```bash
gh api /repos/tzervas/context-mcp/actions/runs?per_page=10
```

Recent runs observed (trimmed):

| Workflow | Conclusion | Branch / title | Created (UTC) |
|----------|------------|----------------|---------------|
| CI | success | `fix/reopen-issues-yaml-block-scalar` | 2026-07-25 |
| fleet-ci | success | same | 2026-07-25 |
| fleet-security | success | same | 2026-07-25 |
| fleet-ci | success | dependabot tower | 2026-07-23 |
| CI | success | dependabot tower | 2026-07-23 |
| reopen-issues workflow | **failure** | dependabot tower | 2026-07-23 |
| fleet-ci | **failure** | dependabot async-trait | 2026-07-23 |

On **main** tip `2d07878` specifically (branch filter):

| Workflow | Conclusion | Title |
|----------|------------|--------|
| CI | success | chore(deps) refresh (#49) |
| fleet-ci | **failure** | chore(deps) refresh (#49) |
| reopen-issues-closed-off-main | **failure** | same |

**Interpretation:** Local rustc gate is green. Remote fleet-ci is not uniformly green on main tip; reopen-issues workflow has been flaky/broken (YAML/startup issues called out in contract history). **No required status checks** on `main` (dispatcher measurement) — green badges or empty required lists must not be read as “merge verified.”

---

## Known defects and gaps (observed; not fixed in docs PR)

| Item | Evidence |
|------|----------|
| README library snippet used incomplete `StorageConfig { memory_cache_size, enable_persistence }` | Struct requires more fields (`persist_path`, `auto_cleanup`, `cleanup_interval_secs`); **would not compile**. Fixed in README in this docs suite to use `StorageConfig::default()` / full fields — see PR. |
| `docs/LOCAL_CHECKS.md` claimed workflows are **manual only** (`workflow_dispatch`) | False as of fleet re-apply: `ci.yml`, `fleet-ci.yml`, `fleet-security.yml` trigger on `push` and `pull_request`. Corrected in this suite. |
| `docs/ASSESSMENT.md` header still says crate **0.2.0** / date 2026-07-08 | Stale vs 0.3.0; matrix still directionally useful but version stamp wrong. |
| `ASSESSMENT_REPORT.md` (0.1.5) claims “lightweight RAG” and fixed throughput numbers | Historical; not re-run; language overclaims vs current honesty stance. |
| README “Performance” section echoed those numbers | Marked historical / not re-measured in this pass. |
| `src/storage.rs` TODO: vector similarity when embeddings available | Still present (read-only survey). |
| Open issues #19, #20, #21, #45 | Product epics unsolved. |
| fleet-ci failure on main tip #49 | Remote gate not clean; local check is. |

---

## What the product is *for* today (honest)

**VERIFIED role:** Local-first **session / runtime memory MCP** — store contexts with domains, tags, TTL, importance; query by metadata; score retrieve with temporal/keyword signals; optional sled persistence; stdio or loopback HTTP.

**Not yet:** Legitimate semantic RAG (real local model default, vector index, hybrid eval, auth’d multi-tenant HTTP). Wave 1 only landed the **embedder interface** and fail-closed semantics.

See [ROADMAP.md](ROADMAP.md) for what would unblock the rest.
