---
nav_order: 8
---

# Release notes

## Upcoming Afterburn 5.7.0 (unreleased)

Major changes:

- Add support for Proxmox VE

Minor changes:

Packaging changes:


## Afterburn 5.6.0

Major changes:

- Add support for Akamai Connected Cloud (Linode)

Packaging changes:

- Require Rust ≥ 1.75.0


## Afterburn 5.5.1 (2024-01-12)

Minor changes:

- providers/vmware: add missing public functions for non-amd64


## Afterburn 5.5.0 (2023-11-22)

Major changes:

- Add support for Hetzner Cloud
- Add support for Scaleway
- Add Netplan guestinfo support on VMware

Minor changes:

- openstack: Add `OPENSTACK_INSTANCE_UUID` attribute
- openstack-metadata: Add `OPENSTACK_INSTANCE_UUID` attribute
- dracut: run hostname service on kubevirt

Packaging changes:

- Require Rust ≥ 1.71.0
- Specify features depended on from Nix
- Replace deprecated dependency `users` by `uzers`


## Afterburn 5.4.3 (2023-06-05)

Packaging changes:

- Fix incomplete package in vendor tarball


## Afterburn 5.4.2 (2023-05-18)

Minor changes:

- Fix SSH key fetching on Azure with `openssl` crate ≥ 0.10.46

Packaging changes:

- Require `clap` ≥ 4
- Require `mockito` ≥ 1
- Require `openssl` ≥ 0.10.46


## Afterburn 5.4.1 (2023-02-06)

Packaging changes:

- Fix missing static archives in vendor tarball
- Remove non-Linux dependencies from vendor tarball


## Afterburn 5.4.0 (2023-02-03)

Major changes:

- Support reading DHCP options from NetworkManager, fixing 30s delay on
  Azure, Azure Stack, and CloudStack

Minor changes:

- Add `AWS_AVAILABILITY_ZONE_ID` attribute on AWS
- Fix default dependency ordering on all `checkin` services
- Fix failure setting SSH keys on IBM Cloud if none are provided
- Don't ignore network interfaces that appear during DHCP option lookup retry
- Add release notes to documentation

Packaging changes:

- Require Rust ≥ 1.66.0
- Require `zbus` ≥ 2.3
- Drop `base64`, `byteorder`, `hostname`, `mime`, `serde_derive` dependencies
- Remove static libraries from vendor archive
- Disable LTO in release builds


## Afterburn 5.3.0 (2022-04-29)

Major changes:

- Add support for KubeVirt
- Add `AWS_IPV6` attribute to AWS
- Enable PowerVS in the sshkeys systemd service

Minor changes:

- Bump AWS IMDS metadata version to `2021-01-03`
- Use `RemainAfterExit` on all `oneshot` services
- Support marking network interfaces as not required for `network-online.target`

Packaging changes:

- Require Rust ≥ 1.56.0
- Migrate from `structopt` to `clap` 3
- Remove Windows binaries from vendor archive


## Afterburn 5.2.0 (2022-01-14)

Changes:

- Limit string written to hostname file to `HOST_NAME_MAX` bytes
- Explicitly log the hostname we write
- Only log that we wrote SSH keys when we actually did
- Log a message when SSH key is removed
- Enable debug symbols in release builds


## Afterburn 5.1.0 (2021-08-10)

Changes:

- docs: fix "Edit this page on GitHub" links
- *: rename master branch to main
- *: fix clippy warnings
- lockfile: general refresh, update all openssl crates
- providers/gcp: access GCP metadata service by IP address
- providers/packet: access metadata service over HTTPS
- cli: don't report an error when --help or --version is specified
- cli: correctly print version when --version specified 
- providers: Add PowerVS 
- workflows: bump toolchains; restrict repository access


## Afterburn 5.0.0 (2021-04-09)

Changes:

- *: update minimum toolchain to 1.44
- cargo: update all dependencies
- *: remove cl-legacy feature
- ibmcloud: don't ignore I/O error when parsing metadata
- providers: fix clippy::unnecessary_wraps lint on 1.50 
- workflows: update pinned lint toolchain to 1.50.0
- *: switch from `error-chain` to `anyhow`
- cli: stop wrapping command-line parse errors
- github: release checklist cleanups
- ci: adapt to new buildroot image
- providers: add Azure Stack Hub


## Afterburn 4.6.0 (2020-12-09)

Changes:

