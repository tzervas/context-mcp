# Fleet standards (tzervas)

Applied from the workstation pack under `plans/fleet-standards/pack/`.

## Workflows

| Workflow | When | Runner |
|----------|------|--------|
| `fleet-ci.yml` | push/PR to main|dev | `[self-hosted, linux, x64, podman, compute-cpu, host-homelab, scribe-cpu-build]` |
| `fleet-security.yml` | push/PR + weekly | same |
| `ci.yml` / product CPU | push/PR to main|dev | same (homelab CPU; **not** WSL) |
| `close-issues-on-main.yml` | PR closed→main | same |
| `reopen-issues-closed-off-main.yml` | PR merged off-main with Closes | same |

WSL is GPU-only. Bare `[self-hosted, linux, x64, podman]` can match a WSL CPU listener — do not use that short set for product or meta jobs.

## Issue close policy

- **`dev` / feature merges:** `Refs #n` only — issues stay open
- **`main` merges:** `Closes #n` / `Fixes #n`
- **Epics:** close only when delivery PR to main includes `Closes #<epic>`

## Badges

README badges use GitHub Actions SVG for **trunk** branch — live status, not static green.

## Copilot

Automatic Copilot code reviews are **disabled** for fleet-managed repos. Do not request Copilot on PRs.

## Permissions

Workflows use minimum `permissions:` blocks (contents read; issues write only for close/reopen jobs).
