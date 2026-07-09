
# AGENTS.md — context-mcp

**Use Tero + cabal-devmelopner for work here.**

**Kickoff framework**: root `.claude/kickoffs/ctx.md` owns gaps/RAG work here. Fresh session `/kickoff ctx`. Per-repo `.claude/kickoffs/`. Apply dev-workflow + tero (categories) + honesty.

## Tero (Layer-1 corpus index)

Repo has `docs/tero-index/index.json` (generated/ refreshed via tero-mcp/scripts/generate_lite_index.py).

**Rule:** Use tero queries before large greps or assumptions.
- Grok: tero__text_search / query_by_id (token "local-dev")
- Direct: tero-mcp-lite --index docs/tero-index/index.json
- cabal-devmelopner: auto-detects local index when run from within this tree (or set TERO_INDEX_PATH).

Example:
```bash
cd /root/git/context-mcp
# agent with context:
uv run --project ../cabal-devmelopner cabal-devmelopner "task description here" --use-tero
```

Citations point at file:line — open them.

## Working with cabal-devmelopner agent tool

This project is prepared for integration:
- Tero index committed on chore/tero-index-cabal-ready (and PRable to dev)
- Local auto index support in cabal
- This AGENTS.md

**PR flow (protect main/dev):**
- Create/checkout feature or chore branch
- Make changes (agent will often use working branch)
- PR the branch → `dev` (then dev → main when ready)

## Local checks

Look for:
- scripts/check.sh
- Cargo.toml / pyproject.toml + standard commands (cargo test, uv run pytest, ruff, etc.)

Run checks before considering work complete.

## Further reading

- README.md
- docs/ROADMAP.md or ROADMAP.md (if present)
- docs/ASSESSMENT.md or similar for intent/gaps
- ../cabal-devmelopner/docs/* for agent architecture
- ../tero-mcp for how indexes are built and served

Leave mycelium isolated; all coordination here targets the other repos + cabal.

## Latest Updates (W2 Facade + Integration, 2026-07-09)

- C0 honesty gate complete (feature/ctx-c0-honesty): enable_semantic=false default (CLI --enable-semantic to opt-in; gated in RagProcessor + retrieve; pseudo only + warning when enabled). Docs/keywords/claims aligned (no false RAG). See docs/ROADMAP.md Wave 0 (C0 items marked done).
- PR #29: chore(tero): re-index after W2 facade + wsfull self-improving (on feature/ctx-c0-honesty). Includes C0 gate code+docs, tero reindex (503 items post facade), honesty updates.
- W2: Common memory facade integration (see dev-docs/schemas/ for StructuredResponse + common_memory_facade stubs; used with tero + context-mcp + memory-gate domains). Context-mcp provides session + future RAG (post gates).
- Docs (AGENTS.md, .claude/kickoffs/README.md, README.md, docs/ROADMAP.md, docs/ASSESSMENT.md) updated to latest compact: C0 gate, facade, W2, PR #29, tero reindex, wsfull-2026-07-09 state.
- Tero index updated post-docs + included; run via update-tero.sh.
- Kickoffs, agent context, claude files refreshed (root + per-repo).
- Part of wsfull wave: tero-first, hygiene, pr-review (adapted), commit+push, review, merge if clean.
- Use tero-first + cabal for work. Always: update docs + tero as part of PR process. Honesty: Empirical/Declared tags on claims.

