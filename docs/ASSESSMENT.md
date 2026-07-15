# context-mcp — Assessment & Gap Analysis

**Date:** 2026-07-08  
**Crate:** `context-mcp` 0.2.0 (Rust)  
**Role today:** Agent **session / runtime memory** MCP + library  
**Role required:** Efficient **legitimate RAG** (product need) — not yet met  
**Complements:** `tero-mcp` (corpus L1 citations) ≠ this (session + future RAG)

---

## 1. What works today

| Capability | Reality |
|------------|---------|
| Store / get / delete / query by metadata | Yes — MCP tools + `ContextStore` |
| Temporal tags, TTL, importance, domains | Yes |
| In-memory LRU | Yes |
| Optional disk (`sled`, feature `persistence`) | Yes — **KV of records**, not vector DB |
| stdio + HTTP transports | Yes (WS overclaimed) |
| “Semantic” retrieve | **Pseudo only** — see §2 |

---

## 2. Critical gap: not legitimate RAG

**Code path** (`src/rag.rs`): `text_to_pseudo_embedding` — word hash + sin → 64-d → cosine. Explicitly demo-only; **no real semantic meaning**.

**Quantization / ternary / RVQ:** efficiency layers that need **real vectors first**. They do not replace an embedder.

**Compared to legitimate RAG:**

| Legitimate RAG | context-mcp today |
|----------------|-------------------|
| Real embedding model | Hash pseudo-vectors |
| Vector index (ANN/exact) | LRU + sled KV |
| Chunk pipeline + eval | Metadata filters + fake similarity |
| Honest “semantic” scores | Can mislabel pseudo as semantic |

**Maintainer product need:** efficient legitimate RAG is **required** for this project’s destination — not optional polish.

---

## 3. Maturity

| Dimension | Score | Notes |
|-----------|-------|--------|
| Session KV MCP | **3–4** | Usable sidecar |
| Packaging / install | **3** | cargo install / scripts |
| Auth | **1** | Missing on HTTP |
| Real RAG | **1** | Blocked on embedder + vector store + eval |
| Doc honesty | **2** | Some good “not” sections; keywords/changelog overclaim RAG |
| Versioning | **2** | Tree 0.2.0 vs tags ~0.1.6 |

---

## 4. Branches

| Branch | Notes |
|--------|--------|
| `main`/`dev`/`integration` | Aligned |
| `claude/finish-context-mcp` | Honesty/gitignore polish — review |
| `claude/fix-deps` | Likely merged-equivalent advisory work |

---

## 5. Integration (cabal-devmelopner)

| Phase | Use |
|-------|-----|
| Now | Optional **session store** only; cabal defaults to JSONL transcripts |
| After RAG exit criteria | Optional **RAG backend** for session/project context |
| Never | Claim RAG while pseudo path is live in semantic mode |

See [ROADMAP.md](ROADMAP.md) for definition of done and API plan.

## Tero index

Layer-1 citation index: [docs/tero-index/](tero-index/) (`index.json`, `INDEX.md`, `MANIFEST.toml`).

## C0 Honesty Pass (complete, PR #29 / feature/ctx-c0-honesty)

- Docs claims aligned (README, src docs, Cargo keywords reviewed; "rag" now aspirational note).
- Semantic gated (enable_semantic=false default in RagConfig + CLI; retrieve_contexts uses only metadata/temporal/keyword unless --enable-semantic; pseudo + warning when on. See rag.rs, tools.rs, main.rs).
- WS/SSE note added.
- No "real RAG" claims.
- C0 items (C0.1-C0.2, C0.5) done; C0.3/4 scoped/deferred.
- Tero reindex included (503 items).
- Part of wsfull-2026-07-09 + W2 facade (common memory facade integration; see dev-docs/schemas + cabal). Next per ROADMAP: Wave 1 embedder.
- Verified via tero text_search ("C0", "honesty", "facade", "wave"), hygiene, checks. (See root ctx.md + docs/ROADMAP.md + compact)

**Post-merge W2/C0 + wsfull state (2026-07-09):** PR #29 landed+merged. C0 honesty pass complete. Tero reindex included. W2 common memory facade integration referenced (wsfull-wave-2026-07-09-compact.md + cabal facade in dev-docs/schemas). Post-merge verification review comment + update-tero.sh + dev propagation per swarm task. All tero-grounded, checks clean.
