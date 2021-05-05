---
nav_order: 9
---

# Contributing

CoreOS projects are [Apache 2.0 licensed][LICENSE] and accept contributions via
GitHub pull requests.  This document outlines some of the conventions on
development workflow, commit message formatting, contact points and other
resources to make it easier to get your contribution accepted.

## Certificate of Origin

By contributing to this project you agree to the Developer Certificate of
Origin (DCO). This document was created by the Linux Kernel community and is a
simple statement that you, as a contributor, have the legal right to make the
contribution. See the [DCO][DCO] file for details.

## Getting Started

- Fork the repository on GitHub
- Read the [documentation][docs].
- Play with the project, submit bugs, submit patches!

### Contribution Flow

This is a rough outline of what a contributor's workflow looks like:

- Create a topic branch from where you want to base your work (usually main).
- Make commits of logical units.
- Make sure your commit messages are in the proper format (see below).
- Push your changes to a topic branch in your fork of the repository.
- Make sure the tests pass, and add any new tests as appropriate.
- Submit a pull request to the original repository.

Thanks for your contributions!

## Project architecture

[Development doc-pages][devdocs] cover several aspects of this project, both at low-level (code and logic) and high-level (architecture and design).

### Code Style

Code should be formatted according to the output of [rustfmt](https://github.com/rust-lang-nursery/rustfmt)

### Format of the Commit Message

We follow a rough convention for commit messages that is designed to answer two
questions: what changed and why. The subject line should feature the what and
the body of the commit should describe the why.

```
scripts: add the test-cluster command

this uses tmux to setup a test cluster that you can easily kill and
start for debugging.

Fixes #38
```

The format can be described more formally as follows:

```
<subsystem>: <what changed>
<BLANK LINE>
<why this change was made>
<BLANK LINE>
<footer>
```

The first line is the subject and should be no longer than 70 characters, the
second line is always blank, and other lines should be wrapped at 80 characters.
This allows the message to be easier to read on GitHub as well as in various
git tools.

## Release process

Release can be performed by [creating a new release ticket][new-release-ticket] and following the steps in the checklist there.

[LICENSE]: https://github.com/coreos/afterburn/blob/main/LICENSE
[DCO]: https://github.com/coreos/afterburn/blob/main/DCO
[docs]: https://coreos.github.io/afterburn
[devdocs]: development.md
[new-release-ticket]: https://github.com/coreos/afterburn/issues/new?labels=kind/release&template=release-checklist.md

