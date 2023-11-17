---
nav_order: 2
parent: Usage
---

# VMware Netplan guestinfo metadata

The network environment can vary between VMware servers and instead of leaking these requirements into userdata snippets, a well-known guestinfo metadata field can be used.
The guestinfo metadata field is OS-independent and supported by cloud-init (spec [here](https://cloudinit.readthedocs.io/en/latest/reference/network-config-format-v2.html), example [here](https://cloudinit.readthedocs.io/en/latest/reference/datasources/vmware.html#walkthrough-of-guestinfo-keys-transport)) and Afterburn. When the OS supports this mechanism the user can provide Netplan configs which the OS renders using the backend of choice.

## Specifying the guestinfo metadata

The guestinfo keys are named `guestinfo.metadata` for the content and `guestinfo.metadata.encoding` to specify the encoding of the content.
The value of the encoding field can be empty to indicate raw string data, or one of `base64` or `b64` to indicate an base64 encoding, or one of `gzip+base64` or `gz+b64` to indicate base64-encoded gzip data.

An example for raw string data is the following:
```
network:
  version: 2
  ethernets:
    nics:
      match:
        name: ens*
      dhcp4: yes
```

The supported config format with examples can be found in the [Netplan specification](https://netplan.readthedocs.io/en/latest/netplan-yaml/).
