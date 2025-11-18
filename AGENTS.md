# Agent Guidelines for SmartFreeze Development

This document provides guidelines for AI agents (like GitHub Copilot, Claude, ChatGPT) working on this codebase.

## ⚠️ Critical: Always Run Full CI/CD Pipeline

Before considering any code changes complete, **you MUST run all CI/CD steps locally** to ensure correctness.

### Required Steps (in order):

```bash
# 1. Check formatting
cargo fmt -- --check

# 2. Run linter
cargo clippy --all-targets --all-features -- -D warnings

# 3. Build release binary
cargo build --release --verbose

# 4. Run all tests
cargo test --verbose

# 5. Verify binary works
.\target\release\smart-freeze.exe --help
```

### Why This Matters

- **Formatting issues** will cause CI/CD to fail
- **Clippy warnings** indicate potential bugs or non-idiomatic code
- **Build failures** mean the code won't compile for users
- **Test failures** indicate broken functionality
- **Binary verification** ensures the executable actually works

### Before Committing

Run the complete pipeline:

```bash
# Auto-fix formatting
cargo fmt

# Check everything
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release --verbose
cargo test --verbose
.\target\release\smart-freeze.exe --help
```

If any step fails, **do not commit**. Fix the issues first.

## CI/CD Pipeline Overview

### Build Workflow (`.github/workflows/build.yml`)

Runs on every push and PR to `main`, `master`, `develop`:

1. **Check formatting** - Enforces consistent code style
2. **Run clippy** - Static analysis for bugs and non-idiomatic code
3. **Build release** - Ensures release binary compiles
4. **Run tests** - Verifies all functionality works
5. **Test binary** - Ensures executable runs correctly
6. **Upload artifacts** - Makes binary available for download

### Daily Release Workflow (`.github/workflows/daily-release.yml`)

Runs daily at 00:00 UTC (or manually):

1. **Check for new commits** - Skips if no changes
2. **Generate CalVer tag** - Format: YYYY.MM.DD-n
3. **Build and test** - Same as build workflow
4. **Create release archive** - ZIP with binary, README, LICENSE
5. **Generate checksums** - SHA256 for verification
6. **Create GitHub release** - Automated release notes
7. **Create Scoop PR** - Automated manifest update

### Security Audit (`.github/workflows/security-audit.yml`)

Runs weekly and on dependency changes:

1. **cargo-audit** - Checks for known vulnerabilities
2. **cargo-deny** - Enforces dependency policies

## Code Quality Standards

### Formatting

- **Tool**: `rustfmt` with default settings (see `rustfmt.toml`)
- **Rule**: Zero tolerance for formatting issues
- **Fix**: Run `cargo fmt` before committing

### Linting

- **Tool**: `clippy` with all warnings as errors
- **Flags**: `--all-targets --all-features -- -D warnings`
- **Fix**: Address all clippy warnings before committing

### Testing

- **Coverage**: Aim for >90% (currently 94%)
- **Location**: Tests in same file as implementation or `tests/` directory
- **Run**: `cargo test --verbose`
- **New features**: Must include tests

## Common Issues and Fixes

### Formatting Failed

```bash
# Fix automatically
cargo fmt

# Verify
cargo fmt -- --check
```

### Clippy Warnings

```bash
# Show all warnings
cargo clippy --all-targets --all-features

# Common fixes:
# - Use #[allow(clippy::lint_name)] for false positives
# - Refactor code to address legitimate warnings
# - Never use #![allow(clippy::all)]
```

### Build Failed

```bash
# Clean and rebuild
cargo clean
cargo build --release --verbose

# Check for missing dependencies
cargo update
```

### Tests Failed

```bash
# Run with detailed output
cargo test --verbose -- --nocapture

# Run specific test
cargo test test_name -- --nocapture

# Run tests in single thread (for debugging)
cargo test -- --test-threads=1
```

## Making Changes

### Small Changes (< 50 lines)

1. Make changes
2. Run full CI/CD pipeline locally
3. Commit if all steps pass

### Medium Changes (50-200 lines)

1. Make changes
2. Run `cargo fmt`
3. Run `cargo clippy --fix --all-targets --all-features`
4. Run full CI/CD pipeline locally
5. Review all changes
6. Commit if all steps pass

### Large Changes (> 200 lines)

