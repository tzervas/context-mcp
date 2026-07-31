# context-mcp — Product roadmap

**Status:** Living (updated 2026-07-25 for PM suite; product waves originally framed 2026-07-08)  
**Version policy:** Stay on **0.x.x** under commitizen until a **human** authorizes 1.x.x (fleet contract). Do not treat any wave as an automatic 1.0 cut.  
**North star:** Fast, local-first **session memory** MCP that can also deliver **efficient legitimate RAG** — real embeddings, real vector retrieval, measured quality — without lying about pseudo-similarity.

Companions: [CURRENT-STATE.md](CURRENT-STATE.md) (measured today), [DEVELOPMENT-PATH.md](DEVELOPMENT-PATH.md) (history), [ASSESSMENT.md](ASSESSMENT.md) (gap narrative).

---

## Definition of done — efficient legitimate RAG

All required before any “RAG” marketing or cabal PROD-6-style consumption:

1. **Real embeddings** — pluggable embedder (local GGUF/ONNX/API); model id + dims recorded per item  
2. **Fail closed** — semantic mode never silently uses hash pseudo-vectors  
3. **Vector storage** — ANN or documented exact search; embeddings persisted with items; reindex story  
4. **Efficiency** — batch embed, content-hash cache; optional quantization **on real vectors only**  
5. **Honest API** — separate scores for metadata vs semantic; tool names/docs match  
6. **Empirical eval** — fixed question→context-id set; report vs keyword baseline  
7. **Secure local defaults** — stdio / loopback; token auth if HTTP  

---

## Waves

### Wave 0 — Honesty & session product

