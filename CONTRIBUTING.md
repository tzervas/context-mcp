# Contributing to context-mcp

Thank you for your interest in contributing! This document outlines the development process and standards for the context-mcp project.

## Development Setup

### Prerequisites

- Rust 1.75+ (MSRV - Minimum Supported Rust Version)
- Git

### Quick Start

1. **Clone and setup**:
   ```bash
   git clone https://github.com/tzervas/context-mcp.git
   cd context-mcp
   ./setup-dev.sh
   ```

2. **Verify setup**:
   ```bash
   just check
   ```

### Development Tools

This project uses [just](https://github.com/casey/just) for task automation. Key commands:

```bash
just                    # Show all available tasks
just check             # Run all quality checks
just test              # Run tests
just bench             # Run benchmarks
just docs              # Generate documentation
just dev               # Full development cycle
```

## Development Workflow

### 1. Choose an Issue

- Check [GitHub Issues](https://github.com/tzervas/context-mcp/issues) for open tasks
- Comment on the issue to indicate you're working on it
- Create a feature branch: `git checkout -b feature/your-feature-name`

### 2. Make Changes

- Write code following the established patterns
- Add tests for new functionality
- Update documentation as needed
- Run quality checks frequently: `just check`

### 3. Pre-Commit Checks

Before committing, ensure all checks pass:

```bash
just pre-commit
```

This runs:
- Code formatting check
- Clippy linting
- Tests
- Security scanning
- Documentation checks

### 4. Commit and Push

- Use descriptive commit messages
- Push your branch to GitHub
- Create a pull request

## Code Quality Standards

### Formatting
- Use `cargo fmt` for consistent formatting
- Formatting is automatically checked in CI

### Linting
- No clippy warnings allowed (`cargo clippy -- -D warnings`)
- CI will fail if warnings are present

### Testing
- Add unit tests for new functionality
- Integration tests for complex features
- All tests must pass in CI

### Documentation
- Update doc comments for public APIs
- Add examples for new features
- Documentation warnings are treated as errors

### Security
- Dependencies are scanned for vulnerabilities
- Only approved licenses are allowed
- Security issues will block releases

## Pull Request Process

1. **Create PR**: Push your branch and create a pull request on GitHub
2. **Description**: Provide a clear description of changes
3. **CI Checks**: Ensure all CI checks pass
4. **Review**: Address reviewer feedback
5. **Merge**: PRs are merged using squash commits

### PR Checklist

- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] Documentation updated
- [ ] Security checks pass
- [ ] Benchmarks still perform well
- [ ] Commit messages are clear

## Release Process

Releases are automated via GitHub Actions:

1. Version is bumped in `Cargo.toml`
2. Tag is created: `git tag v0.x.y`
3. CI runs comprehensive checks
4. Release is published to crates.io
5. GitHub release is created

## Getting Help

- **Issues**: Report bugs or request features on GitHub
- **Discussions**: Ask questions in GitHub Discussions
- **Documentation**: Check docs.rs for API documentation

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