1. Break into smaller logical commits
2. For each commit:
   - Make changes
   - Run `cargo fmt`
   - Run `cargo clippy --fix --all-targets --all-features`
   - Run full CI/CD pipeline locally
   - Commit only if all steps pass
3. Test the entire change set together

## Architecture Principles

### SOLID Design

- **Single Responsibility**: Each module has one clear purpose
- **Open/Closed**: Extend via traits, not modification
- **Liskov Substitution**: Trait implementations are interchangeable
- **Interface Segregation**: Small, focused traits
- **Dependency Inversion**: Depend on abstractions (traits)

### Module Structure

```
src/
├── lib.rs              # Public API + error types
├── main.rs             # CLI entry point (keep small!)
├── cli.rs              # Argument parsing
├── process.rs          # Data structures
├── categorization.rs   # Process categorization
├── freeze_engine.rs    # Core engine (DI)
├── persistence.rs      # State management
├── output/             # Output formatters
└── windows/            # Windows-specific code
```

### Testing Strategy

- **Unit tests**: In same file as implementation
- **Integration tests**: In `tests/` directory
- **Mock via traits**: Dependency injection for testability
- **Coverage**: Use `cargo tarpaulin` locally

## Release Process

### Manual Testing Before Release

Even if CI passes, manually test:

```bash
# Build release binary
cargo build --release

# Test daemon mode
.\target\release\smart-freeze.exe --daemon

# Test manual freeze/resume
.\target\release\smart-freeze.exe --action freeze --pid <PID>
.\target\release\smart-freeze.exe --action resume --pid <PID>

# Test Windows startup
.\target\release\smart-freeze.exe --install-startup
.\target\release\smart-freeze.exe --uninstall-startup

# Test output formats
.\target\release\smart-freeze.exe --format json
.\target\release\smart-freeze.exe --format csv
```

### Triggering a Release

Releases are automatic (daily) but can be triggered manually:

1. Go to Actions → Daily CalVer Release
2. Click "Run workflow"
3. Wait for completion
4. Verify:
   - GitHub release created
   - ZIP archive with SHA256
   - PR opened in `Napolitain/scoop` repository
5. Merge Scoop PR manually

## Documentation

### Keep Updated

When making changes, update:

- `README.md` - User-facing features
- `CHANGELOG.md` - Version history
- `DAEMON_MODE.md` - Daemon-specific docs
- `QUICK_START.md` - Getting started guide
- `CONTRIBUTING.md` - Development guide
- This file (`AGENTS.md`) - Process changes

### Code Comments

- **Public APIs**: Rustdoc comments (`///`)
- **Complex logic**: Inline comments explaining "why"
- **Modules**: Module-level docs (`//!`)
- **No obvious comments**: Don't comment self-explanatory code

## Git Workflow

### Commit Messages

Format: `<type>: <description>`

Types:
- `feat:` - New feature
- `fix:` - Bug fix
- `refactor:` - Code restructuring
- `docs:` - Documentation only
- `test:` - Test additions/changes
- `chore:` - Maintenance (deps, CI, etc.)

Example:
```
feat: add JSON output format support

- Implement JsonFormatter trait
- Add --format json CLI flag
- Include tests for JSON serialization
```

### Branches

- `main` - Stable, production-ready code
- `develop` - Development branch (if using)
- `feature/*` - New features
- `fix/*` - Bug fixes

## Security

### Dependencies

- Run `cargo audit` regularly
- Update dependencies: `cargo update`
- Review changes in `Cargo.lock`

### Secrets

- Never commit secrets (tokens, passwords)
- Use GitHub Secrets for CI/CD
- Rotate PAT tokens regularly

## Performance

### Benchmarking

```bash
# Build with optimizations
cargo build --release

# Time critical operations
cargo bench

# Profile with perf (Linux) or Windows Performance Analyzer
```

### Memory Usage

- Daemon should use <10 MB when idle
- Process enumeration should take <10ms
- Freeze/resume should take <1ms per process

## Questions?

If you're an AI agent and unsure about something:

1. Check this file first
2. Check `CONTRIBUTING.md`
3. Review existing code patterns
4. Ask the human developer

## Summary: Before Every Commit

```bash
# The golden pipeline
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release --verbose
cargo test --verbose
.\target\release\smart-freeze.exe --help

# If all pass → commit
# If any fail → fix and repeat
```

**No exceptions.** This ensures code quality and prevents CI/CD failures.
