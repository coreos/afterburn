# Release process

This project uses [cargo-release][cargo-release] in order to prepare new releases, tag and sign relevant git commit, and publish the resulting artifacts to [crates.io][crates-io].
In order to ease downstream packaging of Rust binaries, [cargo-vendor][cargo-vendor] is also used to provide an archive of vendored dependencies (only relevant for offline builds).

The release process follows the usual PR-and-review flow, allowing an external reviewer to have a final check before publishing.

This document gives an high-level overview as well as a step-by-step guide on how to perform a release.

## Overview

Most of the process is automated with the help of cargo-release and its metadata entries in Cargo manifest.
This helper is in charge of bumping the `version` field in the manifest in a dedicated commit, attaching the corresponding signed tag to it, and then producing another commit to prepare the project for the next development cycle.

The two resulting commits can be then submitted in a dedicated PR and reviewed.
Once merged, the last steps of the process consist of pushing the git tag, publishing the crate, and publishing the vendor tarball.

## Requirements

This guide requires:

 * a web browser (and network connectivity)
 * `git`
 * `tar`
 * `sha256sum`
 * GPG setup and personal key for signing
 * `cargo` (suggested: latest stable toolchain from [rustup][rustup])
 * `cargo-release` (suggested: `cargo install -f cargo-release`)
 * `cargo-vendor` (suggested: `cargo install -f cargo-vendor`)
 * A verified account on crates.io
 * Write access to https://github.com/coreos/afterburn
 * Upload access to https://crates.io/crates/afterburn

## Steps

These steps show how to release version `x.y.z` on the `origin` remote (this can be checked via `git remote -av`).
Push access to the upstream repository is required in order to publish the new tag and the PR branch.

For each release to be published, proceed as follows:

#### 1. Make sure the project is clean and prepare the environment

* `cargo test`
* `cargo clean`
* `git clean -fd`
* `export RELEASE_VER=x.y.z`
* `export UPSTREAM_REMOTE=origin`

:warning:: `UPSTREAM_REMOTE` should reference your locally configured remote that points to the https://github.com/coreos/afterburn git repository

#### 2. Create release commits on a dedicated branch and tag it

* `git checkout -b release-${RELEASE_VER}`
* This will create the tag after asking for version confirmation:

  `cargo release`

#### 3. Open a PR for this release

* `git push ${UPSTREAM_REMOTE} release-${RELEASE_VER}`
* Open a web browser and create a Pull Request for the branch above
* Make sure the resulting PR contains exactly two commits

#### 4. Get the PR reviewed, approved and merged

#### 5. Publish the artifacts (tag and crate)

* `git push ${UPSTREAM_REMOTE} v${RELEASE_VER}`
* Make sure the upstream tag matches the local tag:

    `git fetch --tags --verbose ${UPSTREAM_REMOTE} 2>&1 | grep ${RELEASE_VER}`
* `git checkout v${RELEASE_VER}`
* Make sure the tag is what you intend to release; if so this will show an empty output:

    `git diff release-${RELEASE_VER}~1 v${RELEASE_VER}`
* `cargo publish`

#### 6. Assemble vendor tarball and publish a Release

* `cargo vendor`
* `tar -czf target/afterburn-${RELEASE_VER}-vendor.tar.gz vendor`
* Open a web browser and create a GitHub Release for the tag above
* Attach the `vendor.tar.gz` (located under `target/`) to the current Release
* Record digests of local artifacts:

    `sha256sum target/package/afterburn-${RELEASE_VER}.crate`
    `sha256sum target/afterburn-${RELEASE_VER}-vendor.tar.gz`
* Write a short changelog (see previous entries) and publish the Release

#### 7. Clean up the environment

* `unset RELEASE_VER`
* `unset UPSTREAM_REMOTE`
* `cargo clean`
* `rm -rf vendor`
* `git checkout master`
* `git pull ${UPSTREAM_REMOTE} master`
* `git push ${UPSTREAM_REMOTE} :release-${RELEASE_VER}`

[cargo-release]: https://github.com/sunng87/cargo-release
[cargo-vendor]: https://github.com/alexcrichton/cargo-vendor
[rustup]: https://rustup.rs/
[crates-io]: https://crates.io/
