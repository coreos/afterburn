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
