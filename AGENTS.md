
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
- Prefer PR → `dev` when `dev` is current with `main`; if `dev` is stale/diverged, PR → `main` (default branch; recent PRs #30–#35 landed on main).

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

**Post-merge W2/C0 + wsfull state (2026-07-09, after PR #29):** Merged (commit cdb7f14). C0 honesty gate complete (enable_semantic=false default, gated pseudo + warning), tero reindex (504 items), docs aligned. W2: common memory facade (refs wsfull-wave-2026-07-09-compact.md + dev-docs/schemas/ for StructuredResponse + common_memory_facade stubs in cabal + memory-gate). Verification review comment posted; checks green; tero-first cites confirm (AGENTS:56, ROADMAP:24/26, ASSESSMENT:80). Part of wsfull wave; propagate next. (Tero-grounded; append-only update.)

## W2: Session as Consumer (chore/w2-rollout-docs-wiring)

Context session noted as W2 consumer: session memory (temporal/context items) consumed via CommonMemoryAdapter (AgentDomain.CONTEXT) -> feeds StructuredResponse + MemoryContext in cabal agent (schemas/agent). Integrates tero + memory-gate domains for W2 rollout.

See: plan.md:44 w2-rollout, dev-docs/schemas/structured_response*.example + common_memory_facade.py.example (Context), cabal core/agent.py:79 (facade.query), context-mcp docs/ROADMAP (Wave 0 W2 refs), dev-mcp/servers/context-mcp.md (new W2 section), memory-gate-rs types.rs M1.

Append-only; tero cites to plan/wsfull; hygiene + update-tero; land --no-ff dev/main + propagate. Verify tero hits new sections.

(Note: a duplicate short semver note was appended here by the parallel bg semver loop while on dev; the baseline section lives on the dedicated chore/semver-baseline-v0.2.0 branch.)

## Health TLC (2026-07-16)

- WHAT: Gate-only hygiene on `origin/main` tip — crates.io keywords honesty (`rag` demoted/commented aspirational), `CHANGELOG` 0.2.0 baseline entry, `.gitignore` secrets block + safe `.gitallowed` (no broad `sk-*` allow), `scripts/check.sh` runs `git secrets --scan` when installed. Product epics #19 (real embeddings), #20 (GPU shaders), #21 (security-mcp) left open.
- WHY: Health TLC only; cargo gate already green (29 unit + 4 integration). Docs/semver/secrets honesty gaps were clear vs C0.
- WHY NOT: No mycelium/py2rust; no 1.0 bump; no hardware-hardcoded rayon threads; no product epic implementation.
- Gate: `./scripts/check.sh --quick` OK (fmt/clippy/doc/build/test + git-secrets). Python `test_mcp_server.py` is a stdio smoke harness (not pytest; pytest not required for Rust gate).
