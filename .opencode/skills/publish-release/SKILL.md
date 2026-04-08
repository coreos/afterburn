---
name: publish-release
description: Guide the post-merge release process - release branch, tag, vendor archive, and GitHub release
---

# Publish Release

Guide the post-merge release process for Afterburn. This skill runs automatable steps directly and prompts the user for steps requiring credentials (GPG signing, crates.io publishing).

## What it does

1. Verifies the pre-release PR has been merged
2. Runs clean build verification with vendored dependencies
3. Creates the release branch and guides user through `cargo release`
4. Opens the release PR
5. After merge, pushes the tag and guides user through `cargo publish`
6. Creates the vendor archive with SHA256 digests
7. Drafts the GitHub release body
8. Cleans up local and remote branches

## Prerequisites

- The pre-release PR for this version must already be merged to `main`
- `cargo-release` installed (`cargo install cargo-release`)
- `cargo-vendor-filterer` installed (`cargo install cargo-vendor-filterer`)
- GPG key configured for commit/tag signing
- crates.io account with publish access
- `gh` CLI installed and authenticated

## Usage

```bash
/publish-release 5.11.0
/publish-release 5.11.0 --remote origin
```

## Workflow

This is a multi-phase, interactive workflow. The skill runs what it can automatically and prompts the user at credential-gated steps. Track progress with a checklist displayed after each phase.

### Step 1: Gather Inputs

Get the release version from arguments. If not provided, ask the user.

Set `UPSTREAM_REMOTE` from `--remote` flag, defaulting to `origin`.

### Step 2: Verify Prerequisites

Run these checks:

```bash
# Verify cargo-release is installed
cargo release --version

# Verify cargo-vendor-filterer is installed
cargo vendor-filterer --help 2>&1 | head -1
```

If either is missing, offer to install:

```bash
cargo install cargo-release cargo-vendor-filterer
```

Then verify the pre-release was merged:

```bash
# Pull latest main
git checkout main
git pull ${UPSTREAM_REMOTE} main

# Verify the dependency update commit exists
git log --oneline -5
# Should contain "cargo: update dependencies" and "docs/release-notes: update for release X.Y.Z"
```

If the pre-release commits are not found, warn the user and stop. They need to run `/prepare-release` first.

### Step 3: Clean Build Verification

Display: "**Phase 1: Clean Build Verification**"

Run these commands sequentially:

```bash
cargo vendor-filterer target/vendor
```

```bash
cargo test --all-features --config 'source.crates-io.replace-with="vv"' --config 'source.vv.directory="target/vendor"'
```

```bash
cargo clean
git clean -fd
```

If any step fails, report the error and stop. The build must be clean before proceeding.

### Step 4: Create Release Branch

Display: "**Phase 2: Create Release Commit and Tag**"

```bash
git checkout -b release-${RELEASE_VER}
```

Now instruct the user to run `cargo release` manually, because it requires GPG signing and interactive confirmation:

```
Please run the following command in your terminal:

    cargo release --execute ${RELEASE_VER}

This will:
- Update version in Cargo.toml to ${RELEASE_VER}
- Create a signed commit: "cargo: Afterburn release ${RELEASE_VER}"
- Create a signed tag: v${RELEASE_VER}

Confirm the version when prompted. Let me know when it completes.
```

Use the question tool to ask the user if it completed successfully.

After confirmation, verify:

```bash
# Verify the commit exists
git log --oneline -1
# Should show "cargo: Afterburn release X.Y.Z"

# Verify Cargo.toml version
grep "^version = \"${RELEASE_VER}\"$" Cargo.toml

# Verify tag exists
git tag --list "v${RELEASE_VER}"
```

If verification fails, report what went wrong.

### Step 5: Push and Open Release PR

Display: "**Phase 3: Release PR**"

```bash
git push ${UPSTREAM_REMOTE} release-${RELEASE_VER}
```

Open the PR:

```bash
gh pr create \
  --title "Release ${RELEASE_VER}" \
  --body "$(cat <<'EOF'
Release PR for Afterburn ${RELEASE_VER}.

This PR should contain exactly one commit updating the version in Cargo.toml.
EOF
)"
```

Verify the PR has exactly one commit:

```bash
git log main..release-${RELEASE_VER} --oneline
```

Tell the user:

```
Release PR opened: {PR_URL}

Please:
1. Verify the PR contains exactly one commit
2. Get it reviewed and approved
3. Merge the PR
4. Come back here to continue

Let me know when the PR is merged.
```

Use the question tool to wait for the user.

### Step 6: Publish Tag

Display: "**Phase 4: Publish Artifacts**"

After user confirms PR is merged:

```bash
git checkout v${RELEASE_VER}
```

Verify version:

```bash
grep "^version = \"${RELEASE_VER}\"$" Cargo.toml
```

