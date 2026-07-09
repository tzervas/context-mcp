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
