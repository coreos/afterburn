# CoreOS Metadata

This is a small utility, typically used in conjunction with [Ignition][ignition], which reads metadata from a given cloud-provider and applies it to the system. This can include adding SSH keys and writing cloud-specific attributes into an environment file. This file can then be consumed by systemd service units via `EnvironmentFile=`.

## Support

The supported cloud providers and their respective metadata are as follows:

  - azure
    - Attributes
      - COREOS_AZURE_IPV4_DYNAMIC
      - COREOS_AZURE_IPV4_VIRTUAL
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

[ignition]: https://github.com/coreos/ignition
