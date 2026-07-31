# Development path — context-mcp

**Back-layer history.** How this project got to its current shape, reconstructed from git history, releases, and merged PRs. Inferences are labelled.

Companion measured snapshot: [CURRENT-STATE.md](CURRENT-STATE.md). Living plan: [ROADMAP.md](ROADMAP.md).

---

## Origin (early 2026)

The tree begins as a split baseline from a larger workspace restructure (`967b76c` — workspace restructure + subcrates + benchmarks + tests). The crate was extracted as a standalone Rust MCP server for **agent session / context memory**: store, get, query, and score contexts with temporal metadata.

Early product decisions (inferred from package layout, module names, and alpha release notes):

| Decision | Choice taken | Alternative (rejected or deferred) | Evidence |
|----------|--------------|------------------------------------|----------|
| Language / runtime | Rust + Tokio | Python MCP server | Crate + `Cargo.toml` from first tags |
| Protocol surface | MCP tools over JSON-RPC | Ad-hoc REST-only memory API | `src/protocol.rs`, `src/tools.rs`, nine tools from the start |
| Storage model | In-memory LRU + optional sled KV | Pure DB or remote vector DB first | `src/storage.rs`, feature `persistence` |
| Retrieval | Text/metadata scoring first | Full embedding RAG first | `src/rag.rs` historically used hash pseudo-vectors |
| Transport | stdio + HTTP (axum) | WebSocket as primary | Feature `server`; WS noted as incomplete in docs |

**Inferred:** Session KV was the shippable core; “RAG” naming and ternary/quantization work arrived as **efficiency and aspiration layers** before a real embedder existed. That ordering later required an honesty pass (C0) when marketing and keywords overclaimed semantic capability.

---

## Alpha through v0.1.6 (January 2026)

| Tag / commit | What landed | Why it mattered |
|--------------|-------------|-----------------|
| `v0.1.0-alpha.1` … `v0.1.4` | Packaging, docs.rs config, URL fixes | crates.io / docs.rs publish path |
| `v0.1.5` | Dev tooling, security scanning (`cargo deny` / audit), license hygiene | Local quality bar for a published crate |
| `v0.1.6` (`2d987e2`) | `install.sh`, INSTALL/USAGE docs, `test_mcp_server.py`, assessment report | Operator-facing install story; external stdio smoke harness |
| PR #8 (`33af9dc`) | Sparse balanced ternary embeddings | Quantization / ternary feature matrix (`ternary-*`, GPU feature stubs) |