| ID | Work | Status | What would unblock remaining |
|----|------|--------|------------------------------|
| C0.1 | Docs: remove “production RAG” claims; crates.io keywords review | **done** (PR #29) | — |
| C0.2 | Semantic mode off by default OR hard-error if no real embedder | **done** (default false + fail-closed; PR #29 / #46) | — |
| C0.3 | Token auth for HTTP | **open / deferred** | Design decision (shared fleet auth vs local token); implementation + tests; not blocked on embedder |
| C0.4 | Align version tags with baseline story | **partial** | Tags `v0.2.0` / `v0.3.0` exist; keep changelog honest on each release |
| C0.5 | Fix WebSocket/SSE overclaims | **done** in docs/comments | Full WS MCP still incomplete — see CURRENT-STATE |

### Wave 1 — Real embedder interface

### Wave 1 — Real embedder interface

| ID | Work | Status | What would unblock |
|----|------|--------|--------------------|
| C1.1 | Trait `Embedder: embed_batch / model_id / dims / is_semantic` | **done** — `src/embeddings.rs` (PR #46, v0.3.0) | — |
| C1.2 | At least one **local semantic** backend (fastembed / candle / GGUF / ONNX) | **partial** — `HashingEmbedder` local deterministic (non-semantic, tests); `NullEmbedder` fail-closed stub. Local GGUF/ONNX/candle model still open (issue #19). | Choose backend + license; wire feature; integration test with offline fixture model; closes epic #19 / framing #45 |
| C1.3 | Optional OpenAI-compatible HTTP embedder | **done (feature)** — `HttpEmbedder` / `http-embedder` | Live network test optional; not required for trait completeness |
| C1.6 | Select/construct the embedder from CLI or config | **done** — `--embedder none|local|http` + `--embed-model` / `--embed-dims` / `--embed-base-url`; `EmbedderConfig` on `ServerConfig`, built in `ServerState::new` (PR #58). Unavailable backends abort at startup naming the missing cargo feature — never a silent downgrade. | — |
| C1.4 | Store `embedding_model`, `dims`, `content_hash` on each item | **done** — fields on `Context` + `apply_embedding` / `with_embedding_info` | — |
| C1.5 | Delete or quarantine `text_to_pseudo_embedding` from production paths | **done** — quarantined under tests; fail-closed | — |


**Wave 1 honesty:** Trait + fail-closed + optional HTTP path landed. **Not legitimate RAG** — no vector ANN store, no hybrid rank eval, no MTEB claims. Default `enable_semantic=false`. MCP tool surface unchanged for session use.

### Wave 2 — Vector store & retrieve

| ID | Work | Status | What would unblock |
|----|------|--------|--------------------|
| C2.1 | Persist vectors (sled extension, sqlite-vss, or dedicated index) | **open** | C1.2 or reliable `HttpEmbedder` in prod config; schema design for embedding bytes + model id |
| C2.2 | `retrieve_semantic` using real cosine/ANN | **open** | C2.1 + embedder at query time; new tool or gated path; tests with fixture vectors |
| C2.3 | Hybrid rank: α·semantic + β·temporal + γ·tags (documented) | **open** | C2.2; documented weight defaults + unit tests |
| C2.4 | Reindex CLI: `context-mcp reindex --path ...` | **open** | C2.1; CLI surface design |

Also tracked in code: `src/storage.rs` TODO — vector similarity when embeddings available.

### Wave 3 — Efficiency & eval

| ID | Work | Status | What would unblock |
|----|------|--------|--------------------|
| C3.1 | Batch embed + cache by content hash | **open** | Wave 2 retrieve path worth optimizing |
| C3.2 | Quantization (ternary/RVQ) **only** as compression of real vectors | **partial plumbing** | Real vectors first; current ternary features must not claim RAG quality |
| C3.3 | Eval harness + CI job (small, CPU) | **open** | Fixed golden set; decide keyword baseline metric; CI minutes on self-hosted runners |
| C3.4 | Latency budgets in docs (p50/p95 local) | **open** | Re-run benchmarks on current code (old ASSESSMENT_REPORT is 0.1.5-era) |
| C3.5 | Production maturity cut | **proposed, not committed** | Human authorization for any **1.x.x**; until then ship polished **0.x** when eval gates green |

---

## Open issues (product signals)

| Issue | Title | Wave |
|-------|--------|------|
| #19 | Replace mock embedding generator with a real embedding backend | C1.2 |
| #45 | [1.0 stack] Wave 1: real Embedder backend (C1.1–C1.2) | C1.2 (C1.1 done in 0.3.0) |
| #20 | GPU compute shaders for similarity (CPU-fallback placeholder) | post–C2 / efficiency |
| #21 | Wire real security-mcp integration for content screening | orthogonal; screening fields exist |

---

## API plan

### MCP tools — current (session KV; keep)

| Tool | Purpose |
|------|---------|
| `store_context` | Store content + metadata/TTL |
| `get_context` / `delete_context` | CRUD by id |
| `query_contexts` | Filter metadata |
| `retrieve_contexts` | Scored metadata/temporal/keyword; semantic gated |
| `update_screening` | Safety flag on item |
| stats / cleanup tools | Operational |

### MCP tools — planned (RAG)

| Tool | Purpose | Notes |
|------|---------|--------|
| `embed_status` | Active model, dims, cache stats | proposed |
| `retrieve_semantic` | Query string → top-k with real vectors | Requires Wave 1–2 |
| `retrieve_hybrid` | Semantic + metadata + temporal | |
| `reindex` | Admin/refresh scope | Token-scoped if HTTP auth lands |

### Response envelope (target)

```json
{
  "kind": "answer",
  "items": [
    {
      "id": "uuid",
      "score": 0.82,
      "score_breakdown": {
        "semantic": 0.9,
        "temporal": 0.5,
        "tags": 1.0
      },
      "embedding_model": "all-MiniLM-L6-v2",
      "content": "...",
      "citations": []
    }
  ]
}
```

Refusals: empty semantic index / missing embedder → **typed error**, not fake scores.

### Library API (Rust) — target shape

```rust
pub trait Embedder: Send + Sync {
    fn model_id(&self) -> &str;
    fn dims(&self) -> usize;
    fn is_semantic(&self) -> bool;
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
}

// Wave 2 target (not all present today):
// impl ContextStore {
//     pub async fn store_with_embed(&self, ctx: Context) -> Result<Id>;
//     pub async fn retrieve_semantic(&self, q: &str, k: usize) -> Result<Vec<ScoredContext>>;
// }
```

### Config (CLI / env) — direction

### Config (CLI / env) — direction

| Flag / env | Meaning | Status |
|------------|---------|--------|
| `--embedder none\|local\|http` | Required for semantic | **done** — default `none`; `local` = non-semantic hashing stub (C1.2 still open), `http` needs the `http-embedder` cargo feature |
| `--embed-model` | Model id/path | **done** — required for `http` |
| `--embed-dims` | Vector dimensionality | **done** — required for `http`; `local` defaults to 384 |
| `--embed-base-url` | API root for `http` | **done** — `http` is unusable without it |
| `$CONTEXT_MCP_EMBED_API_KEY` | Bearer token for `http` | **done** — env only, deliberately not a flag (argv is world-readable via `ps`) |
| `--enable-semantic` | Opt into semantic retrieve | **done** — defaults false; aborts startup unless selected backend is `is_semantic()` |
| `--vector-path` | Index location | **open** — Wave 2 (C2.1). No flag is exposed; there is no vector index to point it at |


---

## Known defects to track (from measurement, not fixed here)

See [CURRENT-STATE.md](CURRENT-STATE.md). Highlights:

- Historical perf numbers in root assessment report not re-run on 0.3.0  
- fleet-ci not clean on all main tip runs; local `cargo` gate is  
- HTTP auth missing  
- GPU path falls back to CPU  

---

## Relationship to Tero

| System | Use |
|--------|-----|
| tero-mcp | Project corpus, citations, decisions |
| context-mcp | Agent session memory → **plus** legitimate RAG over stored contexts (future) |

Do not merge responsibilities; agents may call both.

---

## PR sequencing (suggested, not dated)

1. ~~Docs assessment + roadmap~~  
2. ~~Honesty: gate pseudo path + README~~  
3. ~~`Embedder` trait + fail-closed~~ (0.3.0)  
4. Local semantic backend (C1.2 / #19)  
5. Vector persist + `retrieve_semantic` (Wave 2)  
6. Eval harness + hybrid rank (Wave 3)  
7. Quantization as optimization only on real vectors  
8. Human-gated maturity release (0.x polish or authorized 1.x — **human only**)  
