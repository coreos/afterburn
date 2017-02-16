# CoreOS Metadata

This is a small utility, typically used in conjunction with [Ignition][ignition], which reads metadata from a given cloud-provider and applies it to the system. This can include adding SSH keys and writing cloud-specific attributes into an environment file. This file can then be consumed by systemd service units via `EnvironmentFile=`.

## Support

The supported cloud providers and their respective metadata are as follows:

  - azure
    - Attributes
      - COREOS_AZURE_IPV4_DYNAMIC
      - COREOS_AZURE_IPV4_VIRTUAL
  - digitalocean
    - SSH Keys
    - Network Configs
    - Attributes
      - COREOS_DIGITALOCEAN_HOSTNAME
      - COREOS_DIGITALOCEAN_IPV4_ANCHOR_0
      - COREOS_DIGITALOCEAN_IPV4_PUBLIC_0
      - COREOS_DIGITALOCEAN_IPV4_PRIVATE_0
      - COREOS_DIGITALOCEAN_IPV6_PUBLIC_0
      - COREOS_DIGITALOCEAN_IPV6_PRIVATE_0
  - ec2
    - SSH Keys
    - Attributes
      - COREOS_EC2_HOSTNAME
      - COREOS_EC2_IPV4_LOCAL
      - COREOS_EC2_IPV4_PUBLIC
  - gce
    - SSH Keys
    - Attributes
      - COREOS_GCE_HOSTNAME
      - COREOS_GCE_IP_EXTERNAL_0
      - COREOS_GCE_IP_LOCAL_0
  - packet
    - SSH Keys
    - Attributes
      - COREOS_PACKET_HOSTNAME
      - COREOS_PACKET_IPV4_PUBLIC_0
      - COREOS_PACKET_IPV4_PRIVATE_0
      - COREOS_PACKET_IPV6_PUBLIC_0
  - openstack-metadata
    - SSH Keys
    - Attributes
      - COREOS_OPENSTACK_HOSTNAME
      - COREOS_OPENSTACK_IPV4_LOCAL
      - COREOS_OPENSTACK_IPV4_PUBLIC
      - COREOS_OPENSTACK_INSTANCE_ID
  - sakuracloud
    - SSH Keys
    - Network Configs
    - Attributes
      - COREOS_SAKURACLOUD_HOSTNAME
      - COREOS_SAKURACLOUD_IPV4_0
      - COREOS_SAKURACLOUD_IPV6_0

[ignition]: https://github.com/coreos/ignition
