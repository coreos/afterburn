//! Cloud-init NoCloud datasource support for network configuration.
//!
//! This module implements parsing for cloud-init network-config files
//! following the NoCloud datasource specification. It supports both
//! Network Config v1 and v2 formats, in either JSON or YAML format.
//!
//! Reference: https://cloudinit.readthedocs.io/en/latest/reference/datasources/nocloud.html

use crate::network::{self, DhcpSetting, NetworkRoute};
use anyhow::{Context, Result};
use ipnetwork::IpNetwork;
use pnet_base::MacAddr;
use serde::Deserialize;
use slog_scope::warn;
use std::{collections::HashMap, fs::File, io::BufReader, net::IpAddr, path::Path, str::FromStr};

pub fn read_config_file(path: &Path, file: &str) -> Result<Option<BufReader<File>>> {
    let filename = path.join(file);
    if !filename.exists() {
        return Ok(None);
    }
    let file =
        File::open(&filename).with_context(|| format!("failed to open file '{filename:?}'"))?;
    Ok(Some(BufReader::new(file)))
}

/// Cloud-init Network Config format wrapper
///
/// This can be either v1 or v2 format
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum NetworkConfig {
    V1(NetworkConfigV1),
    V2(NetworkConfigV2),
}

/// Network Config v1 format
///
/// Used by cloud-init for network configuration
#[derive(Debug, Deserialize)]
pub struct NetworkConfigV1 {
    /// Version number (should be 1)
    #[serde(default)]
    #[allow(dead_code)]
    pub version: Option<u8>,
    /// List of network configuration entries
    pub config: Vec<NetworkConfigV1Entry>,
}

/// Network Config v1 entry
#[derive(Debug, Deserialize)]
pub struct NetworkConfigV1Entry {
    /// Type of network config: "physical", "nameserver", etc.
    #[serde(rename = "type")]
    pub network_type: String,
    /// Interface name
    pub name: Option<String>,
    /// MAC address
    pub mac_address: Option<String>,
    /// Static IP addresses
    #[serde(default)]
    pub address: Vec<String>,
    /// Subnet configurations
    #[serde(default)]
    pub subnets: Vec<NetworkConfigV1Subnet>,
}

/// Route configuration in v1 format
#[derive(Debug, Deserialize)]
pub struct RouteConfigV1 {
    /// Destination network
    pub network: String,
    /// Netmask for the destination network
    pub netmask: String,
    /// Gateway address
    pub gateway: String,
}

/// Network Config v1 subnet
#[derive(Debug, Deserialize)]
pub struct NetworkConfigV1Subnet {
    /// Type of subnet: "static", "dhcp", "dhcp4", "dhcp6", etc.
    #[serde(rename = "type")]
    pub subnet_type: String,
    /// IP address (for static configuration)
    pub address: Option<String>,
    /// Netmask (for static configuration)
    pub netmask: Option<String>,
    /// Gateway (for static configuration)
    pub gateway: Option<String>,
    /// DNS nameservers
    #[serde(default)]
    pub dns_nameservers: Vec<String>,
    /// Routes (for static configuration)
    #[serde(default)]
    pub routes: Vec<RouteConfigV1>,
}

/// Network Config v2 format
///
/// More modern format used by cloud-init and netplan
#[derive(Debug, Deserialize)]
pub struct NetworkConfigV2 {
    /// Version number (should be 2)
    #[serde(default)]
    #[allow(dead_code)]
    pub version: Option<u8>,
    /// Ethernet interfaces configuration
    #[serde(default)]
    pub ethernets: HashMap<String, EthernetConfigV2>,
    /// Global nameservers configuration
    pub nameservers: Option<NameserversConfig>,
}

