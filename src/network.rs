// Copyright 2017 CoreOS, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! network abstracts away the manipulation of network device and
//! interface unit files. All that is left is to write the resulting string to
//! the necessary unit.

use anyhow::{anyhow, bail, Context, Result};
use ipnetwork::IpNetwork;
use pnet_base::MacAddr;
use slog_scope::warn;
use std::fmt::Write;
use std::net::IpAddr;
use std::string::String;
use std::string::ToString;

pub const BONDING_MODE_BALANCE_RR: u32 = 0;
pub const BONDING_MODE_ACTIVE_BACKUP: u32 = 1;
pub const BONDING_MODE_BALANCE_XOR: u32 = 2;
pub const BONDING_MODE_BROADCAST: u32 = 3;
pub const BONDING_MODE_LACP: u32 = 4;
pub const BONDING_MODE_BALANCE_TLB: u32 = 5;
pub const BONDING_MODE_BALANCE_ALB: u32 = 6;

const BONDING_MODES: [(u32, &str); 7] = [
    (BONDING_MODE_BALANCE_RR, "balance-rr"),
    (BONDING_MODE_ACTIVE_BACKUP, "active-backup"),
    (BONDING_MODE_BALANCE_XOR, "balance-xor"),
    (BONDING_MODE_BROADCAST, "broadcast"),
    (BONDING_MODE_LACP, "802.3ad"),
    (BONDING_MODE_BALANCE_TLB, "balance-tlb"),
    (BONDING_MODE_BALANCE_ALB, "balance-alb"),
];

pub fn bonding_mode_to_string(mode: u32) -> Result<String> {
    for &(m, s) in &BONDING_MODES {
        if m == mode {
            return Ok(s.to_owned());
        }
    }
    Err(anyhow!("no such bonding mode: {}", mode))
}

/// Try to parse an IP+netmask pair into a CIDR network.
pub fn try_parse_cidr(address: IpAddr, netmask: IpAddr) -> Result<IpNetwork> {
    let prefix = ipnetwork::ip_mask_to_prefix(netmask)?;
    IpNetwork::new(address, prefix).context("failed to parse network")
}

/// Format an IP address for dracut kernel arguments.
/// IPv6 addresses are wrapped in brackets so dracut's colon-delimited
/// parser can distinguish them from field separators.
pub fn dracut_addr(addr: &IpAddr) -> String {
    match addr {
        IpAddr::V6(_) => format!("[{addr}]"),
        _ => addr.to_string(),
    }
}

/// Format an IP network (address/prefix) for dracut kernel arguments.
pub fn dracut_network(net: &IpNetwork) -> String {
    match net {
        IpNetwork::V6(_) => format!("[{net}]"),
        _ => net.to_string(),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NetworkRoute {
    pub destination: IpNetwork,
    pub gateway: IpAddr,
}

/// A network interface/link.
///
/// Depending on platforms, an interface may be identified by
/// name or by MAC address (at least one of those must be provided).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Interface {
    /// Interface name.
    pub name: Option<String>,
    /// Interface MAC address.
    pub mac_address: Option<MacAddr>,
    /// Path as identifier
    pub path: Option<String>,
    /// Relative priority for interface configuration.
    pub priority: u8,
    pub nameservers: Vec<IpAddr>,
    pub ip_addresses: Vec<IpNetwork>,
    // Optionally enable DHCP
    pub dhcp: Option<DhcpSetting>,
    pub routes: Vec<NetworkRoute>,
    pub bond: Option<String>,
    pub unmanaged: bool,
    /// Optional requirement setting instead of the default
    pub required_for_online: Option<String>,
}

/// A virtual network interface.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VirtualNetDev {
    pub name: String,
    pub kind: NetDevKind,
    pub mac_address: MacAddr,
    pub priority: Option<u32>,
    pub custom_sections: CustomNetworkSections,
}

/// A free-form `systemd.netdev` section.
///
/// Visit the [systemd documentation](docs) to learn more.
///
/// docs: https://www.freedesktop.org/software/systemd/man/latest/systemd.netdev.html
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SdSection {
    pub name: String,
    pub attributes: Vec<(String, String)>,
}

/// A free-form `NetworkManager` section.
///
/// Visit the [NetworkManager documentation](docs) to learn more.
///
/// docs: https://www.networkmanager.dev/docs/api/latest/ref-settings.html
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NmSection {
    pub name: String,
    pub attributes: Vec<(String, String)>,
}

