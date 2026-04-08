---
name: prepare-release
description: Create the pre-release PR with dependency updates and drafted release notes
---

# Prepare Release

Automate the pre-release PR for an Afterburn release: create branch, update dependencies, draft release notes from commit history, and open the PR.

## What it does

1. Creates the `pre-release-X.Y.Z` branch from up-to-date `main`
2. Checks `Cargo.toml` for unintended lower bound changes since the last tag
3. Runs `cargo update` and commits with message `cargo: update dependencies`
4. Analyzes commits since the last tag to draft release notes
5. Updates `docs/release-notes.md` with the drafted notes and commits with message `docs/release-notes: update for release X.Y.Z`
6. Pushes the branch and opens a PR

## Prerequisites

- On the `main` branch, up to date with the upstream remote
- `cargo` installed
- `gh` CLI installed and authenticated (for opening the PR)
- Write access to the repository

## Usage

```bash
/prepare-release 5.11.0
/prepare-release 5.11.0 --next 5.12.0
```

## Workflow

### Step 1: Gather Version Information

Get the release version from the user's command arguments. If not provided, ask:

```
What version are you releasing? (e.g., 5.11.0)
```

Determine the next development version:
- If `--next` was provided, use that
- Otherwise, auto-increment the minor version (e.g., `5.11.0` -> `5.12.0`)

### Step 2: Validate Prerequisites

Run these checks and stop if any fail:

```bash
# Verify we're on main
git branch --show-current
# Should output: main

# Verify main is up to date
git fetch origin
git status
# Should show "Your branch is up to date with 'origin/main'"

# Verify no existing branch
git branch --list "pre-release-${RELEASE_VER}"
# Should be empty

# Verify no existing tag
git tag --list "v${RELEASE_VER}"
# Should be empty

# Get the last tag for reference
git describe --abbrev=0
```

If `main` is behind, run `git pull origin main` first.

If a branch or tag already exists, warn the user and stop.

### Step 3: Create Pre-Release Branch

```bash
git checkout -b pre-release-${RELEASE_VER}
```

### Step 4: Check Cargo.toml Lower Bounds

```bash
git diff $(git describe --abbrev=0) Cargo.toml
```

If there are changes to dependency version lower bounds, present them to the user:

```
The following Cargo.toml changes were detected since the last release.
Please review for unintended lower bound increases:

{diff output}

Continue? (y/n)
```

Use the question tool to ask the user to confirm. If the user wants to stop, provide instructions for cleaning up the branch.

### Step 5: Update Dependencies

```bash
cargo update
git add Cargo.lock
git commit -m "cargo: update dependencies"
```

### Step 6: Analyze Commits for Release Notes

Get all commits since the last tag, excluding noise:

```bash
# Get the last tag
LAST_TAG=$(git describe --abbrev=0)

# Get commits since last tag, excluding merges
git log ${LAST_TAG}..HEAD --oneline --no-merges
```

Classify each commit into categories. Use these rules:

**EXCLUDE these commits (do not include in release notes):**
- Commits starting with `build(deps):` (Dependabot bumps)
- Commits starting with `Sync repo templates` (automated)
- Commits starting with `cargo: update dependencies` (the one we just made)
- Commits starting with `cargo: Afterburn release` (release machinery)
- Commits starting with `docs/release-notes:` (release notes updates)
- Any merge commits

**MAJOR changes (new features, new providers, significant changes):**
- New provider implementations (commits mentioning "implement" + "provider", or "Add support for")
- Major new feature areas (network config, new platform support)
- Significant architectural changes or refactors

**MINOR changes (bug fixes, small improvements):**
- Bug fixes to existing providers
- Small feature additions (new attributes, config tweaks)
- Documentation changes
- Test additions/improvements
- Build/CI fixes
- Dracut/systemd service changes

**PACKAGING changes:**
- Rust version requirement changes (check if `rust-version` changed in Cargo.toml)
- Dependency requirement changes (not routine bumps, but actual new deps or removed deps)
- Build system changes

### Step 7: Draft Release Notes

Read the current `docs/release-notes.md` file. The top section will look like:

```markdown
## Upcoming Afterburn X.Y.Z (unreleased)

Major changes:

- {existing items that were added incrementally}

Minor changes:

- {existing items}

Packaging changes:

{existing items}
```

