# CI/CD Quick Start Guide

Get up and running with automated builds and releases in 5 minutes.

## Prerequisites

- GitHub repository with this code pushed
- GitHub Actions enabled (default for all repos)

## Step 1: Enable GitHub Actions

1. Go to your repository on GitHub
2. Click on **Actions** tab
3. If prompted, click **I understand my workflows, go ahead and enable them**

That's it! Workflows are now enabled.

## Step 2: Verify Build Workflow

After pushing code:

1. Go to **Actions** tab
2. You should see **Build and Test** workflow running
3. Wait for it to complete (typically 2-6 minutes)
4. Green checkmark = success ✅

## Step 3: Download Build Artifacts

After a successful build:

1. Click on the completed workflow run
2. Scroll to **Artifacts** section at the bottom
3. Download `smart-freeze-windows-x64`
4. Extract and run the executable

## Step 4: Create Your First Release

### Option A: Using Git Tags (Recommended)

```bash
# Create and push a tag
git tag v0.2.0
git push origin v0.2.0

# GitHub Actions will automatically:
# 1. Build the release
# 2. Run tests
# 3. Create ZIP archive
# 4. Generate checksum
# 5. Create GitHub Release
```

### Option B: Manual Trigger

1. Go to **Actions** → **Release** workflow
2. Click **Run workflow**
3. Enter version: `v0.2.0`
4. Click **Run workflow**

### View Your Release

1. Go to repository main page
2. Click **Releases** (right sidebar)
3. Your release is published with:
   - Auto-generated changelog
   - ZIP download
   - SHA256 checksum

## Step 5: Monitor Security

The security audit runs automatically:
- Every Monday at 9 AM UTC
- Whenever you change dependencies
- Can be triggered manually from Actions tab

Check results:
1. **Actions** → **Security Audit**
2. View latest run
3. Check for any vulnerabilities

## Common Tasks

### Run Build Manually

1. **Actions** → **Build and Test**
2. Click **Run workflow**
3. Select branch
4. Click **Run workflow**

### Fix Formatting Issues

If build fails on formatting:

```bash
cargo fmt
git add .
git commit -m "Fix formatting"
git push
```

### Fix Clippy Warnings

If build fails on linting:

```bash
cargo clippy --fix --allow-dirty
git add .
git commit -m "Fix clippy warnings"
git push
```

## Status Badges

Add to your README:

```markdown
[![Build Status](https://github.com/YOUR_USERNAME/baremetal-win11/actions/workflows/build.yml/badge.svg)](https://github.com/YOUR_USERNAME/baremetal-win11/actions/workflows/build.yml)
```

Replace `YOUR_USERNAME` with your GitHub username.

## Troubleshooting

### Workflow Doesn't Run

**Check:**
- Actions are enabled (Settings → Actions)
- `.github/workflows/*.yml` files exist
- You pushed to `main`, `master`, or `develop` branch

### Build Fails

**Check workflow logs:**
1. Actions → Failed workflow
2. Click on the red X
3. Expand failed step
4. Read error message

**Common issues:**
- Formatting: Run `cargo fmt`
- Clippy warnings: Run `cargo clippy --fix`
- Tests failing: Run `cargo test` locally

### Release Doesn't Create

**Check:**
- Tag format is `vX.Y.Z` (e.g., `v0.2.0`)
- Tag was pushed to GitHub: `git push origin v0.2.0`
- Workflow permissions enabled (Settings → Actions → General)

## Next Steps

- **Read full documentation**: [.github/workflows/README.md](workflows/README.md)
- **Configure branch protection**: Require status checks before merge
- **Set up notifications**: Get alerts for failed builds
- **Add more tests**: Expand test coverage

## Support

- Check workflow logs for detailed error messages
- Review [GitHub Actions documentation](https://docs.github.com/en/actions)
- Open an issue if you encounter problems

---

**That's it!** Your CI/CD pipeline is now fully operational. Every push is automatically built and tested, and releases are just one `git tag` away.