/// Defines the free-form network sections used for defining custom attributes for
/// `systemd.netdev` and `NetworkManager`.
///
/// See [`SdSection`] and [`NmSection`] for more information on each.
//
// NOTE: These are stored together in this struct to enforce setting
// both at the same time. This should ensure any provider implementation
// which uses custom attributes will at least consider both formats.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CustomNetworkSections {
    sd_netdev_sections: Vec<SdSection>,
    nm_sections: Vec<NmSection>,
}

impl CustomNetworkSections {
    pub fn new(sd_netdev_sections: Vec<SdSection>, nm_sections: Vec<NmSection>) -> Self {
        Self {
            sd_netdev_sections,
            nm_sections,
        }
    }
}

/// Supported virtual network device kinds.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NetDevKind {
    /// Parent aggregation for physically bonded devices.
    Bond,
    /// VLAN child interface for a physical device with 802.1Q.
    Vlan,
}

impl NetDevKind {
    /// Return device kind according to `systemd.netdev`.
    ///
    /// See [systemd documentation](kinds) for the full list.
    ///
    /// kinds: https://www.freedesktop.org/software/systemd/man/systemd.netdev.html#Supported%20netdev%20kinds
    fn sd_netdev_kind(&self) -> String {
        let kind = match *self {
            NetDevKind::Bond => "bond",
            NetDevKind::Vlan => "vlan",
        };
        kind.to_string()
    }
}

/// Optional use of DHCP.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DhcpSetting {
    Both,
    V4,
    V6,
}

impl DhcpSetting {
    /// Return DHCP setting according to `systemd.network`
    ///
    /// See [systemd documentation](dhcp) for the full list.
    ///
    /// dhcp: https://www.freedesktop.org/software/systemd/man/latest/systemd.network.html#DHCP=
    fn sd_dhcp_setting(&self) -> String {
        let setting = match *self {
            DhcpSetting::Both => "yes",
            DhcpSetting::V4 => "ipv4",
            DhcpSetting::V6 => "ipv6",
        };
        setting.to_string()
    }
}

impl Interface {
    /// Return a deterministic name for this device
    pub fn name(&self) -> Result<String> {
        Ok(match (&self.name, &self.mac_address, &self.path) {
            (Some(ref name), _, _) => name.to_owned(),
            (None, Some(ref addr), _) => addr.to_string(),
            (None, None, Some(ref path)) => path.to_owned(),
            (None, None, None) => bail!("network interface without name, MAC address, or path"),
        })
    }

    /// Return a deterministic `systemd.networkd` unit name for this device.
    pub fn sd_unit_name(&self) -> Result<String> {
        let iface_name = self.name()?;
        Ok(format!("{:02}-{iface_name}.network", self.priority))
    }

    /// Return the `systemd.networkd` configuration for this device.
    pub fn sd_config(&self) -> String {
        let mut config = String::new();

        // [Match] section
        writeln!(config, "[Match]").unwrap();
        if let Some(name) = self.name.clone() {
            writeln!(config, "Name={name}").unwrap();
        }
        if let Some(mac) = self.mac_address {
            writeln!(config, "MACAddress={mac}").unwrap();
        }
        if let Some(path) = &self.path {
            writeln!(config, "Path={path}").unwrap();
        }

        // [Network] section
        writeln!(config, "\n[Network]").unwrap();
        if let Some(dhcp) = &self.dhcp {
            writeln!(config, "DHCP={}", dhcp.sd_dhcp_setting()).unwrap();
        }
        for ns in &self.nameservers {
            writeln!(config, "DNS={ns}").unwrap()
        }
        if let Some(bond) = self.bond.clone() {
            writeln!(config, "Bond={bond}").unwrap();
        }

        // [Link] section
        if self.unmanaged || self.required_for_online.is_some() {
            writeln!(config, "\n[Link]").unwrap();
        }
        if self.unmanaged {
            writeln!(config, "Unmanaged=yes").unwrap();
        }
        if let Some(operational_state) = &self.required_for_online {
            writeln!(config, "RequiredForOnline={operational_state}").unwrap();
        }

        // [Address] sections
        for addr in &self.ip_addresses {
            writeln!(config, "\n[Address]\nAddress={addr}").unwrap();
        }

        // [Route] sections
        for route in &self.routes {
            writeln!(
                config,
                "\n[Route]\nDestination={}\nGateway={}",
                route.destination, route.gateway
            )
            .unwrap();
        }

        config
    }

