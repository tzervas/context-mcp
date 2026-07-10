# context-mcp — Product Roadmap

**Status:** Living (2026-07-08)  
**North star:** Fast, local-first **session memory** MCP that also delivers **efficient legitimate RAG** — real embeddings, real vector retrieval, measured quality — without lying about pseudo-similarity.

Companion: [ASSESSMENT.md](ASSESSMENT.md).

---

## Definition of done — efficient legitimate RAG

All required before any “RAG” marketing or cabal PROD-6 consumption:

1. **Real embeddings** — pluggable embedder (local GGUF/ONNX/API); model id + dims recorded per item  
2. **Fail closed** — semantic mode never silently uses hash pseudo-vectors  
3. **Vector storage** — ANN or documented exact search; embeddings persisted with items; reindex story  
4. **Efficiency** — batch embed, content-hash cache; optional quantization **on real vectors only**  
5. **Honest API** — separate scores for metadata vs semantic; tool names/docs match  
6. **Empirical eval** — fixed question→context-id set; report vs keyword baseline  
7. **Secure local defaults** — stdio / loopback; token auth if HTTP  

---

## Waves

### Wave 0 — Honesty & session product (ship now)

| ID | Work |
|----|------|
| C0.1 | Docs: remove “production RAG” claims; crates.io keywords review | (done in C0 PR #29: README, src/*, Cargo, ASSESSMENT) |
| C0.2 | Semantic mode off by default OR hard-error if no real embedder | (done: RagConfig.enable_semantic=false default; gated in score_context + tool desc; CLI --enable-semantic) |
| C0.3 | Token auth for HTTP | (scoped; stub or defer per orch if shared) |
| C0.4 | Align version tags with 0.2.0 story | (0.2.0 in Cargo; tags later) |
| C0.5 | Fix WebSocket/SSE overclaims | (done: README + server.rs comments updated to HTTP/POST+SSE) |

C0 gate complete (feature/ctx-c0-honesty, PR #29). Part of wsfull + W2 facade integration (common memory: tero + context-mcp + memory-gate). Tero reindexed post-changes. See compact wsfull-wave-2026-07-09.

**Post-merge W2/C0 + wsfull state (2026-07-09):** PR #29 merged to dev (cdb7f14). C0 items done + tero (504 items). W2 facade refs in docs point to wsfull-wave-2026-07-09-compact.md + dev-docs/schemas/common_memory_facade (cabal). Post-merge tero update + verification review posted. Continue per PR plan / Wave 1.

## W2 Session Consumer Note (chore/w2-rollout-docs-wiring)

Context-mcp session is W2 consumer in rollout (StructuredResponse/CommonMemoryAdapter + AgentDomain mirrors). 

- Domain CONTEXT in facade for session memory_contexts (cabal agent + memory-gate cross).
- See plan.md §2 (w2-rollout in_progress), wsfull-wave-2026-07-09-compact.md, dev-docs/schemas/ (examples), dev-mcp/servers/context-mcp.md (W2 usage), AGENTS.md here.
- Tero-first cites to plan/wsfull. Append-only hygiene/update-tero/land/propagate/verify.

Wave 0 updated with this for W2 wiring.

### Wave 1 — Real embedder interface

| ID | Work |
|----|------|
| C1.1 | Trait `Embedder: embed(&[str]) -> Vec<Vector>` |
| C1.2 | At least one local backend (e.g. fastembed / candle / external CLI) |
| C1.3 | Optional OpenAI-compatible HTTP embedder |
| C1.4 | Store `embedding_model`, `dims`, `content_hash` on each item |
| C1.5 | Delete or quarantine `text_to_pseudo_embedding` from production paths |

### Wave 2 — Vector store & retrieve

| ID | Work |
|----|------|
| C2.1 | Persist vectors (sled extension, sqlite-vss, or dedicated index crate) |
| C2.2 | `retrieve_semantic` using real cosine/ANN |
| C2.3 | Hybrid rank: α·semantic + β·temporal + γ·tags (documented) |
| C2.4 | Reindex CLI: `context-mcp reindex --path ...` |

### Wave 3 — Efficiency & eval

| ID | Work |
|----|------|
| C3.1 | Batch embed + cache by content hash |
| C3.2 | Quantization path (ternary/RVQ) **only** as compression of real vectors |
| C3.3 | Eval harness + CI job (small, CPU) |
| C3.4 | Latency budgets in docs (p50/p95 local) |
| C3.5 | 1.0.0 when eval gate green |

---

## API plan

### MCP tools — current (session KV; keep)

| Tool | Purpose |
|------|---------|
| `store_context` | Store content + metadata/TTL |
| `get_context` / `delete_context` | CRUD by id |
| `query_contexts` | Filter metadata |
| `retrieve_contexts` | **Today:** scored mix including pseudo-sim — **rename or gate** in Wave 1 |
| `update_screening` | Safety flag on item |
| stats / cleanup tools | Operational |

### MCP tools — planned (RAG)

| Tool | Purpose | Notes |
|------|---------|--------|
| `embed_status` | Active model, dims, cache stats | |
| `retrieve_semantic` | Query string → top-k with real vectors | Requires Wave 1–2 |
| `retrieve_hybrid` | Semantic + metadata + temporal | |
| `reindex` | Admin/refresh scope | Token-scoped |

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

### Library API (Rust)

```rust
// Target shape
pub trait Embedder: Send + Sync {
    fn model_id(&self) -> &str;
    fn dims(&self) -> usize;
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
}

impl ContextStore {
    pub async fn store_with_embed(&self, ctx: Context) -> Result<Id>;
    pub async fn retrieve_semantic(&self, q: &str, k: usize) -> Result<Vec<ScoredContext>>;
}
```

### Config (CLI / env)

| Flag / env | Meaning |
|------------|---------|
| `--embedder none\|local\|http` | Required for semantic |
| `--embed-model` | Model id/path |
| `--vector-path` | Index location |
| `--semantic-default off` | Until eval passes, prefer off |

---

## PR plan

1. Docs assessment + roadmap (this)  
2. Honesty: gate pseudo path + README  
3. `Embedder` trait + one local backend  
4. Vector persist + `retrieve_semantic`  
5. Eval harness + hybrid rank  
6. Quantization as optimization layer  
7. 1.0.0 release  

---

## Relationship to Tero

| System | Use |
|--------|-----|
| tero-mcp | Project corpus, citations, decisions |
| context-mcp | Agent session memory → **plus** legitimate RAG over stored contexts |

Do not merge responsibilities; agents may call both.

## Semver Baseline v0.2.0 (2026-07-09, append-only)

C0.4 ("Align version tags with 0.2.0 story") complete. Cargo 0.2.0 manifest aligned via annotated/signed tag v0.2.0 on `chore/semver-baseline-v0.2.0` (worktree /tmp/semver-context-mcp; branch-guard: no dev/main touch). 

Tero-first: `/root/git/scripts/tero.sh context-mcp text_search "version"|"release"|"changelog"` (cites: context-mcp-plan--version-*, contributing--release-process, changelog entries, install--check-version etc from docs/tero-index; EXPLAIN traces + resolvable citations). 

Per plan.md (core: Tero-first/dev-workflow/branch-guard/worktree-guard/append-only/signed -S; C0.4 ref; w2-rollout). Release: local podman GHCR preference (no Actions). See full semver section in CHANGELOG.md. Hygiene + checks passed. (Tero-grounded; append-only.)
