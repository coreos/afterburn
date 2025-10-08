use anyhow::Result;
use ipnetwork::IpNetwork;
use pnet_base::MacAddr;
use serde::Deserialize;
use slog_scope::warn;
use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;

use crate::network::{self, DhcpSetting, NetworkRoute};

/// OpenStack network metadata format for `network_data.json`
///
/// This format follows the OpenStack network metadata specification as described
/// in nova-specs and is used by OpenStack to provide network configuration
/// information to instances.
#[derive(Debug, Deserialize)]
pub struct NetworkData {
    /// Network links (interfaces)
    #[serde(default)]
    pub links: Vec<NetworkLink>,
    /// Network configurations
    #[serde(default)]
    pub networks: Vec<NetworkConfig>,
    /// Network services (DNS, etc.)
    #[serde(default)]
    pub services: Vec<NetworkService>,
}

/// Network link configuration (interface definition)
///
/// Describes a network interface with its physical properties.
#[derive(Debug, Deserialize)]
pub struct NetworkLink {
    /// Unique identifier for this link
    pub id: String,
    /// Interface name (if provided)
    pub name: Option<String>,
    /// Type of link: "vif", "phy", "bond", or "vlan"
    #[serde(rename = "type")]
    #[allow(dead_code)]
    pub link_type: String,
    /// MAC address of the interface
    pub ethernet_mac_address: Option<String>,
    /// Maximum transmission unit
    #[allow(dead_code)]
    pub mtu: Option<u16>,
    /// VIF ID for virtual interfaces
    #[allow(dead_code)]
    pub vif_id: Option<String>,
}

/// Network configuration
///
/// Defines IP configuration for a network interface.
#[derive(Debug, Deserialize)]
pub struct NetworkConfig {
    /// Network identifier
    #[allow(dead_code)]
    pub id: String,
    /// Network type: "ipv4", "ipv6", "ipv4_dhcp", "ipv6_dhcp", etc.
    #[serde(rename = "type")]
    pub network_type: String,
    /// Reference to the link this network applies to
    pub link: String,
    /// Static IP address
    pub ip_address: Option<String>,
    /// Network mask (for IPv4) or prefix length (for IPv6)
    pub netmask: Option<String>,
    /// Network routes
    #[serde(default)]
    pub routes: Vec<OpenStackRoute>,
    /// DNS nameservers
    #[serde(default)]
    pub dns_nameservers: Vec<String>,
    /// Network ID in OpenStack
    #[allow(dead_code)]
    pub network_id: Option<String>,
}

/// Network service configuration
///
/// Describes network services like DNS servers.
#[derive(Debug, Deserialize)]
pub struct NetworkService {
    /// Service type (e.g., "dns")
    #[serde(rename = "type")]
    pub service_type: String,
    /// Service address
    pub address: String,
}

/// Network route configuration
///
/// Defines routing information for networks following OpenStack metadata format.
#[derive(Debug, Deserialize)]
pub struct OpenStackRoute {
    /// Network address (e.g., "10.0.0.0", "0.0.0.0", "::")
    pub network: String,
    /// Network mask (e.g., "255.0.0.0", "255.255.255.0", "::")
    pub netmask: String,
    /// Gateway IP address
    pub gateway: String,
    /// Route metric (priority)
    #[allow(dead_code)]
    pub metric: Option<u32>,
}

impl NetworkData {
    /// Convert OpenStack network data to interface configurations
    ///
    /// This processes the OpenStack network metadata format and converts it
    /// into afterburn's common network::Interface format for generating
    /// dracut kernel arguments.
    pub fn to_interfaces(&self) -> Result<Vec<network::Interface>> {
        let mut interfaces = Vec::new();
        let mut link_map: HashMap<String, &NetworkLink> = HashMap::new();

        // Build a map of link IDs to link objects for easy lookup
        for link in &self.links {
            link_map.insert(link.id.clone(), link);
        }

        // Group networks by link to create interfaces
        let mut link_networks: HashMap<String, Vec<&NetworkConfig>> = HashMap::new();
        for network in &self.networks {
            link_networks
                .entry(network.link.clone())
                .or_default()
                .push(network);
        }

        // Create interfaces from links and their associated networks
        for (link_id, networks) in link_networks {
            if let Some(link) = link_map.get(&link_id) {
                let interface = self.create_interface_from_link_and_networks(link, &networks)?;
                interfaces.push(interface);
            }
        }

        // Sort interfaces by name to ensure consistent ordering
        // Put named interfaces first, then unnamed ones
        interfaces.sort_by(|a, b| match (&a.name, &b.name) {
            (Some(name_a), Some(name_b)) => name_a.cmp(name_b),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });

        Ok(interfaces)
    }