    /// Return the `NetworkManager` connection profile configuration for this device.
    pub fn nm_config(&self) -> Result<String> {
        let mut config = String::new();

        // [connection] section
        writeln!(config, "[connection]")?;
        let iface_name = self.name()?;
        writeln!(config, "id={}", iface_name)?;
        writeln!(config, "type=ethernet")?;
        // NOTE: Only write interface name if there is no mac address to match against.
        // This is to avoid issues with modern systems which use predictable interface names.
        // e.g. Hetzner's network-config names the primary interface eth0, but on FCOS the
        // interface is instead given a predictable name (enp1s0) and hence won't be matched.
        // See: https://www.freedesktop.org/wiki/Software/systemd/PredictableNetworkInterfaceNames
        if self.mac_address.is_none() {
            if let Some(name) = &self.name {
                writeln!(config, "interface-name={}", name)?;
            }
        }
        writeln!(config, "autoconnect=true")?;
        writeln!(
            config,
            "autoconnect-priority={}",
            // Lower number means higher priority for systemd, but it's the opposite for NM
            100 - (self.priority as u16)
        )?;

        if let Some(ref bond) = self.bond {
            writeln!(config, "master={bond}")?;
            writeln!(config, "slave-type=bond")?;
        }

        // [ethernet] section
        if let Some(ref mac) = self.mac_address {
            writeln!(config, "\n[ethernet]")?;
            writeln!(config, "mac-address={}", mac)?;
        }

        // [ipv4] and [ipv6] sections
        self.write_nm_config_common(&mut config)?;

        Ok(config)
    }

    /// Write NetworkManager configuration that is common to bond masters and other devices to the
    /// given string.
    fn write_nm_config_common(&self, config: &mut String) -> Result<()> {
        // [ipv4] section
        writeln!(config, "\n[ipv4]")?;

        let ipv4_addresses: Vec<_> = self.ip_addresses.iter().filter(|a| a.is_ipv4()).collect();
        if matches!(self.dhcp, Some(DhcpSetting::V4 | DhcpSetting::Both)) {
            writeln!(config, "method=auto")?;
        } else if ipv4_addresses.is_empty() {
            writeln!(config, "method=disabled")?;
        } else {
            writeln!(config, "method=manual")?;
            for (i, addr) in ipv4_addresses.iter().enumerate() {
                writeln!(config, "address{}={}", i + 1, addr)?;
            }

            // IPv4 gateway
            let (default_route, ipv4_routes): (Vec<NetworkRoute>, Vec<NetworkRoute>) = self
                .routes
                .iter()
                .filter(|r| r.destination.is_ipv4())
                .partition(|r| r.destination.prefix() == 0);
            if let Some(default_route) = default_route.first() {
                writeln!(config, "gateway={}", default_route.gateway)?;
            }

            // IPv4 routes (non-default)
            for (i, route) in ipv4_routes.iter().enumerate() {
                writeln!(
                    config,
                    "route{}={},{},0",
                    i + 1,
                    route.destination,
                    route.gateway
                )?;
            }
        }

        // IPv4 DNS servers
        let ipv4_dns: Vec<_> = self.nameservers.iter().filter(|n| n.is_ipv4()).collect();
        if !ipv4_dns.is_empty() {
            let dns_list: Vec<_> = ipv4_dns
                .iter()
                .map(std::string::ToString::to_string)
                .collect();
            writeln!(config, "dns={};", dns_list.join(";"))?;
        }

        // [ipv6] section
        writeln!(config, "\n[ipv6]")?;

        let ipv6_addresses: Vec<_> = self.ip_addresses.iter().filter(|a| a.is_ipv6()).collect();
        if matches!(self.dhcp, Some(DhcpSetting::V6 | DhcpSetting::Both)) {
            writeln!(config, "method=auto")?;
        } else if ipv6_addresses.is_empty() {
            writeln!(config, "method=disabled")?;
        } else {
            writeln!(config, "method=manual")?;
            for (i, addr) in ipv6_addresses.iter().enumerate() {
                writeln!(config, "address{}={}", i + 1, addr)?;
            }

            // IPv6 gateway
            let (default_route, ipv6_routes): (Vec<NetworkRoute>, Vec<NetworkRoute>) = self
                .routes
                .iter()
                .filter(|r| r.destination.is_ipv6())
                .partition(|r| r.destination.prefix() == 0);
            if let Some(default_route) = default_route.first() {
                writeln!(config, "gateway={}", default_route.gateway)?;
            }

            // IPv6 routes (non-default)
            for (i, route) in ipv6_routes.iter().enumerate() {
                writeln!(
                    config,
                    "route{}={},{},0",
                    i + 1,
                    route.destination,
                    route.gateway
                )?;
            }
        }

        // IPv6 DNS servers
        let ipv6_dns: Vec<_> = self.nameservers.iter().filter(|n| n.is_ipv6()).collect();
        if !ipv6_dns.is_empty() {
            let dns_list: Vec<_> = ipv6_dns
                .iter()
                .map(std::string::ToString::to_string)
                .collect();
            writeln!(config, "dns={};", dns_list.join(";"))?;
        }

        Ok(())
    }
}

