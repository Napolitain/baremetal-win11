# CI/CD Workflows

This directory contains GitHub Actions workflows for automated building, testing, and releasing of the smart-freeze application.

## Workflows

### 1. Build and Test (`build.yml`)

**Triggers:**
- Push to `main`, `master`, or `develop` branches
- Pull requests to `main`, `master`, or `develop` branches
- Manual trigger via GitHub Actions UI

**What it does:**
- Checks out the code
- Installs Rust toolchain with clippy and rustfmt
- Caches cargo dependencies for faster builds
- Runs code formatting check (`cargo fmt`)
- Runs linter (`cargo clippy`)
- Builds debug and release versions
- Runs tests
- Verifies the executable works (tests help command)
- Uploads build artifacts (30-day retention)

**Artifacts produced:**
- `smart-freeze-windows-x64` - The compiled executable
- `build-info` - Build metadata (commit, branch, date)

**Badge:**
```markdown
![Build Status](https://github.com/Napolitain/baremetal-win11/actions/workflows/build.yml/badge.svg)
```

---

### 2. Release (`release.yml`)

**Triggers:**
- Push of version tags (e.g., `v0.2.0`, `v1.0.0`)
- Manual trigger via GitHub Actions UI with version input

**What it does:**
- Builds release version
- Runs tests on release build
- Creates ZIP archive with executable, README, and LICENSE
- Generates SHA256 checksum
- Creates GitHub Release with auto-generated notes
- Attaches archive and checksum to release

**Creating a release:**

Using git tags:
```bash
git tag v0.2.0
git push origin v0.2.0
```

Manual via GitHub UI:
1. Go to Actions → Release
2. Click "Run workflow"
3. Enter version (e.g., v0.2.0)
4. Click "Run workflow"

**Artifacts produced:**
- `smart-freeze-vX.X.X-windows-x64.zip`
- `smart-freeze-vX.X.X-windows-x64.zip.sha256`

---

### 3. Security Audit (`security-audit.yml`)

**Triggers:**
- Weekly on Mondays at 9:00 AM UTC
- Push to `main`/`master` that modifies dependencies
- Manual trigger via GitHub Actions UI

**What it does:**
- Runs `cargo audit` to check for known security vulnerabilities
- Checks for outdated dependencies with `cargo outdated`
- Reports issues as workflow failures

**Security best practices:**
- Weekly automated scans
- Immediate scanning on dependency changes
- Proactive vulnerability detection

---

## Setup Requirements

### Repository Settings

1. **Actions Permissions:**
   - Settings → Actions → General
   - Ensure "Allow all actions and reusable workflows" is enabled

2. **Workflow Permissions:**
   - Settings → Actions → General → Workflow permissions
   - Enable "Read and write permissions" for releases
   - Check "Allow GitHub Actions to create and approve pull requests"

3. **Required Secrets:**
   - `GITHUB_TOKEN` is automatically provided by GitHub Actions
   - No additional secrets needed for basic functionality

### Branch Protection (Recommended)

Settings → Branches → Add rule for `main`:
- ☑ Require status checks to pass before merging
- ☑ Require branches to be up to date before merging
- Select: `build-windows` status check

---

## Local Testing

Before pushing, test locally:

```bash
# Format check
cargo fmt -- --check

# Linting
cargo clippy --all-targets --all-features -- -D warnings

# Build both versions
cargo build
cargo build --release

# Run tests
cargo test

# Security audit
cargo install cargo-audit
cargo audit
```

---

## Monitoring

### Build Status

View workflow runs:
- Repository → Actions tab
- Click on specific workflow
- View logs for each step

### Artifacts

Download artifacts:
- Actions → Successful workflow run
- Scroll to "Artifacts" section
- Download `smart-freeze-windows-x64`

### Releases

View releases:
- Repository → Releases (right sidebar)
- Each release includes:
  - Auto-generated changelog
  - ZIP archive
  - SHA256 checksum

---

## Troubleshooting

### Build Fails on Formatting

```bash
# Fix locally
cargo fmt

# Commit and push
git add .
git commit -m "Fix formatting"
git push
```

### Clippy Warnings Fail Build

The workflow uses `-D warnings` which treats warnings as errors.

```bash
# Fix locally
cargo clippy --fix --allow-dirty

# Or manually fix the issues, then:
git add .
git commit -m "Fix clippy warnings"
git push
```

### Cache Issues

If builds are slow or behave oddly:
1. Go to Actions → Caches
2. Delete old caches
3. Re-run the workflow

### Release Fails

Common issues:
- Tag already exists: Delete old tag first
- Permissions: Check workflow permissions in settings
- Build fails: Ensure tests pass locally first

---

## Workflow Customization

### Change Build Frequency

Edit trigger sections:

```yaml
on:
  push:
    branches: [ main ]  # Remove develop if not needed
```

### Add More Tests

Add steps in `build.yml`:

```yaml
- name: Integration tests
  run: cargo test --test integration_tests
```

### Change Artifact Retention

Modify retention days:

```yaml
retention-days: 7  # Instead of 30
```

---

## Performance

### Build Times

Typical times on `windows-latest`:
- Fresh build: ~6 minutes
- Cached build: ~2 minutes
- Clippy check: ~1 minute
- Tests: ~30 seconds

### Cache Effectiveness

Caches speed up builds by:
- Cargo registry: ~1 minute saved
- Cargo index: ~30 seconds saved
- Target directory: ~2 minutes saved

**Total savings: ~3.5 minutes per build with cache**

---

## Future Enhancements

Potential workflow additions:

1. **Code Coverage:**
   ```yaml
   - uses: actions-rs/tarpaulin@v0.1
   ```

2. **Cross-platform Builds:**
   ```yaml
   strategy:
     matrix:
       os: [windows-2019, windows-2022]
   ```

3. **Benchmark Tracking:**
   ```yaml
   - name: Run benchmarks
     run: cargo bench
   ```

4. **Dependency Updates (Dependabot):**
   Create `.github/dependabot.yml`

5. **Scheduled Builds:**
   ```yaml
   schedule:
     - cron: '0 0 * * 0'  # Weekly
   ```

---

## Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust GitHub Actions](https://github.com/actions-rs)
- [dtolnay/rust-toolchain](https://github.com/dtolnay/rust-toolchain)
- [softprops/action-gh-release](https://github.com/softprops/action-gh-release)