    fn create_interface_from_link_and_networks(
        &self,
        link: &NetworkLink,
        networks: &[&NetworkConfig],
    ) -> Result<network::Interface> {
        let mut iface = network::Interface {
            name: link.name.clone(),
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

        // Set MAC address if available
        if let Some(mac) = &link.ethernet_mac_address {
            iface.mac_address = Some(MacAddr::from_str(mac)?);
        }

        let mut has_dhcp4 = false;
        let mut has_dhcp6 = false;
        let mut all_nameservers = Vec::new();

        // Process each network configuration for this link
        for network in networks {
            match network.network_type.as_str() {
                "ipv4" => {
                    if let Some(ip_str) = &network.ip_address {
                        let ip_network = if let Some(netmask) = &network.netmask {
                            let ip_addr = IpAddr::from_str(ip_str)?;
                            if let Ok(netmask_addr) = IpAddr::from_str(netmask) {
                                IpNetwork::with_netmask(ip_addr, netmask_addr)?
                            } else if let Ok(prefix_len) = netmask.parse::<u8>() {
                                IpNetwork::new(ip_addr, prefix_len)?
                            } else {
                                return Err(anyhow::anyhow!(
                                    "Invalid netmask format: {}. Expected IP address or prefix length.",
                                    netmask
                                ));
                            }
                        } else {
                            IpNetwork::from_str(ip_str)?
                        };
                        iface.ip_addresses.push(ip_network);
                    }
                }
                "ipv6" => {
                    if let Some(ip_str) = &network.ip_address {
                        let ip_network = if let Some(netmask) = &network.netmask {
                            let ip_addr = IpAddr::from_str(ip_str)?;
                            if let Ok(prefix_len) = netmask.parse::<u8>() {
                                IpNetwork::new(ip_addr, prefix_len)?
                            } else {
                                IpNetwork::with_netmask(ip_addr, IpAddr::from_str(netmask)?)?
                            }
                        } else {
                            IpNetwork::from_str(ip_str)?
                        };
                        iface.ip_addresses.push(ip_network);
                    }
                }
                "ipv4_dhcp" => {
                    has_dhcp4 = true;
                }
                "ipv6_dhcp" => {
                    has_dhcp6 = true;
                }
                _ => {
                    warn!("Unsupported network type: {}", network.network_type);
                }
            }

            // Collect nameservers
            for ns in &network.dns_nameservers {
                let nameserver = IpAddr::from_str(ns)?;
                if !all_nameservers.contains(&nameserver) {
                    all_nameservers.push(nameserver);
                }
            }

            // Process routes
            for route in &network.routes {
                // Handle network and netmask according to OpenStack schema
                let destination = if route.network == "0.0.0.0" && route.netmask == "0.0.0.0" {
                    // Default IPv4 route
                    IpNetwork::from_str("0.0.0.0/0")?
                } else if route.network == "::" && route.netmask == "::" {
                    // Default IPv6 route
                    IpNetwork::from_str("::/0")?
                } else {
                    // Calculate prefix length from netmask for proper CIDR notation
                    let network_addr = IpAddr::from_str(&route.network)?;
                    if let Ok(netmask_addr) = IpAddr::from_str(&route.netmask) {
                        IpNetwork::with_netmask(network_addr, netmask_addr)?
                    } else if let Ok(prefix_len) = route.netmask.parse::<u8>() {
                        IpNetwork::new(network_addr, prefix_len)?
                    } else {
                        // For IPv6, netmask might be in full format like "ffff:ffff:ffff:ffff::"
                        if network_addr.is_ipv6() && route.netmask == "ffff:ffff:ffff:ffff::" {
                            IpNetwork::new(network_addr, 64)?
                        } else {
                            return Err(anyhow::anyhow!(
                                "Invalid netmask format: {}. Expected IP address or prefix length.",
                                route.netmask
                            ));
                        }
                    }
                };
                let gateway = IpAddr::from_str(&route.gateway)?;
                iface.routes.push(NetworkRoute {
                    destination,
                    gateway,
                });
            }
        }

        // Set DHCP configuration
        iface.dhcp = match (has_dhcp4, has_dhcp6) {
            (true, true) => Some(DhcpSetting::Both),
            (true, false) => Some(DhcpSetting::V4),
            (false, true) => Some(DhcpSetting::V6),
            (false, false) => None,
        };

        // Add global DNS servers from services (per OpenStack schema)
        for service in &self.services {
            if service.service_type == "dns" {
                let nameserver = IpAddr::from_str(&service.address)?;
                if !all_nameservers.contains(&nameserver) {
                    all_nameservers.push(nameserver);
                }
            }
        }

        iface.nameservers = all_nameservers;

        Ok(iface)
    }
}

/// Transform OpenStack network data into a set of interface configurations.
///
/// This is the main entry point for converting parsed OpenStack network data into
/// afterburn's common interface format for generating dracut kernel arguments.
pub fn network_interfaces(input: &NetworkData) -> Result<Vec<network::Interface>> {
    input.to_interfaces()
}
