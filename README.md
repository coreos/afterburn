# Afterburn

[![Build Status](https://travis-ci.org/coreos/afterburn.svg?branch=master)](https://travis-ci.org/coreos/afterburn)
[![crates.io](https://img.shields.io/crates/v/afterburn.svg)](https://crates.io/crates/afterburn)
![minimum rust 1.37](https://img.shields.io/badge/rust-1.37%2B-orange.svg)

This is a small utility, typically used in conjunction with [Ignition][ignition], which reads metadata from a given cloud-provider and applies it to the system.
This can include adding SSH keys and writing cloud-specific attributes into an environment file (e.g. `/run/metadata/afterburn`), which can then be consumed by systemd service units via `EnvironmentFile=`.

## Support

The supported cloud providers and their respective metadata are listed below.
On CoreOS Container Linux, the supported providers and metadata are [somewhat different][cl-legacy].

  - aliyun
    - SSH Keys
    - Attributes
      - AFTERBURN_ALIYUN_EIPV4
      - AFTERBURN_ALIYUN_HOSTNAME
      - AFTERBURN_ALIYUN_IMAGE_ID
      - AFTERBURN_ALIYUN_INSTANCE_ID
      - AFTERBURN_ALIYUN_INSTANCE_TYPE
      - AFTERBURN_ALIYUN_IPV4_PRIVATE
      - AFTERBURN_ALIYUN_IPV4_PUBLIC
      - AFTERBURN_ALIYUN_REGION_ID
      - AFTERBURN_ALIYUN_VPC_ID
      - AFTERBURN_ALIYUN_ZONE_ID
  - aws
    - SSH Keys
    - Attributes
      - AFTERBURN_AWS_HOSTNAME
      - AFTERBURN_AWS_PUBLIC_HOSTNAME
      - AFTERBURN_AWS_IPV4_LOCAL
      - AFTERBURN_AWS_IPV4_PUBLIC
      - AFTERBURN_AWS_AVAILABILITY_ZONE
      - AFTERBURN_AWS_INSTANCE_ID
      - AFTERBURN_AWS_INSTANCE_TYPE
      - AFTERBURN_AWS_REGION
  - azure
    - SSH Keys
    - Attributes
      - AFTERBURN_AZURE_IPV4_DYNAMIC
      - AFTERBURN_AZURE_IPV4_VIRTUAL
      - AFTERBURN_AZURE_VMSIZE
  - cloudstack-configdrive
    - SSH Keys
    - Attributes
      - AFTERBURN_CLOUDSTACK_AVAILABILITY_ZONE
      - AFTERBURN_CLOUDSTACK_INSTANCE_ID
      - AFTERBURN_CLOUDSTACK_SERVICE_OFFERING
      - AFTERBURN_CLOUDSTACK_CLOUD_IDENTIFIER
      - AFTERBURN_CLOUDSTACK_LOCAL_HOSTNAME
      - AFTERBURN_CLOUDSTACK_VM_ID
  - cloudstack-metadata
    - SSH Keys
    - Attributes
      - AFTERBURN_CLOUDSTACK_AVAILABILITY_ZONE
      - AFTERBURN_CLOUDSTACK_CLOUD_IDENTIFIER
      - AFTERBURN_CLOUDSTACK_HOSTNAME
      - AFTERBURN_CLOUDSTACK_INSTANCE_ID
      - AFTERBURN_CLOUDSTACK_IPV4_LOCAL
      - AFTERBURN_CLOUDSTACK_IPV4_PUBLIC
      - AFTERBURN_CLOUDSTACK_LOCAL_HOSTNAME
      - AFTERBURN_CLOUDSTACK_PUBLIC_HOSTNAME
      - AFTERBURN_CLOUDSTACK_SERVICE_OFFERING
      - AFTERBURN_CLOUDSTACK_VM_ID
  - digitalocean
    - SSH Keys
    - Network Configs
    - Attributes
      - AFTERBURN_DIGITALOCEAN_HOSTNAME
      - AFTERBURN_DIGITALOCEAN_IPV4_ANCHOR_0
      - AFTERBURN_DIGITALOCEAN_IPV4_PUBLIC_0
      - AFTERBURN_DIGITALOCEAN_IPV4_PRIVATE_0
      - AFTERBURN_DIGITALOCEAN_IPV6_PUBLIC_0
      - AFTERBURN_DIGITALOCEAN_IPV6_PRIVATE_0
      - AFTERBURN_DIGITALOCEAN_REGION
  - exoscale
    - SSH Keys
    - Attributes
      - AFTERBURN_EXOSCALE_AVAILABILITY_ZONE
      - AFTERBURN_EXOSCALE_CLOUD_IDENTIFIER
      - AFTERBURN_EXOSCALE_INSTANCE_ID
      - AFTERBURN_EXOSCALE_LOCAL_IPV4
      - AFTERBURN_EXOSCALE_PUBLIC_IPV4
      - AFTERBURN_EXOSCALE_LOCAL_HOSTNAME
      - AFTERBURN_EXOSCALE_PUBLIC_HOSTNAME
      - AFTERBURN_EXOSCALE_SERVICE_OFFERING
      - AFTERBURN_EXOSCALE_VM_ID
  - gcp
    - SSH Keys
    - Attributes
      - AFTERBURN_GCP_HOSTNAME
      - AFTERBURN_GCP_IP_EXTERNAL_0
      - AFTERBURN_GCP_IP_LOCAL_0
      - AFTERBURN_GCP_MACHINE_TYPE
  - ibmcloud
    - Attributes
      - AFTERBURN_IBMCLOUD_INSTANCE_ID
      - AFTERBURN_IBMCLOUD_LOCAL_HOSTNAME
  - ibmcloud-classic
    - Attributes
      - AFTERBURN_IBMCLOUD_CLASSIC_INSTANCE_ID
      - AFTERBURN_IBMCLOUD_CLASSIC_LOCAL_HOSTNAME
  - openstack-metadata
    - SSH Keys
    - Attributes
      - AFTERBURN_OPENSTACK_HOSTNAME
      - AFTERBURN_OPENSTACK_IPV4_LOCAL
      - AFTERBURN_OPENSTACK_IPV4_PUBLIC
      - AFTERBURN_OPENSTACK_INSTANCE_ID
      - AFTERBURN_OPENSTACK_INSTANCE_TYPE
  - packet
    - SSH Keys
    - Network Configs
    - Attributes
      - AFTERBURN_PACKET_HOSTNAME
      - AFTERBURN_PACKET_PLAN
      - AFTERBURN_PACKET_IPV4_PUBLIC_0
      - AFTERBURN_PACKET_IPV4_PUBLIC_GATEWAY_0
      - AFTERBURN_PACKET_IPV4_PRIVATE_0
      - AFTERBURN_PACKET_IPV4_PRIVATE_GATEWAY_0
      - AFTERBURN_PACKET_IPV6_PUBLIC_0
      - AFTERBURN_PACKET_IPV6_PUBLIC_GATEWAY_0
  - vagrant-virtualbox
    - Attributes
      - AFTERBURN_VAGRANT_VIRTUALBOX_PRIVATE_IPV4
      - AFTERBURN_VAGRANT_VIRTUALBOX_HOSTNAME

Additionally, some attribute names are reserved for usage by [custom metadata providers][custom-metadata].
These can be safely used by external providers on a platform not supported by Afterburn:

  - custom
    - Attributes
      - AFTERBURN_CUSTOM_HOSTNAME
      - AFTERBURN_CUSTOM_PUBLIC_IPV4
      - AFTERBURN_CUSTOM_PRIVATE_IPV4
      - AFTERBURN_CUSTOM_PUBLIC_IPV6
      - AFTERBURN_CUSTOM_PRIVATE_IPV6

[ignition]: https://github.com/coreos/ignition
[custom-metadata]: https://github.com/coreos/container-linux-config-transpiler/blob/v0.8.0/doc/dynamic-data.md#custom-metadata-providers
[cl-legacy]: docs/container-linux-legacy.md
