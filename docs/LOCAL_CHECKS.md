# Local checks (CI parity)

GitHub Actions workflows in this repo are **manual only** (`workflow_dispatch`).
Day-to-day quality gates run **locally** so remote CI is not the only source of truth.

## Run everything the remote job would run

```bash
./scripts/check.sh
```

Optional:

```bash
./scripts/check.sh --quick   # skip slower steps (bench/audit when applicable)
./scripts/check.sh --fix  # apply formatters instead of --check
```

## Tero index

```bash
# from a checkout that can see the generator (sibling tero-mcp recommended):
python3 ../tero-mcp/scripts/generate_lite_index.py --root "$(pwd)"
# or:
python3 scripts/generate_tero_index.sh   # if present as a thin wrapper
```

Artifacts land in `docs/tero-index/` (`index.json`, `INDEX.md`, `MANIFEST.toml`, `README.md`).

## Remote (optional)

In GitHub: **Actions → CI → Run workflow**.

## Semver Baseline v0.2.0 (2026-07-09, append-only)

Context for release process in semver baseline: local podman GHCR preference (no Actions) per chore/semver-baseline-v0.2.0. See CHANGELOG.md [0.2.0] Semver Baseline (Tero cites from /root/git/scripts/tero.sh context-mcp text_search, plan.md, dev-workflow). Hygiene: always run `./scripts/check.sh` locally. (Append-only; tero-first; branch/worktree guards.)