/// DHCP overrides configuration
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct DhcpOverrides {
    /// Ignore DNS from DHCP
    #[serde(rename = "use-dns", default)]
    pub use_dns: Option<bool>,
    /// Ignore routes from DHCP
    #[serde(rename = "use-routes", default)]
    pub use_routes: Option<bool>,
    /// Ignore domain settings from DHCP
    #[serde(rename = "use-domains", default)]
    pub use_domains: Option<bool>,
    /// Ignore hostname from DHCP
    #[serde(rename = "use-hostname", default)]
    pub use_hostname: Option<bool>,
    /// Ignore NTP from DHCP
    #[serde(rename = "use-ntp", default)]
    pub use_ntp: Option<bool>,
    /// Override route metric
    #[serde(rename = "route-metric", default)]
    pub route_metric: Option<u32>,
}

/// Ethernet interface configuration in v2 format
#[derive(Debug, Deserialize)]
pub struct EthernetConfigV2 {
    /// DHCP for IPv4
    #[serde(default)]
    pub dhcp4: bool,
    /// DHCP for IPv6
    #[serde(default)]
    pub dhcp6: bool,
    /// DHCP overrides for IPv4
    #[serde(rename = "dhcp4-overrides")]
    #[allow(dead_code)]
    pub dhcp4_overrides: Option<DhcpOverrides>,
    /// DHCP overrides for IPv6
    #[serde(rename = "dhcp6-overrides")]
    #[allow(dead_code)]
    pub dhcp6_overrides: Option<DhcpOverrides>,
    /// Static IP addresses in CIDR notation
    #[serde(default)]
    pub addresses: Vec<String>,
    /// Gateway for IPv4
    pub gateway4: Option<String>,
    /// Gateway for IPv6
    pub gateway6: Option<String>,
    /// MAC address
    #[serde(rename = "match")]
    pub match_config: Option<MatchConfig>,
    /// Nameservers configuration
    pub nameservers: Option<NameserversConfig>,
    /// Routes configuration
    #[serde(default)]
    pub routes: Vec<RouteConfigV2>,
}

/// Match configuration for identifying interfaces
#[derive(Debug, Deserialize)]
pub struct MatchConfig {
    /// MAC address to match
    pub macaddress: Option<String>,
    /// Interface name to match
    #[allow(dead_code)]
    pub name: Option<String>,
}

/// Nameservers configuration
#[derive(Debug, Deserialize)]
pub struct NameserversConfig {
    /// List of nameserver addresses
    #[serde(default)]
    pub addresses: Vec<String>,
}

/// Route configuration in v2 format
#[derive(Debug, Deserialize)]
pub struct RouteConfigV2 {
    /// Destination network in CIDR notation
    pub to: String,
    /// Gateway address
    pub via: String,
}

impl NetworkConfig {
    /// Parse network-config file from a path
    ///
    /// Supports both JSON and YAML formats, automatically detecting which is used
    pub fn from_file(path: &Path) -> Result<Option<Self>> {
        let network_config_path = path.join("network-config");

        if !network_config_path.exists() {
            return Ok(None);
        }

        let file =
            File::open(&network_config_path).context("failed to open network-config file")?;
        let reader = BufReader::new(file);

        // serde_yaml can parse both YAML and JSON
        let config: NetworkConfig =
            serde_yaml::from_reader(reader).context("failed to parse network-config file")?;

        Ok(Some(config))
    }

    /// Convert to network interfaces
    pub fn to_interfaces(&self) -> Result<Vec<network::Interface>> {
        match self {
            NetworkConfig::V1(v1) => v1.to_interfaces(),
            NetworkConfig::V2(v2) => v2.to_interfaces(),
        }
    }
}