impl VirtualNetDev {
    /// Return a deterministic netdev unit name for this device.
    pub fn netdev_unit_name(&self) -> String {
        format!("{:02}-{}.netdev", self.priority.unwrap_or(10), self.name)
    }

    /// Return the `systemd.netdev` configuration fragment for this device.
    pub fn sd_netdev_config(&self) -> String {
        let mut config = String::new();

        // [NetDev] section
        writeln!(config, "[NetDev]").unwrap();
        writeln!(config, "Name={}", self.name).unwrap();
        writeln!(config, "Kind={}", self.kind.sd_netdev_kind()).unwrap();
        writeln!(config, "MACAddress={}", self.mac_address).unwrap();

        // Custom sections.
        for section in &self.custom_sections.sd_netdev_sections {
            writeln!(config, "\n[{}]", section.name).unwrap();
            for attr in &section.attributes {
                writeln!(config, "{}={}", attr.0, attr.1).unwrap();
            }
        }

        config
    }

    /// Return the `NetworkManager` connection profile configuration for this virtual device.
    /// Optionally takes a physical interface, which is used for bond network configuration.
    pub fn nm_config(&self, physical_interface: Option<&Interface>) -> Result<String> {
        let mut config = String::new();
        if self.kind == NetDevKind::Bond && physical_interface.is_none() {
            warn!("writing bond configuration without networking information",);
        };

        writeln!(config, "[connection]")?;
        writeln!(config, "id={}", self.name)?;
        writeln!(config, "type={}", self.kind.sd_netdev_kind())?;
        // Unlike in the `Interface` implementation, we don't need to worry about
        // predictable names, as these devices will be created by NetworkManager.
        writeln!(config, "interface-name={}", self.name)?;
        writeln!(config, "autoconnect=true")?;
        if let Some(priority) = self.priority {
            writeln!(
                config,
                "autoconnect-priority={}",
                100i32.saturating_sub_unsigned(priority)
            )?;
        }

        // Bond and VLAN specific configurations
        // See:
        // - https://www.networkmanager.dev/docs/api/latest/settings-vlan.html
        // - https://www.networkmanager.dev/docs/api/latest/settings-bond.html
        // - https://www.kernel.org/doc/html/v5.9/networking/bonding.html
        match self.kind {
            NetDevKind::Bond => {
                writeln!(config, "\n[bond]")?;

                if let Some(section) = self
                    .custom_sections
                    .nm_sections
                    .iter()
                    .find(|s| s.name == "bond")
                {
                    for (key, value) in &section.attributes {
                        writeln!(config, "{}={}", key, value)?;
                    }
                };

                // WARN: does not set mac address when creating the device, only when reloading the configuration
                writeln!(config, "\n[ethernet]")?;
                writeln!(config, "cloned-mac-address={}", self.mac_address)?;

                if let Some(interface) = physical_interface {
                    interface.write_nm_config_common(&mut config)?;
                }
            }
            NetDevKind::Vlan => {
                writeln!(config, "\n[vlan]")?;
                let mut has_parent = false;

                if let Some(section) = self
                    .custom_sections
                    .nm_sections
                    .iter()
                    .find(|s| s.name == "vlan")
                {
                    for (key, value) in &section.attributes {
                        if key == "parent" {
                            has_parent = true;
                        }
                        writeln!(config, "{}={}", key, value)?;
                    }
                };

                // Match parent based on mac-address
                if !has_parent {
                    writeln!(config, "\n[ethernet]")?;
                    writeln!(config, "mac-address={}", self.mac_address)?;
                }
            }
        }

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ipnetwork::{Ipv4Network, Ipv6Network};
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn mac_addr_display() {
        let m = MacAddr(0xf4, 0x00, 0x34, 0x09, 0x73, 0xee);
        assert_eq!(m.to_string(), "f4:00:34:09:73:ee");
    }

    #[test]
    fn interface_unit_name() {
        let cases = vec![
            (
                Interface {
                    name: Some(String::from("lo")),
                    mac_address: Some(MacAddr(0, 0, 0, 0, 0, 0)),
                    path: None,
                    priority: 20,
                    nameservers: vec![],
                    ip_addresses: vec![],
                    dhcp: None,
                    routes: vec![],
                    bond: None,
                    unmanaged: false,
                    required_for_online: None,
                },
                "20-lo.network",
            ),
            (
                Interface {
                    name: Some(String::from("lo")),
                    mac_address: Some(MacAddr(0, 0, 0, 0, 0, 0)),
                    path: None,
                    priority: 10,
                    nameservers: vec![],
                    ip_addresses: vec![],
                    dhcp: None,
                    routes: vec![],
                    bond: None,
                    unmanaged: false,
                    required_for_online: None,
                },
                "10-lo.network",
            ),
            (
                Interface {
                    name: None,
                    mac_address: Some(MacAddr(0, 0, 0, 0, 0, 0)),
                    path: None,
                    priority: 20,
                    nameservers: vec![],
                    ip_addresses: vec![],
                    dhcp: None,
                    routes: vec![],
                    bond: None,
                    unmanaged: false,
                    required_for_online: None,
                },
                "20-00:00:00:00:00:00.network",
            ),
            (
                Interface {
                    name: Some(String::from("lo")),
                    mac_address: None,
                    path: None,
                    priority: 20,
                    nameservers: vec![],
                    ip_addresses: vec![],
                    dhcp: None,
                    routes: vec![],
                    bond: None,
                    unmanaged: false,
                    required_for_online: None,
                },
                "20-lo.network",
            ),
            (
                Interface {
                    name: None,
                    mac_address: None,
                    path: Some("pci-*".to_owned()),
                    priority: 20,
                    nameservers: vec![],
                    ip_addresses: vec![],
                    dhcp: None,
                    routes: vec![],
                    bond: None,
                    unmanaged: false,
                    required_for_online: None,
                },
                "20-pci-*.network",
            ),
        ];

        for (iface, expected) in cases {
            let unit_name = iface.sd_unit_name().unwrap();
            assert_eq!(unit_name, expected);
        }
    }

    #[test]
    fn interface_unit_name_no_name_no_mac() {
        let i = Interface {
            name: None,
            mac_address: None,
            path: None,
            priority: 20,
            nameservers: vec![],
            ip_addresses: vec![],
            dhcp: None,
            routes: vec![],
            bond: None,
            unmanaged: false,
            required_for_online: None,
        };
        i.sd_unit_name().unwrap_err();
    }

    #[test]
    fn virtual_netdev_unit_name() {
        let ds = vec![
            (
                VirtualNetDev {
                    name: String::from("vlan0"),
                    kind: NetDevKind::Vlan,
                    mac_address: MacAddr(0, 0, 0, 0, 0, 0),
                    priority: Some(20),
                    custom_sections: CustomNetworkSections::default(),
                },
                "20-vlan0.netdev",
            ),
            (
                VirtualNetDev {
                    name: String::from("vlan0"),
                    kind: NetDevKind::Vlan,
                    mac_address: MacAddr(0, 0, 0, 0, 0, 0),
                    priority: None,
                    custom_sections: CustomNetworkSections::default(),
                },
                "10-vlan0.netdev",
            ),
        ];

        for (d, s) in ds {
            assert_eq!(d.netdev_unit_name(), s);
        }
    }

    #[test]
    fn interface_sd_config() {
        let is = vec![
            (
                Interface {
                    name: Some(String::from("lo")),
                    mac_address: Some(MacAddr(0, 0, 0, 0, 0, 0)),
                    path: None,
                    priority: 20,
                    nameservers: vec![
                        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                        IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)),
                    ],
                    ip_addresses: vec![
                        IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(127, 0, 0, 1), 8).unwrap()),
                        IpNetwork::V6(
                            Ipv6Network::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), 128).unwrap(),
                        ),
                    ],
                    dhcp: None,
                    routes: vec![NetworkRoute {
                        destination: IpNetwork::V4(
                            Ipv4Network::new(Ipv4Addr::new(127, 0, 0, 1), 8).unwrap(),
                        ),
                        gateway: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                    }],
                    bond: Some(String::from("james")),
                    unmanaged: false,
                    required_for_online: None,
                },
                "[Match]
