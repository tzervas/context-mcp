#!/usr/bin/env bash
set -euo pipefail

# CI hosts (esp. cgroup 8G no-swap): skip cargo bench — it is not a unit gate and
# doubles peak RSS after clippy+test. Full local: ./scripts/check.sh without CI env.
if [[ "${GITHUB_ACTIONS:-}" == "true" ]]; then
  export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-1}"
  export CARGO_PROFILE_DEV_DEBUG="${CARGO_PROFILE_DEV_DEBUG:-0}"
  export CARGO_PROFILE_TEST_DEBUG="${CARGO_PROFILE_TEST_DEBUG:-0}"
  # Full audit/deny/bench + --all-features (wgpu) + cargo-audit-as-dev-dep
  # pulls huge trees and OOMs 8G no-swap runners (exit 137/143/255).
  # Local full gate remains: ./scripts/check.sh (no GITHUB_ACTIONS).
  export CONTEXT_MCP_CI_QUICK="${CONTEXT_MCP_CI_QUICK:-1}"
fi
cd "$(dirname "$0")/.."
MODE="${1:-}"
export CARGO_TERM_COLOR="${CARGO_TERM_COLOR:-always}"
export RUST_BACKTRACE="${RUST_BACKTRACE:-1}"
TOOLCHAIN="${RUSTUP_TOOLCHAIN:-stable}"
CARGO=(cargo)
if command -v rustup >/dev/null 2>&1; then
  rustup component add rustfmt clippy --toolchain "$TOOLCHAIN" >/dev/null 2>&1 || true
  CARGO=(cargo "+$TOOLCHAIN")
fi
AUDIT_IGNORES=(
  --ignore RUSTSEC-2025-0140
  --ignore RUSTSEC-2025-0021
  --ignore RUSTSEC-2025-0001
  --ignore RUSTSEC-2025-0056
  --ignore RUSTSEC-2025-0057
  --ignore RUSTSEC-2024-0384
)

QUICK=0
if [[ "$MODE" == "--quick" || "${CONTEXT_MCP_CI_QUICK:-}" == "1" ]]; then
  QUICK=1
fi

if [[ "$MODE" == "--fix" ]]; then
  "${CARGO[@]}" fmt
else
  "${CARGO[@]}" fmt --check
fi

if [[ "$QUICK" -eq 1 ]]; then
  # Causal OOM mitigations (class C5):
  # 1) default features only — --all-features enables gpu-acceleration/wgpu
  # 2) no benches — criterion + bench targets double peak RSS
  # 3) cargo-audit is no longer a dev-dep (gix tree)
  echo "CONTEXT_MCP_CI_QUICK=1: clippy/build/test on default features (no benches/gpu)"
  "${CARGO[@]}" clippy --lib --bins --tests --examples -- -D warnings
  RUSTDOCFLAGS="-D warnings" "${CARGO[@]}" doc --no-deps
  "${CARGO[@]}" build
  "${CARGO[@]}" test --verbose
else
  "${CARGO[@]}" clippy --all-targets --all-features -- -D warnings
  RUSTDOCFLAGS="-D warnings" "${CARGO[@]}" doc --all-features --no-deps
  "${CARGO[@]}" build --all-features
  "${CARGO[@]}" test --all-features --verbose
fi

# Secrets scan (optional tool; fail closed when installed)
if command -v git-secrets >/dev/null 2>&1; then
  git secrets --scan
else
  echo "WARN: git-secrets not installed; skip secrets scan"
fi

if [[ "$QUICK" -eq 0 ]]; then
  if cargo audit -V >/dev/null 2>&1; then
    cargo audit "${AUDIT_IGNORES[@]}"
  else
    echo "WARN: cargo-audit not installed; skip"
  fi
  if cargo deny -V >/dev/null 2>&1; then
    cargo deny check licenses
    cargo deny check advisories
    cargo deny check bans
  else
    echo "WARN: cargo-deny not installed; skip"
  fi
  # Criterion warmup + release rebuild peaks past self-hosted cgroup/host memory when
  # other agents share the box; GHA sets CI=true. Skip benches in CI (still available
  # locally via plain `./scripts/check.sh`). Use --quick to skip audit/deny/bench entirely.
  if [[ "${CI:-}" == "true" ]]; then
    echo "CI=true: skip cargo bench (avoid OOM/SIGTERM under fleet memory pressure)"
  else
    "${CARGO[@]}" bench --all-features
  fi
fi
echo "OK: context-mcp checks passed"
