---
nav_order: 7
---

# Supported platforms

By default Afterburn uses the Ignition platform ID to detect the environment where it is running.

The following platforms are supported, with a different set of features available on each:

* akamai
  - Attributes
  - SSH Keys
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
* azurestack
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
* hetzner
  - Attributes
  - Hostname
  - SSH Keys
* ibmcloud
  - Attributes
  - SSH Keys
* ibmcloud-classic
  - Attributes
* kubevirt
  - Attributes
  - SSH Keys
* openstack
  * Metadata source: config-drive if present, otherwise metadata service
  * Features:
      - Attributes
      - SSH Keys
* openstack-metadata
  * Metadata source: metadata service
  * Features:
      - Attributes
      - SSH Keys
* packet
  - Attributes
  - First-boot check-in
  - SSH Keys
* powervs
  - Attributes
  - SSH keys
* proxmoxve
  - Attributes
  - Hostname
  - SSH keys
  - Network configuration
* scaleway
  - Attributes
  - Boot check-in
  - SSH keys
* vmware
  - Custom network command-line arguments
* vultr
  - Attributes
  - SSH Keys