- ci: move Travis jobs to GitHub actions
- dracut: run hostname service on aliyun and ibmcloud
- providers/*stack: allow 404 returns on *stack network provider
- workflows/rust: update linting toolchain to latest stable


## Afterburn 4.5.3 (2020-10-22)

Changes:

- docs: clarify afterburn.service enablement
- docs: Add GitHub Pages support
- sshkeys: activate service on OpenStack
- providers/vmware: allow avoiding iopl permission errors
- cargo: remove carets from semver requirements


## Afterburn 4.5.1 (2020-09-02)

Changes:

- ibmcloud: add support for SSH keys
- providers: move trait noops into MetadataProvider
- systemd: add afterburn-sshkeys.target
- ci: bump linting toolchain on Travis


## Afterburn 4.5.0 (2020-08-06)

Changes:

- providers: add a new `openstack` platform
- openstack: use config-drive by default and metadata API as fallback
- azure: rework ready-state posting
- azure: log a warning on network failure if non-root
- providers: add a new `vultr` platform
- systemd: activate relevant services on Vultr


## Afterburn 4.4.2 (2020-07-14)

Changes:

- providers/azure: do not fail on keyless instances
- providers/vmware: fix CPU detection (vmw_backdoor 0.1.3 bugfix)
- providers: mark vagrant-virtualbox as legacy
- github: release checklist cleanups


## Afterburn 4.4.1 (2020-06-30)

Changes:

- docs: rework README
- provider/vagrant: do not loop back local system hostname
- CONTRIBUTING: drop mailing list and IRC references
- docs: explain initrd network arguments usage
- cargo: exclude tool configs from crate
- dracut: run hostname service on digitalocean
- aws: support IMDSv2 for AWS metadata service
- travis: bump to latest toolchain
- github: create Dependabot config file
- aws: change metadata version from 2009-04-04 to 2019-10-01


## Afterburn 4.4.0 (2020-05-22)

Changes:

- providers: setup network kernel arguments in initrd
- providers/vmware: support injecting custom network kargs
- cargo: all dependencies updated


## Afterburn 4.3.3 (2020-04-24)

Changes:

- sshkeys: send structured info to journald
- ci: test a secondary arch on Travis
- ci: rust version from 1.39.0 to 1.40.0 
- makefile: tweak install step
- providers: add vmware
- util/cmdline: add helpers for detecting network kargs
- afterburn: minor cleanups
- ci: hook up to CoreOS CI


## Afterburn 4.3.2 (2020-02-27)

Changes:

- cargo: relax dependencies micro versions 
- cargo: switch from deprecated tempdir crate to tempfile
- providers: add exoscale
- providers: add ibmcloud-classic as a separate platform
- providers/packet: add gateway attributes
- util/mount: log intermediate errors


## Afterburn 4.3.1 (2020-01-21)

Changes:

- cli: introduce sub-commands
- ci: bump toolchains
- providers/azure: fix clippy warnings
- retry: update to new reqwest API
- cargo: update all dependencies


## Afterburn 4.3.0 (2019-12-02)

Changes:

- sshkeys: fix missing directory on empty set
- providers: add metadata support for `ibmcloud` (IBM Cloud VPC Gen2)
- providers/ibmcloud: add support for Classic instance types
- ibmcloud/classic: source network configuration from metadata
- docs: minor fixes
- network: clean up interface logic
- network: clean up virtual netdev logic
- retry: cleanup and test max-retries
- github: add release-checklist template
- cloudstack-configdrive: cleanup mounting and umounting logic
- providers: rework to speed up negative tests
- main: drop extern crate declarations


## Afterburn 4.2.0 (2019-10-11)

Changes:

- providers: fetch instance types for all providers
- providers: add Alibaba Cloud (`aliyun`)
- systemd: allow sshkeys service on aliyun
- tests/gcp: avoid flake
- travis: bump minimum toolchain
- metadata: add `dyn` to remove warning


## Afterburn 4.1.3 (2019-09-12)

Changes:

- cargo: update dependencies (slog, serde, serde_derive, nix, mime)


## Afterburn 4.1.2 (2019-07-23)

Changes:

- systemd: schedule checkin after network target
- systemd: unify provider overriding via env
- dracut: relabel the hostname file
- docs: minor cleanups
- cargo: update all compatible dependencies


## Afterburn 4.1.1 (2019-06-21)

Changes:

- dracut: add afterburn dracut module
- systemd: add comment to sshkeys@.service
- systemd: enable sshkeys unit on supported platforms
- cargo: use pnet_* subcrates
- cargo: update dependencies to latest
- Makefile: add checkin service files


## Afterburn 4.1.0 (2019-04-23)

Changes:

- providers/azure: fetch hostname from metadata 
- add checkin service files for Azure and Packet
- metadata: accept "ec2" provider name only in legacy mode
- bump minimum toolchain to 1.31
- cargo: switch to 2018 edition
- update all dependencies to latest


## Afterburn 4.0.0 (2019-03-28)

Changes:

- rename project from coreos-metadata to Afterburn
- introduce a `cl-legacy` feature flag for backward compatibility with Container Linux
- change metadata attribute prefix from `COREOS_` to `AFTERBURN_` in non-legacy mode
- read `ignition.platform.id` kernel argument instead of `coreos.oem.id` in non-legacy mode
- drop merging of `authorized_keys.d` into `authorized_keys` in non-legacy mode
- providers: rename `ec2` -> `aws`, `gce` -> `gcp` in non-legacy mode

Bugfixes:

- providers/gce: fix panic fetching metadata

Misc: 
- providers/gce: add basic hostname mock-test
- rustfmt whole project


## Afterburn 3.1.0 (2019-03-13)

New features:

- provider: add boot check-in on azure and packet

Misc:

- azure: hardcode fallback for wireserver endpoint
- providers/packet: minor code cleanup
- cargo: update most dependencies to latest versions


## Afterburn 3.0.2 (2018-11-05)

Misc:

- Update compatible dependencies to latest version


## Afterburn 3.0.1 (2018-09-28)

Bugfixes:

- util: minor fixes to cmdline parser (coreos#116)


## Afterburn 3.0.0 (2018-08-24)

Bugfixes:

- providers/gcp: scrape new endpoint for ssh keys (coreos#112, thanks @andor44!)

Misc:

- cargo: bump update-ssh-keys to 0.3.0 (coreos#114)
- src: make a single-binary only project (coreos#113)

This project is now available on the public crates.io registry:
https://crates.io/crates/coreos-metadata


## Afterburn 2.0.0 (2018-08-03)

Features:

- Drop Oracle OCI provider support (coreos#86)
- Refactor providers to use `MetadataProvider` trait (coreos#88)
- Support partial metadata fetching (coreos/bugs#2362, coreos#88)
- Add basic `Makefile` and systemd units (coreos#90)
- Add EC2_PUBLIC_HOSTNAME attribute (coreos#104, thanks @fspijkerman!)
- Enable link-time optimization for release builds (coreos#105)

Bugfixes:

- Do not attempt to write empty sets of SSH keys (coreos/bugs#2312, coreos#97)

Misc:

- Bump `error-chain` dependency to 0.12 (coreos#101)
- Bump `update-ssh-keys` dependency to 0.2.1 (coreos#108)


## Afterburn 1.0.6 (2018-04-20)

- Update dependencies - this fixes breakages on rust compilers newer than 1.23 (coreos#85)


## Afterburn 1.0.5 (2017-12-16)

- fix issue with `oracle-oci` provider where `coreos-metadata` would fail to deserialize the metadata if there were no configured ssh keys (coreos#75)
- fix references in logs to `oracle` instead of `oracle-oci` (coreos#76)
- fix issue with `packet` provider incorrectly configuring network interfaces to bond when no bonds were specified (coreos#77)


## Afterburn 1.0.4 (2017-11-29)

- fixed a bug in digitalocean provider where private ip attributes would be improperly indexed if there were also public ips (coreos#72, thanks @tfussell!)


## Afterburn 1.0.3 (2017-11-16)

- fix issue where golang-style single-hyphen flags were no longer accepted (coreos/bugs#2240, coreos#70)


## Afterburn 1.0.2 (2017-11-08)

- update `openssh-keys` to v0.2.0 to fix bug with `authorized_keys` file parsing.


## Afterburn 1.0.1 (2017-11-07)

- reduce the amount logging volume for the release build (coreos#64)


## Afterburn 1.0.0 (2017-10-19)

`coreos-metadata` has been rewritten in rust.  The command-line interface
and behavior for all providers should be identical to the previous golang
version.  If it's not, please file a bug in our bug tracker,
https://github.com/coreos/bugs (or submit a pr!).

Additionally, `coreos-metadata` now supports ssh keys for azure. 
