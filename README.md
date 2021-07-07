# Afterburn

[![crates.io](https://img.shields.io/crates/v/afterburn.svg)](https://crates.io/crates/afterburn)
![minimum rust 1.46](https://img.shields.io/badge/rust-1.46%2B-orange.svg)

Afterburn is a one-shot agent for cloud-like platforms which interacts with provider-specific metadata endpoints.
It is typically used in conjunction with [Ignition](https://github.com/coreos/ignition).

## Features

It comprises several modules which may run at different times during the lifecycle of an instance.

Depending on the specific platform, the following services may run in the [initramfs](dracut/30afterburn/) on first boot:
 * setting local hostname
 * injecting [network command-line arguments](docs/usage/initrd-network-cmdline.md)

The following features are conditionally available on some platforms as [systemd service units](systemd/):
 * installing public SSH keys for local system users
 * retrieving [attributes](docs/usage/attributes.md) from instance metadata
 * checking in to the provider in order to report a successful boot or instance provisioning

## Supported platforms

See [Supported platforms](docs/platforms.md).
