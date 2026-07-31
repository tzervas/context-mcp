# Documentation index — context-mcp

## Project management (start here for status)

| Doc | Layer | Question it answers |
|-----|--------|---------------------|
| [DEVELOPMENT-PATH.md](DEVELOPMENT-PATH.md) | Back | How we got here; decisions and rejected alternatives |
| [CURRENT-STATE.md](CURRENT-STATE.md) | Back | What works **today**, measured (commands + output) |
| [ROADMAP.md](ROADMAP.md) | Back | Planned work and what would unblock each item |

## Product / gap notes (pre-existing)

| Doc | Role |
|-----|------|
| [ASSESSMENT.md](ASSESSMENT.md) | Gap analysis toward legitimate RAG (may lag crate version — prefer CURRENT-STATE for measured facts) |
| [LOCAL_CHECKS.md](LOCAL_CHECKS.md) | How to run `./scripts/check.sh` and optional Tero regen |
| [FLEET_STANDARDS.md](FLEET_STANDARDS.md) | Fleet workflow / badge / issue-close policy notes |
| [tero-index/](tero-index/) | Layer-1 corpus index artifacts for agent search |

## Root-level guides (operator-facing)

| Doc | Role |
|-----|------|
| [../README.md](../README.md) | Front layer: what it is, quickstart |
| [../INSTALL.md](../INSTALL.md) | Install and client wiring |
| [../USAGE_EXAMPLES.md](../USAGE_EXAMPLES.md) | Usage scenarios |
| [../CONTRIBUTING.md](../CONTRIBUTING.md) | Contribution notes |
| [../CHANGELOG.md](../CHANGELOG.md) | Version history |
| [../AGENTS.md](../AGENTS.md) | Agent / tero / cabal working notes |

## Historical / archival (treat as claims, not live status)

These may predate C0 honesty and Wave 1. Do not trust performance or “RAG complete” language without re-measurement.

- `ASSESSMENT_REPORT.md`, `BENCHMARK_*.md`, `ADVANCED_BENCHMARK_SUMMARY.md`
- `TERNARY_EMBEDDINGS_IMPLEMENTATION.md`, `CHANGELOG_TERNARY_EMBEDDINGS.md`
- `IMPLEMENTATION_COMPLETE.md`, `PR_DEPLOYMENT_SUMMARY.md`, `INTEGRATION_DEPLOYMENT_GUIDE.md`
- `RMCP_AUDIT_PLAN.md`, `context-mcp-plan.md`