Push the tag:

```bash
git push ${UPSTREAM_REMOTE} v${RELEASE_VER}
```

Now instruct the user to publish to crates.io:

```
Please run the following command to publish to crates.io:

    cargo publish

This requires your crates.io authentication token.
Let me know when it completes.
```

Use the question tool to wait for the user.

### Step 7: Vendor Archive

Display: "**Phase 5: Vendor Archive**"

```bash
cargo vendor-filterer --format=tar.gz --prefix=vendor target/afterburn-${RELEASE_VER}-vendor.tar.gz
```

Compute digests:

```bash
sha256sum target/package/afterburn-${RELEASE_VER}.crate
sha256sum target/afterburn-${RELEASE_VER}-vendor.tar.gz
```

Store the digest values to include in the GitHub release.

### Step 8: Draft GitHub Release

Display: "**Phase 6: GitHub Release**"

Read the release notes for this version from `docs/release-notes.md`. Extract the section between `## Afterburn ${RELEASE_VER}` and the next `## ` header.

Present to the user:

```
Please create a GitHub release:

1. Go to: https://github.com/coreos/afterburn/tags
2. Find tag v${RELEASE_VER}, click the three-dot menu, and create a release
3. Paste the following changelog:

---
{extracted release notes}
---

4. Upload: target/afterburn-${RELEASE_VER}-vendor.tar.gz

5. Include these digests in the release description:

    sha256sum afterburn-${RELEASE_VER}.crate: {digest}
    sha256sum afterburn-${RELEASE_VER}-vendor.tar.gz: {digest}

6. Publish the release

Let me know when done.
```

Use the question tool to wait for the user.

### Step 9: Cleanup

Display: "**Phase 7: Cleanup**"

```bash
cargo clean
git checkout main
git pull ${UPSTREAM_REMOTE} main
git push ${UPSTREAM_REMOTE} :pre-release-${RELEASE_VER} :release-${RELEASE_VER}
git branch -d pre-release-${RELEASE_VER} release-${RELEASE_VER}
```

### Step 10: Final Summary

```
Release ${RELEASE_VER} complete!

Completed steps:
  [x] Clean build verification
  [x] Release commit and tag (cargo release)
  [x] Release PR merged
  [x] Tag pushed to upstream
  [x] Crate published to crates.io
  [x] Vendor archive created
  [x] GitHub release published
  [x] Branches cleaned up

Remaining manual steps (Fedora/CentOS packaging):
  [ ] Review Packit PR in Fedora: https://src.fedoraproject.org/rpms/rust-afterburn/pull-requests
  [ ] Merge rawhide into relevant branches (e.g., f43) and run fedpkg build
  [ ] Submit builds to bodhi
  [ ] Submit fast-track for FCOS testing-devel
  [ ] Submit fast-track for FCOS next-devel (if open)
  [ ] Create rebase-c9s-afterburn issue (CentOS Stream 9)
  [ ] Create rebase-c10s-afterburn issue (CentOS Stream 10)
```

## Checklist Coverage

From the release checklist, this skill covers:

- [x] Make sure cargo-release and cargo-vendor-filterer are up to date
- [x] `git checkout main && git pull`
- [x] `cargo vendor-filterer target/vendor`
- [x] `cargo test` with vendored deps
- [x] `cargo clean && git clean -fd`
- [x] `git checkout -b release-X.Y.Z`
- [x] `cargo release --execute X.Y.Z` (guided, user runs)
- [x] Push release branch and open PR
- [x] `git checkout vX.Y.Z` and verify version
- [x] Push tag
- [x] `cargo publish` (guided, user runs)
- [x] Vendor archive creation
- [x] SHA256 digest computation
- [x] GitHub release body preparation
- [x] Branch cleanup

## What's NOT covered

- GPG key setup or management
- crates.io account setup
- Fedora packaging (Packit PR review, bodhi submissions)
- CentOS Stream packaging (internal team process)
- FCOS fast-track submissions

## Automation Boundaries

| Step | Automated? | Reason |
|------|-----------|--------|
| cargo vendor-filterer, cargo test | Yes | Deterministic build commands |
| git checkout, push, branch, clean | Yes | Standard git operations |
| gh pr create | Yes | CLI-based |
| cargo release --execute | No | Requires GPG signing + interactive confirmation |
| cargo publish | No | Requires crates.io auth token |
| GitHub release creation | No | User should review changelog before publishing |
| sha256sum computation | Yes | Deterministic |

## References

- Full release checklist: `.github/ISSUE_TEMPLATE/release-checklist.md`
- cargo-release config: `Cargo.toml:14-20`
- Example release PRs: #1224 (5.9.0), #1206 (5.8.2), #1200 (5.8.0)
- Release that needed a redo: v5.8.1 ("Re-release of 5.8.0 due to error")