Name=lo
MACAddress=00:00:00:00:00:00

[Network]
DNS=127.0.0.1
DNS=::1
Bond=james

[Address]
Address=127.0.0.1/8

[Address]
Address=::1/128

[Route]
Destination=127.0.0.1/8
Gateway=127.0.0.1
",
            ),
            // this isn't really a valid interface object, but it's testing
            // the minimum possible configuration for all pieces at the same
            // time, so I'll allow it. (sdemos)
            (
                Interface {
                    name: None,
                    mac_address: None,
                    path: None,
                    priority: 10,
                    nameservers: vec![],
                    ip_addresses: vec![],
                    dhcp: None,
                    routes: vec![],
                    bond: None,
                    unmanaged: false,
                    required_for_online: None,
                },
                "[Match]

[Network]
",
            ),
            // test the path and required_for_online settings
            (
                Interface {
                    name: None,
                    mac_address: None,
                    path: Some("pci-*".to_owned()),
                    priority: 10,
                    nameservers: vec![],
                    ip_addresses: vec![],
                    dhcp: None,
                    routes: vec![],
                    bond: None,
                    unmanaged: false,
                    required_for_online: Some("no".to_owned()),
                },
                "[Match]
Path=pci-*

[Network]

[Link]
RequiredForOnline=no
",
            ),
            // test the unmanaged setting
            (
                Interface {
                    name: Some("*".to_owned()),
                    mac_address: None,
                    path: None,
                    priority: 10,
                    nameservers: vec![],
                    ip_addresses: vec![],
                    dhcp: None,
                    routes: vec![],
                    bond: None,
                    unmanaged: true,
                    required_for_online: None,
                },
                "[Match]
Name=*

[Network]

[Link]
Unmanaged=yes
",
            ),
            // test the DHCP setting
            (
                Interface {
                    name: Some("*".to_owned()),
                    mac_address: None,
                    path: None,
                    priority: 10,
                    nameservers: vec![],
                    ip_addresses: vec![],
                    dhcp: Some(DhcpSetting::V4),
                    routes: vec![],
                    bond: None,
                    unmanaged: false,
                    required_for_online: None,
                },
                "[Match]
Name=*

[Network]
DHCP=ipv4
",
            ),
        ];

        for (i, s) in is {
            assert_eq!(i.sd_config(), s);
        }
    }

    #[test]
    fn virtual_netdev_sd_config() {
        let ds = vec![
            (
                VirtualNetDev {
                    name: String::from("vlan0"),
                    kind: NetDevKind::Vlan,
                    mac_address: MacAddr(0, 0, 0, 0, 0, 0),
                    priority: Some(20),
                    custom_sections: CustomNetworkSections::new(
                        vec![
                            SdSection {
                                name: String::from("Test"),
                                attributes: vec![
                                    (String::from("foo"), String::from("bar")),
                                    (String::from("oingo"), String::from("boingo")),
                                ],
                            },
                            SdSection {
                                name: String::from("Empty"),
                                attributes: vec![],
                            },
                        ],
                        vec![],
                    ),
                },
                "[NetDev]
Name=vlan0
Kind=vlan
MACAddress=00:00:00:00:00:00

[Test]
foo=bar
oingo=boingo

[Empty]
",
            ),
            (
                VirtualNetDev {
                    name: String::from("vlan0"),
                    kind: NetDevKind::Vlan,
                    mac_address: MacAddr(0, 0, 0, 0, 0, 0),
                    priority: Some(20),
                    custom_sections: CustomNetworkSections::default(),
                },
                "[NetDev]
Name=vlan0
Kind=vlan
MACAddress=00:00:00:00:00:00
",
            ),
        ];

        for (d, s) in ds {
            assert_eq!(d.sd_netdev_config(), s);
        }
    }

    #[test]
    fn interface_nm_config() {
        let ds = vec![(
            Interface {
                name: Some(String::from("eth0")),
                mac_address: Some(MacAddr(0, 0, 0, 0, 0, 0)),
                path: None,
                priority: 0,
                nameservers: vec![
                    IpAddr::V6(Ipv6Addr::new(0x2a01, 0x4ff, 0xff00, 0, 0, 0, 0x0add, 2)),
                    IpAddr::V6(Ipv6Addr::new(0x2a01, 0x4ff, 0xff00, 0, 0, 0, 0x0add, 1)),
                ],
                ip_addresses: vec![IpNetwork::V6(
                    Ipv6Network::new(Ipv6Addr::new(0x2a01, 0x4f9, 0xc014, 0x6d3b, 0, 0, 0, 1), 64)
                        .unwrap(),
                )],
                dhcp: Some(DhcpSetting::V4),
                routes: vec![NetworkRoute {
                    destination: IpNetwork::V6(
                        Ipv6Network::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), 0).unwrap(),
                    ),
                    gateway: IpAddr::V6(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1)),
                }],
                bond: None,
                unmanaged: false,
                required_for_online: None,
            },
            "[connection]
