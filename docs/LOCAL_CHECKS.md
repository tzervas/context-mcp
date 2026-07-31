# Local checks (CI parity)

Day-to-day quality gates should be run **locally** so remote CI is not the only source of truth.

Remote GitHub Actions in this repo run on **push and pull_request** for the fleet workflows (`ci.yml`, `fleet-ci.yml`, `fleet-security.yml`), plus `workflow_dispatch` on several workflows. They are **not** manual-only. Treat CI conclusions as informative; this repo has historically had **zero required status checks** on `main` — a green or missing check is not automatic proof of merge safety. Prefer `./scripts/check.sh` before declaring work done.

Measured snapshot of gates: [CURRENT-STATE.md](CURRENT-STATE.md).

## Run everything the remote job would run

```bash
./scripts/check.sh
```

Optional:

```bash
./scripts/check.sh --quick   # skip slower steps (bench/audit when applicable)
./scripts/check.sh --fix  # apply formatters instead of --check
```

Resource note for shared builders: set `CARGO_BUILD_JOBS=3` (or similar) so unbounded cargo parallelism does not OOM multi-tenant hosts.

## Tero index

```bash
# from a checkout that can see the generator (sibling tero-mcp recommended):
python3 ../tero-mcp/scripts/generate_lite_index.py --root "$(pwd)"
# or:
python3 scripts/generate_tero_index.sh   # if present as a thin wrapper
```

Artifacts land in `docs/tero-index/` (`index.json`, `INDEX.md`, `MANIFEST.toml`, `README.md`).

## Remote (optional)

In GitHub: **Actions** → select workflow → **Run workflow**, or rely on push/PR triggers for fleet-ci / fleet-security / CI.