**Decision — ternary / quantization early (PR #8):** Land compression and sparse ternary plumbing while embeddings were still mock/hash-based. **Why:** Explore efficiency paths for a future vector stack. **Cost:** Docs and release notes could be read as “RAG complete”; that mismatch is what C0 later corrected. Legitimate vector store remains Wave 2 (open).

**Decision — “production-ready” language at 0.1.6:** Release notes and `ASSESSMENT_REPORT.md` (dated 2026-01-10, server 0.1.5) labelled the session MCP as production-ready with throughput numbers and “lightweight RAG.” **Measured later (2026-07):** session KV remains the solid core; RAG claims were overstated relative to code. Prefer [CURRENT-STATE.md](CURRENT-STATE.md) over that report for capability truth.

---

## Honesty and fleet integration (July 2026)

### C0 honesty gate (PR #29, merge `cdb7f14`)

Merged into `dev` then promoted. Core product decision:

- `RagConfig.enable_semantic` **defaults to `false`**
- Semantic mode **fails closed** without a real (`is_semantic`) embedder
- Hash pseudo-embeddings quarantined off production retrieve paths
- Docs and crates.io keywords demoted “rag” to aspirational
- WebSocket/SSE overclaims corrected toward HTTP POST + SSE honesty

**Rejected alternative:** Keep pseudo-cosine scores labelled as semantic for demo convenience. **Why rejected:** Stale or soft claims made bugs invisible fleet-wide; session memory is valuable without pretending to be vector RAG.

### Tero + agent surface

- `AGENTS.md` and `docs/tero-index/` added so agents query a Layer-1 corpus index before large greps (PR path around #29, kickoff docs).
- Role split (declared and kept): **tero-mcp** = project corpus + citations; **context-mcp** = session runtime memory (+ future legitimate RAG). Do not merge responsibilities.

### Health TLC and fleet CI (PRs #32–#39)

| PR | Decision |
|----|----------|
| #32–#33 | Commitizen semver config; stay **0.x.x** under `major_version_zero` |
| #34 | wgpu 0.20 → 30.0 (API migration; GPU path still CPU-fallback in practice) |
| #35 | Linux x64 jobs → self-hosted podman fleet |
| #36 | Keywords honesty, secrets hygiene, `scripts/check.sh` + optional `git-secrets` |
| #37, #39 | Fleet standards badges; issue close-on-main policy |

**Decision — local check script as primary gate:** `./scripts/check.sh` (fmt, clippy `-D warnings`, doc, build, test; optional audit/deny/bench). CI workflows also run on push/PR via fleet packs; earlier “manual-only CI” wording in `docs/LOCAL_CHECKS.md` became **stale** after fleet re-apply (see CURRENT-STATE).

---

## Wave 1 embedder — v0.3.0 (July 2026)

| PR / tag | Content |
|----------|---------|
| PR #46 (`120be9b` …) | `Embedder` trait; `NullEmbedder`, `HashingEmbedder`; fail-closed semantic path; context embedding metadata fields |
| PR #47 / `v0.3.0` | Release polish: “Embedder Wave 1” |
| Optional feature | `http-embedder` → `HttpEmbedder` (OpenAI-compatible HTTP; `is_semantic = true`) |

**Decision — trait first, local model later:** Ship the pluggable interface and fail-closed semantics before ONNX/candle/GGUF. **Why:** Unblocks honest integration and tests without claiming MTEB-quality retrieval. **Still open:** real local backend (issue #19 / #45), vector ANN store (Wave 2), eval harness (Wave 3).

**Rejected for 0.3.0:** Marketing as legitimate RAG; enabling semantic by default; leaving hash fallback in semantic mode.

---

## Transport correctness (PR #48)

Two independent stdio defects fixed (`e530428`):

1. Logs went to **stdout** (JSON-RPC transport) → subscriber forced to **stderr**.
2. Wire format was **snake_case** where MCP requires **camelCase** → serde renames on initialize/tool types.

Regression test pins camelCase on the wire. This is a textbook case of “docs/status looked fine while clients could not connect.”

---

## Dependency refresh (PR #49)

`Cargo.lock` refresh to latest compatible versions on `main` (`2d07878`). No product surface change; tip measured for this PM suite.

---

## Version line (actual tags)

| Version | Approx. meaning |
|---------|-----------------|
| 0.1.x | Session MCP + packaging + docs/tooling |
| 0.2.0 | Semver baseline + C0 honesty narrative |
| **0.3.0** | Wave 1 Embedder + fail-closed semantic (current crate / cz version) |

**Policy (fleet contract):** Remain **0.x.x** until a human authorizes 1.x.x. Roadmap items that once said “1.0.0 when eval green” are product *intent*, not an agent authorization to cut 1.x.x.

---

## What shaped the codebase (summary)

1. **Session memory first** — nine MCP tools + LRU/sled; still the verified product.
2. **Efficiency experiments early** — ternary/RVQ/GPU features before real embeddings.
3. **Honesty over demo convenience** — C0 + Wave 1 fail-closed semantic.
4. **Fleet-shaped process** — commitizen 0.x, self-hosted CI, tero index for agents, docs as deliverable.
5. **Complementarity with tero** — corpus citations elsewhere; session store here.

Open product epics (issues): #19 real embedder backend, #20 GPU shaders, #21 security-mcp screening, #45 Wave 1 local backend stack framing.