id=eth0
type=ethernet
autoconnect=true
autoconnect-priority=100

[ethernet]
mac-address=00:00:00:00:00:00

[ipv4]
method=auto

[ipv6]
method=manual
address1=2a01:4f9:c014:6d3b::1/64
gateway=fe80::1
dns=2a01:4ff:ff00::add:2;2a01:4ff:ff00::add:1;
",
        )];

        for (d, s) in ds {
            assert_eq!(d.nm_config().unwrap(), s);
        }
    }

    #[test]
    fn virtual_netdev_nm_config() {
        let interface = Interface {
            name: Some(String::from("bond0")),
            mac_address: Some(MacAddr(0, 0, 0, 0, 0, 0)),
            path: None,
            priority: 0,
            nameservers: vec![],
            ip_addresses: vec![],
            dhcp: Some(DhcpSetting::V4),
            routes: vec![],
            bond: None,
            unmanaged: false,
            required_for_online: None,
        };

        let dis = vec![
            (
                VirtualNetDev {
                    name: String::from("bond0"),
                    kind: NetDevKind::Bond,
                    mac_address: MacAddr(0, 0, 0, 0, 0, 0),
                    priority: Some(20),
                    custom_sections: CustomNetworkSections::new(
                        vec![],
                        vec![NmSection {
                            name: String::from("bond"),
                            attributes: vec![
                                (
                                    String::from("mode"),
                                    bonding_mode_to_string(BONDING_MODE_BALANCE_RR).unwrap(),
                                ),
                                (String::from("miimon"), String::from("100")),
                                (String::from("lp_interval"), String::from("2")),
                                (String::from("arp_validate"), String::from("backup")),
                                (String::from("all_slaves_active"), String::from("1")),
                                (String::from("xmit_hash_policy"), String::from("layer2")),
                            ],
                        }],
                    ),
                },
                Some(&interface),
                "[connection]
id=bond0
type=bond
interface-name=bond0
autoconnect=true
autoconnect-priority=80

[bond]
mode=balance-rr
miimon=100
lp_interval=2
arp_validate=backup
all_slaves_active=1
xmit_hash_policy=layer2

[ethernet]
cloned-mac-address=00:00:00:00:00:00

[ipv4]
method=auto

[ipv6]
method=disabled
",
            ),
            (
                VirtualNetDev {
                    name: String::from("vlan0"),
                    kind: NetDevKind::Vlan,
                    mac_address: MacAddr(0, 0, 0, 0, 0, 0),
                    priority: Some(20),
                    custom_sections: CustomNetworkSections::new(
                        vec![],
                        vec![NmSection {
                            name: String::from("vlan"),
                            attributes: vec![
                                (String::from("id"), String::from("100")),
                                (String::from("ingress-priority-map"), String::from("25:5")),
                                (String::from("protocol"), String::from("802.1ad")),
                                (String::from("flags"), String::from("6")),
                            ],
                        }],
                    ),
                },
                None,
                "[connection]
id=vlan0
type=vlan
interface-name=vlan0
autoconnect=true
autoconnect-priority=80

[vlan]
id=100
ingress-priority-map=25:5
protocol=802.1ad
flags=6

[ethernet]
mac-address=00:00:00:00:00:00
",
            ),
        ];

        for (d, i, s) in dis {
            assert_eq!(d.nm_config(i).unwrap(), s);
        }
    }
}