impl NetworkConfigV1 {
    /// Convert v1 config to network interfaces
    pub fn to_interfaces(&self) -> Result<Vec<network::Interface>> {
        let nameservers = self
            .config
            .iter()
            .filter(|config| config.network_type == "nameserver")
            .collect::<Vec<_>>();

        if nameservers.len() > 1 {
            warn!("multiple nameserver entries found, using first one");
        }

        let mut interfaces = self
            .config
            .iter()
            .filter(|config| config.network_type == "physical")
            .map(|entry| entry.to_interface())
            .collect::<Result<Vec<_>, _>>()?;

        // Collect global nameservers
        let global_nameservers: Vec<IpAddr> = if let Some(nameserver) = nameservers.first() {
            nameserver
                .address
                .iter()
                .map(|ip| IpAddr::from_str(ip))
                .collect::<Result<Vec<IpAddr>, _>>()?
        } else {
            Vec::new()
        };

        // Add global nameservers to all interfaces
        for iface in &mut interfaces {
            iface.nameservers.extend(global_nameservers.iter().copied());
        }

        Ok(interfaces)
    }
}

impl NetworkConfigV1Entry {
    /// Convert a v1 config entry to an interface
    pub fn to_interface(&self) -> Result<network::Interface> {
        if self.network_type != "physical" {
            return Err(anyhow::anyhow!(
                "cannot convert config to interface: unsupported config type \"{}\"",
                self.network_type
            ));
        }

        let mut iface = network::Interface {
            name: self.name.clone(),
            nameservers: vec![],
            ip_addresses: vec![],
            routes: vec![],
            dhcp: None,
            mac_address: None,
            bond: None,
            path: None,
            priority: 20,
            unmanaged: false,
            required_for_online: None,
        };

        // Process subnets
        for subnet in &self.subnets {
            // Collect nameservers from subnets
            for ns in &subnet.dns_nameservers {
                let nameserver = IpAddr::from_str(ns)?;
                if !iface.nameservers.contains(&nameserver) {
                    iface.nameservers.push(nameserver);
                }
            }

            // Handle static configuration
            if subnet.subnet_type.contains("static") {
                // Static subnet may have an IP address, or just routes/DNS configuration
                if let Some(address) = &subnet.address {
                    if let Some(netmask) = &subnet.netmask {
                        let ip_addr = IpAddr::from_str(address)?;
                        // Try to parse netmask as IP address first, then as prefix length
                        let ip_network = if let Ok(netmask_addr) = IpAddr::from_str(netmask) {
                            IpNetwork::with_netmask(ip_addr, netmask_addr)?
                        } else if let Ok(prefix_len) = netmask.parse::<u8>() {
                            IpNetwork::new(ip_addr, prefix_len)?
                        } else {
                            return Err(anyhow::anyhow!(
                                "Invalid netmask format: {}. Expected IP address or prefix length.",
                                netmask
                            ));
                        };
                        iface.ip_addresses.push(ip_network);
                    } else {
                        iface.ip_addresses.push(IpNetwork::from_str(address)?);
                    }
                }
            } else if subnet.subnet_type == "dhcp" || subnet.subnet_type == "dhcp4" {
                iface.dhcp = match iface.dhcp {
                    Some(DhcpSetting::V6) => Some(DhcpSetting::Both),
                    _ => Some(DhcpSetting::V4),
                };
            } else if subnet.subnet_type == "dhcp6" {
                iface.dhcp = match iface.dhcp {
                    Some(DhcpSetting::V4) => Some(DhcpSetting::Both),
                    _ => Some(DhcpSetting::V6),
                };
            } else {
                warn!(
                    "subnet type \"{}\" not supported, ignoring",
                    subnet.subnet_type
                );
            }

            // Handle routes from subnet
            // First, process any routes defined in the subnet's routes array
            for route in &subnet.routes {
                let gateway = IpAddr::from_str(&route.gateway)?;

                // Parse the destination network
                let network = IpAddr::from_str(&route.network)?;
                let netmask = IpAddr::from_str(&route.netmask)?;

                let destination = IpNetwork::with_netmask(network, netmask)?;

                iface.routes.push(NetworkRoute {
                    destination,
                    gateway,
                });
            }

            // Then, handle legacy gateway field (for backwards compatibility)
            if let Some(gateway) = &subnet.gateway {
                let gateway = IpAddr::from_str(gateway)?;

                let destination = if gateway.is_ipv6() {
                    IpNetwork::from_str("::/0")?
                } else {
                    IpNetwork::from_str("0.0.0.0/0")?
                };

                iface.routes.push(NetworkRoute {
                    destination,
                    gateway,
                });
            }
        }

        // Set MAC address if available
        if let Some(mac) = &self.mac_address {
            iface.mac_address = Some(MacAddr::from_str(mac)?);
        }

        Ok(iface)
    }
}