Some release notes entries may already exist (contributors often add entries when they merge features). Preserve those and add any missing ones from the commit analysis.

Present the drafted release notes to the user for review:

```
Here are the drafted release notes for Afterburn ${RELEASE_VER}:

## Afterburn ${RELEASE_VER}

Major changes:

- {items}

Minor changes:

- {items}

Packaging changes:

- {items}

Does this look correct? You can edit or I can proceed.
```

Use the question tool to ask the user to confirm or request changes.

### Step 8: Update Release Notes File

Edit `docs/release-notes.md` to:

1. **Replace** the `## Upcoming Afterburn X.Y.Z (unreleased)` header with `## Afterburn ${RELEASE_VER}`
2. **Prepend** a new upcoming section before it:

```markdown
## Upcoming Afterburn ${NEXT_VER} (unreleased)

Major changes:

Minor changes:

Packaging changes:

```

3. Clean up any empty sections from the now-current release (remove "Packaging changes:" if it has no items under it, etc.)

Then commit:

```bash
git add docs/release-notes.md
git commit -m "docs/release-notes: update for release ${RELEASE_VER}"
```

### Step 9: Push and Open PR

```bash
git push origin pre-release-${RELEASE_VER}
```

Open a PR:

```bash
gh pr create \
  --title "pre-release ${RELEASE_VER}" \
  --body "$(cat <<'EOF'
Pre-release PR for Afterburn ${RELEASE_VER}.

This PR contains:
1. `cargo: update dependencies` - Updated Cargo.lock
2. `docs/release-notes: update for release ${RELEASE_VER}` - Release notes

Please review and merge. After merging, continue with the release checklist.
EOF
)"
```

### Step 10: Summary

```
Pre-release PR created!

Branch: pre-release-${RELEASE_VER}
Commits:
  1. cargo: update dependencies
  2. docs/release-notes: update for release ${RELEASE_VER}

PR: {PR_URL}

Next steps (from the release checklist):
  1. Get PR reviewed and merged
  2. Run /publish-release ${RELEASE_VER} to continue with the release
```

## Checklist Coverage

From the release checklist in `.github/ISSUE_TEMPLATE/release-checklist.md`:

- [x] `RELEASE_VER=x.y.z` -- captured as input
- [x] `git checkout -b pre-release-${RELEASE_VER}` -- Step 3
- [x] `git diff $(git describe --abbrev=0) Cargo.toml` -- Step 4
- [x] `cargo update` -- Step 5
- [x] `git add Cargo.lock && git commit` -- Step 5
- [x] Write release notes -- Steps 6-8
- [x] `git add docs/release-notes.md && git commit` -- Step 8
- [x] PR the changes -- Step 9

## What's NOT covered

- Reviewing the actual dependency changes in `Cargo.lock`
- Branched releases (cherry-picking release notes into main)
- Final approval/merge of the PR

## Worked Examples

### Release 5.10.0
Commits between v5.9.0 and pre-release (excluding noise):
```
e469d05 providers/azure: switch SSH key retrieval from certs endpoint to IMDS
96c5530 microsoft/azure: Fix SharedConfig parsing of XML attributes
627089c microsoft/azure: Add XML attribute alias for serde-xml-rs Fedora compat
c8cc721 upcloud: implement UpCloud provider
```
Resulting release notes:
- Major: "Add support for UpCloud"
- Minor: "Azure: fetch SSH keys from IMDS instead of deprecated certificates endpoint", "Azure: fix parsing of SharedConfig XML document with current serde-xml-rs"

### Release 5.9.0
Key commits:
```
d4f8031 oraclecloud: implement oraclecloud provider
62d9ce2 dracut: Return 255 in module-setup
5f3dca0 Add TMT test structure and basic smoke test
```
Resulting release notes:
- Major: "Add support for Oracle Cloud Infrastructure", "dracut: don't include module by default"
- Minor: "Add TMT test structure and basic smoke test"

## References

- Full release checklist: `.github/ISSUE_TEMPLATE/release-checklist.md`
- Release notes format: `docs/release-notes.md`
- Example pre-release PRs: #1223 (5.9.0), #1205 (5.8.2), #1199 (5.8.0)
- Commit message convention: `docs/contributing.md:48-69`
