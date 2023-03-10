---
nav_order: 1
parent: Development
---

# Integrating Afterburn into a distribution

## SSH keys

The `--ssh-keys` option (invoked by `afterburn-sshkeys@.service`) writes SSH keys to `~user/.ssh/authorized_keys.d/afterburn`.
For sshd to respect this file, it must be configured with an `AuthorizedKeysCommand` that reads files from the `authorized_keys.d` directory.
Alternatively, sshd can be configured to read the fragment file directly:

```
AuthorizedKeysFile .ssh/authorized_keys .ssh/authorized_keys.d/afterburn
```

## VMware Netplan guestinfo metadata

The `guestinfo.metadata` and `guestinfo.metadata.encoding` fields can contain a Netplan configuration provided by the VM provisioning logic.
Netplan is required on the OS to render the Netplan format to either NetworkManager or systemd-networkd configuration files. By default, Netplan generates systemd-networkd units. Since the renderer backend is defined in the Netplan config itself, requiring NetworkManager in the config would rule out support for systems that don't use it (unless they would ship a drop-in file with later lexicographical ordering to force it to `networkd`). As systemd-networkd can work in parallel with NetworkManager, it's expected that the renderer field is left to its default but systems can also add a default drop-in file with early lexicographical ordering to prefer NetworkManager.

The Afterburn invocation is as follows, where `FOLDER` could be `/run/netplan/`:

```
afterburn multi --netplan-configs FOLDER --provider vmware
```

Afterwards, `netplan generate` can be used to render the config files. If that is done before `systemd-networkd` runs, this is enough, but if the network already is up, `netplan apply` should be used instead.