impl NetworkConfigV2 {
    /// Convert v2 config to network interfaces
    pub fn to_interfaces(&self) -> Result<Vec<network::Interface>> {
        let mut interfaces = Vec::new();

        for (key, config) in &self.ethernets {
            // Determine the interface name:
            // - Use the key as name unless there's a MAC match without a name
            // - If there's a MAC match and the key looks like an arbitrary ID, set name to None
            let interface_name = if config.match_config.is_some() && !key.starts_with("eth") {
                None
            } else {
                Some(key.clone())
            };

            let mut iface = network::Interface {
                name: interface_name,
                nameservers: vec![],
                ip_addresses: vec![],
                routes: vec![],
                dhcp: None,
                mac_address: None,
                bond: None,
                path: None,
                priority: 20,
                unmanaged: false,
                required_for_online: None,
            };

            // Set DHCP
            iface.dhcp = match (config.dhcp4, config.dhcp6) {
                (true, true) => Some(DhcpSetting::Both),
                (true, false) => Some(DhcpSetting::V4),
                (false, true) => Some(DhcpSetting::V6),
                (false, false) => None,
            };

            // Set static addresses
            for addr_str in &config.addresses {
                iface.ip_addresses.push(IpNetwork::from_str(addr_str)?);
            }

            // Set gateways as default routes
            if let Some(gateway4) = &config.gateway4 {
                iface.routes.push(NetworkRoute {
                    destination: IpNetwork::from_str("0.0.0.0/0")?,
                    gateway: IpAddr::from_str(gateway4)?,
                });
            }
            if let Some(gateway6) = &config.gateway6 {
                iface.routes.push(NetworkRoute {
                    destination: IpNetwork::from_str("::/0")?,
                    gateway: IpAddr::from_str(gateway6)?,
                });
            }

            // Process explicit routes
            for route in &config.routes {
                iface.routes.push(NetworkRoute {
                    destination: IpNetwork::from_str(&route.to)?,
                    gateway: IpAddr::from_str(&route.via)?,
                });
            }

            // Set nameservers
            if let Some(nameservers) = &config.nameservers {
                iface.nameservers = nameservers
                    .addresses
                    .iter()
                    .map(|ns| IpAddr::from_str(ns))
                    .collect::<Result<Vec<_>, _>>()?;
            }

            // Set MAC address from match config
            if let Some(match_config) = &config.match_config {
                if let Some(mac) = &match_config.macaddress {
                    iface.mac_address = Some(MacAddr::from_str(mac)?);
                }
            }

            interfaces.push(iface);
        }

        // Sort interfaces by name for consistent ordering
        // Put named interfaces first, then unnamed ones
        interfaces.sort_by(|a, b| match (&a.name, &b.name) {
            (Some(name_a), Some(name_b)) => name_a.cmp(name_b),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });

        // Add global nameservers to all interfaces
        if let Some(global_nameservers) = &self.nameservers {
            let nameserver_addrs: Vec<IpAddr> = global_nameservers
                .addresses
                .iter()
                .map(|ns| IpAddr::from_str(ns))
                .collect::<Result<Vec<_>, _>>()?;

            for iface in &mut interfaces {
                for ns in &nameserver_addrs {
                    if !iface.nameservers.contains(ns) {
                        iface.nameservers.push(*ns);
                    }
                }
            }
        }

        Ok(interfaces)
    }
}
