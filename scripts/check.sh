#!/usr/bin/env bash
set -euo pipefail
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

if [[ "$MODE" == "--fix" ]]; then
  "${CARGO[@]}" fmt
else
  "${CARGO[@]}" fmt --check
fi
"${CARGO[@]}" clippy --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" "${CARGO[@]}" doc --all-features --no-deps
"${CARGO[@]}" build --all-features
"${CARGO[@]}" test --all-features --verbose

# Secrets scan (optional tool; fail closed when installed)
if command -v git-secrets >/dev/null 2>&1; then
  git secrets --scan
else
  echo "WARN: git-secrets not installed; skip secrets scan"
fi

if [[ "$MODE" != "--quick" ]]; then
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
  "${CARGO[@]}" bench --all-features
fi
echo "OK: context-mcp checks passed"
