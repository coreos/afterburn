---
nav_order: 1
parent: Usage
---

# Initrd first-boot network arguments

Afterburn supports injecting arguments for dracut-cmdline, in order to setup networking in initrd for the first boot.
Valid network arguments are documented in [dracut manpages](https://www.man7.org/linux/man-pages/man7/dracut.cmdline.7.html).

This is a feature with a fairly limited scope, which allows augmenting boot arguments ("kargs") in an additive-only way.
Those arguments are injected by the `afterburn-network-kargs` service and processed by the network management stack in initrd (but they do not have effects on the kernel itself).

If the kernel command-line already specifies some network arguments, Afterburn does not inject any additional parameters.
Otherwise, distribution-specific values are inserted before the networking logic starts.

On platforms where a suitable side-channel is available, it is possible for the user to override the defaults with customized values.

# Platform-specific overrides

## VMware

On VMware, it is possible to provide a guestinfo property containing custom network arguments.
Afterburn will lookup for the well-known key `guestinfo.afterburn.initrd.network-kargs` and use its value instead of the default.
For this to work, the property must be set on the VM before the first boot. It does not affect subsequent boots.

The example below shows a `node1` VM booting with a static hostname and IPv4 routing on the `ens192` interface.

```
VM_NAME="node1"

IPCFG="ip=10.20.30.42::10.20.30.254:255.255.255.0:mynode01:ens192:off"

govc vm.change -vm "${VM_NAME}" -e "guestinfo.afterburn.initrd.network-kargs=${IPCFG}"

govc vm.power -on "${VM_NAME}"
```
