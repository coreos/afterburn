# CoreOS Metadata #

This is a small utility, typically used in conjunction with
[Ignition][ignition], which reads metadata from a given cloud-provider and
writes the values into a specified file. This file can then be consumed by
systemd service units via `EnvironmentFile=`.

## Support ##

The supported cloud providers and their respective metadata are as follows:

 - ec2
   - COREOS_IPV4_LOCAL
   - COREOS_IPV4_PUBLIC
   - COREOS_HOSTNAME
 - azure
   - COREOS_IPV4_LOCAL
   - COREOS_IPV4_PUBLIC

[ignition]: https://github.com/coreos/ignition
