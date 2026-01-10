#!/bin/bash
# Development environment setup script for context-mcp
# Run this to set up your local development environment

set -e

echo "Setting up development environment for context-mcp..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Rust not found. Please install Rust from https://rustup.rs/"
    exit 1
fi

echo "Installing development tools..."

# Install cargo tools
cargo install just --locked
cargo install cargo-audit --locked
cargo install cargo-deny --locked
cargo install cargo-tarpaulin --locked
cargo install cargo-udeps --locked
cargo install cargo-outdated --locked
cargo install cargo-watch --locked

# Optional tools (commented out by default)
# cargo install cargo-flamegraph --locked  # For profiling
# cargo install cargo-expand --locked      # For macro expansion

echo "Setting up git hooks..."
git config core.hooksPath .githooks

echo "Development environment setup complete!"
echo ""
echo "Available commands:"
echo "  just                # Show all available tasks"
echo "  just check          # Run all quality checks"
echo "  just test           # Run tests"
echo "  just bench          # Run benchmarks"
echo "  just docs           # Generate documentation"
echo "  just security       # Run security checks"
echo ""
echo "Happy coding! 🚀"