# Development tasks for context-mcp
# Install just: cargo install just

# Default recipe (shows available tasks)
default:
    @just --list

# === CODE QUALITY ===

# Run all code quality checks
check: fmt-check clippy test security audit licenses unused-deps outdated-deps

# Format code
fmt:
    cargo fmt

# Check formatting
fmt-check:
    cargo fmt --check

# Run clippy lints
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Run tests
test:
    cargo test --all-features --verbose

# Run tests with coverage (requires cargo-tarpaulin)
test-coverage:
    @echo "Coverage reporting requires manual setup. Consider using cargo-llvm-cov or grcov for coverage."

# === SECURITY SCANNING ===

# Run security audit
audit:
    cargo audit --ignore RUSTSEC-2025-0140 --ignore RUSTSEC-2025-0021 --ignore RUSTSEC-2025-0001 --ignore RUSTSEC-2025-0056 --ignore RUSTSEC-2025-0057 --ignore RUSTSEC-2024-0384

# Check licenses
licenses:
    cargo deny check licenses

# Check for security advisories
security:
    cargo deny check advisories

# Check for banned dependencies
bans:
    cargo deny check bans

# === DEPENDENCY ANALYSIS ===

# Check for unused dependencies
unused-deps:
    @echo "Use 'cargo machete' or similar tools for unused dependency detection"

# Check for outdated dependencies
outdated-deps:
    @echo "Use 'cargo update --dry-run' to check for outdated dependencies"

# Update dependencies
update-deps:
    cargo update

# === BENCHMARKING ===

# Run benchmarks
bench:
    cargo bench --all-features

# Run benchmarks with flamegraph (requires cargo-flamegraph)
bench-flamegraph bench_name:
    cargo flamegraph --bench {{bench_name}} --root --output target/flamegraph.svg

# === DOCUMENTATION ===

# Generate documentation
docs:
    cargo doc --all-features --open --no-deps

# Generate documentation with private items
docs-private:
    cargo doc --all-features --open --no-deps --document-private-items

# Check documentation
docs-check:
    RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps

# === BUILDING ===

# Build all targets
build:
    cargo build --all-targets --all-features

# Build release
build-release:
    cargo build --release --all-features

# Build examples
build-examples:
    cargo build --examples --all-features

# === CLEANING ===

# Clean build artifacts
clean:
    cargo clean

# Deep clean (including caches)
clean-deep: clean
    rm -rf target/

# === DEVELOPMENT WORKFLOW ===

# Full development cycle
dev: check build test bench docs

# Pre-commit checks
pre-commit: fmt clippy test security docs-check

# CI simulation (what runs in GitHub Actions)
ci: fmt-check clippy build test bench audit licenses

# === RELEASE PREPARATION ===

# Prepare for release
prepare-release: clean dev
    @echo "Ready for release!"

# Publish to crates.io (dry run)
publish-dry-run:
    cargo publish --dry-run --all-features

# === UTILITY ===

# Show project info
info:
    @echo "context-mcp development info:"
    @cargo --version
    @rustc --version
    @echo "Available features:"
    @cargo metadata --format-version 1 | jq -r '.packages[0].features | keys[]'

# Watch for changes and run tests
watch-test:
    cargo watch -x "test --all-features"

# Watch for changes and run clippy
watch-clippy:
    cargo watch -x clippy

# Install development dependencies
install-dev-deps:
    cargo install cargo-audit
    cargo install cargo-deny
    cargo install cargo-tarpaulin
    cargo install cargo-udeps
    cargo install cargo-outdated
    cargo install cargo-watch
    cargo install just