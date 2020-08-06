# Afterburn

[![Build Status](https://travis-ci.org/coreos/afterburn.svg?branch=master)](https://travis-ci.org/coreos/afterburn)
[![crates.io](https://img.shields.io/crates/v/afterburn.svg)](https://crates.io/crates/afterburn)
![minimum rust 1.40](https://img.shields.io/badge/rust-1.40%2B-orange.svg)

Afterburn is a one-shot agent for cloud-like platforms which interacts with provider-specific metadata endpoints.
It is typically used in conjunction with [Ignition](https://github.com/coreos/ignition).

## Features

It comprises several modules which may run at different times during the lifecycle of an instance.

Depending on the specific platform, the following services may run in the [initramfs](./dracut/30afterburn/) on first boot:
 * setting local hostname
 * injecting [network command-line arguments](./docs/usage/initrd-network-cmdline.md)

The following features are conditionally available on some platforms as [systemd service units](./systemd/):
 * installing public SSH keys for local system users
 * retrieving [attributes](./docs/usage/attributes.md) from instance metadata
 * checking in to the provider in order to report a successful boot or instance provisioning

## Supported platforms

By default Afterburn uses the Ignition platform ID to detect the environment where it is running.

The following platforms are supported, with a different set of features available on each: 
* aliyun
  - Attributes
  - SSH Keys
* aws
  - Attributes
  - SSH Keys
* azure
  - Attributes
  - Boot check-in
  - SSH Keys
* cloudstack-configdrive
  - Attributes
  - SSH Keys
* cloudstack-metadata
  - Attributes
  - SSH Keys
* digitalocean
  - Attributes
  - SSH Keys
* exoscale
  - Attributes
  - SSH Keys
* gcp
  - Attributes
  - SSH Keys
* ibmcloud
  - Attributes
  - SSH Keys
* ibmcloud-classic
  - Attributes
* openstack
  - Attributes
  - SSH Keys
* openstack-metadata
  - Attributes
  - SSH Keys
* packet
  - Attributes
  - First-boot check-in
  - SSH Keys
* vmware
  - Custom network command-line arguments
* vultr
  - Attributes
  - SSH Keys
