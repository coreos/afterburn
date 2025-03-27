use crate::{
    network::{self, DhcpSetting, NetworkRoute},
    providers::MetadataProvider,
};
use anyhow::{Context, Result};
use ipnetwork::IpNetwork;
use openssh_keys::PublicKey;
use pnet_base::MacAddr;
use serde::Deserialize;
use slog_scope::warn;
use std::{
    collections::HashMap,
    fs::File,
    net::{AddrParseError, IpAddr},
    path::Path,
    str::FromStr,
};

#[derive(Debug)]
pub struct ProxmoxVECloudConfig {
    pub meta_data: ProxmoxVECloudMetaData,
    pub user_data: Option<ProxmoxVECloudUserData>,
    #[allow(dead_code)]
    pub vendor_data: ProxmoxVECloudVendorData,
    pub network_config: ProxmoxVECloudNetworkConfig,
}

#[derive(Debug, Deserialize)]
pub struct ProxmoxVECloudMetaData {
    #[serde(rename = "instance-id")]
    pub instance_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ProxmoxVECloudUserData {
    pub hostname: String,
    pub manage_etc_hosts: bool,
    pub fqdn: String,
    pub chpasswd: ProxmoxVECloudChpasswdConfig,
    pub users: Vec<String>,
    pub package_upgrade: bool,
    #[serde(default)]
    pub ssh_authorized_keys: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProxmoxVECloudChpasswdConfig {
    pub expire: bool,
}

#[derive(Debug, Deserialize)]
pub struct ProxmoxVECloudVendorData {}

#[derive(Debug, Deserialize)]
pub struct ProxmoxVECloudNetworkConfig {
    pub version: u32,
    pub config: Vec<ProxmoxVECloudNetworkConfigEntry>,
}

#[derive(Debug, Deserialize)]
pub struct ProxmoxVECloudNetworkConfigEntry {
    #[serde(rename = "type")]
    pub network_type: String,
    pub name: Option<String>,
    pub mac_address: Option<String>,
    #[serde(default)]
    pub address: Vec<String>,
    #[serde(default)]
    pub search: Vec<String>,
    #[serde(default)]
    pub subnets: Vec<ProxmoxVECloudNetworkConfigSubnet>,
}

#[derive(Debug, Deserialize)]
pub struct ProxmoxVECloudNetworkConfigSubnet {
    #[serde(rename = "type")]
    pub subnet_type: String,
    pub address: Option<String>,
    pub netmask: Option<String>,
    pub gateway: Option<String>,
}

impl ProxmoxVECloudConfig {
    pub fn try_new(path: &Path) -> Result<Self> {
        let mut user_data = None;
        let raw_user_data = std::fs::read_to_string(path.join("user-data"))?;

        if let Some(first_line) = raw_user_data.split('\n').next() {
            if first_line.starts_with("#cloud-config") {
                user_data = serde_yaml::from_str(&raw_user_data)?;
            }
        }

        if user_data.is_none() {
            warn!(
                "user-data does not have the expected header `#cloud-config`, ignoring this file"
            );
        }

        Ok(Self {
            user_data,
            meta_data: serde_yaml::from_reader(File::open(path.join("meta-data"))?)?,
            vendor_data: serde_yaml::from_reader(File::open(path.join("vendor-data"))?)?,
            network_config: serde_yaml::from_reader(File::open(path.join("network-config"))?)?,
        })
    }
}

impl MetadataProvider for ProxmoxVECloudConfig {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        let mut out = HashMap::new();

        out.insert(
            "PROXMOXVE_INSTANCE_ID".to_owned(),
            self.meta_data.instance_id.clone(),
        );

        if let Some(hostname) = self.hostname()? {
            out.insert("PROXMOXVE_HOSTNAME".to_owned(), hostname);
        }

        if let Some(first_interface) = self.networks()?.first() {
            first_interface.ip_addresses.iter().for_each(|ip| match ip {
                IpNetwork::V4(network) => {
                    out.insert("PROXMOXVE_IPV4".to_owned(), network.ip().to_string());
                }
                IpNetwork::V6(network) => {
                    out.insert("PROXMOXVE_IPV6".to_owned(), network.ip().to_string());
                }
            });
        }

        Ok(out)
    }

    fn hostname(&self) -> Result<Option<String>> {
        Ok(self
            .user_data
            .as_ref()
            .map(|user_data| user_data.hostname.clone()))
    }

    fn ssh_keys(&self) -> Result<Vec<PublicKey>> {
        if let Some(user_data) = &self.user_data {
            return Ok(user_data
                .ssh_authorized_keys
                .iter()
                .map(|key| PublicKey::from_str(key))
                .collect::<Result<Vec<_>, _>>()?);
        }

        Ok(vec![])
    }

    fn networks(&self) -> Result<Vec<network::Interface>> {
        let nameservers = self
            .network_config
            .config
            .iter()
            .filter(|config| config.network_type == "nameserver")
            .collect::<Vec<_>>();

        if nameservers.len() > 1 {
            return Err(anyhow::anyhow!("too many nameservers, only one supported"));
        }

        let mut interfaces = self
            .network_config
            .config
            .iter()
            .filter(|config| config.network_type == "physical")
            .map(|entry| entry.to_interface())
            .collect::<Result<Vec<_>, _>>()?;

        if let Some(iface) = interfaces.first_mut() {
            if let Some(nameserver) = nameservers.first() {
                iface.nameservers = nameserver
                    .address
                    .iter()
                    .map(|ip| IpAddr::from_str(ip))
                    .collect::<Result<Vec<IpAddr>, AddrParseError>>()?;
            }
        }

        Ok(interfaces)
    }

    fn rd_network_kargs(&self) -> Result<Option<String>> {
        let mut kargs = Vec::new();

        if let Ok(networks) = self.networks() {
            for iface in networks {
                // Add IP configuration if static
                for addr in iface.ip_addresses {
                    match addr {
                        IpNetwork::V4(network) => {
                            if let Some(gateway) = iface
                                .routes
                                .iter()
                                .find(|r| r.destination.is_ipv4() && r.destination.prefix() == 0)
                            {
                                kargs.push(format!(
                                    "ip={}::{}:{}",
                                    network.ip(),
                                    gateway.gateway,
                                    network.mask()
                                ));
                            } else {
                                kargs.push(format!("ip={}:::{}", network.ip(), network.mask()));
                            }
                        }
                        IpNetwork::V6(network) => {
                            if let Some(gateway) = iface
                                .routes
                                .iter()
                                .find(|r| r.destination.is_ipv6() && r.destination.prefix() == 0)
                            {
                                kargs.push(format!(
                                    "ip={}::{}:{}",
                                    network.ip(),
                                    gateway.gateway,
                                    network.prefix()
                                ));
                            } else {
                                kargs.push(format!("ip={}:::{}", network.ip(), network.prefix()));
                            }
                        }
                    }
                }

                // Add DHCP configuration
                if let Some(dhcp) = iface.dhcp {
                    match dhcp {
                        DhcpSetting::V4 => kargs.push("ip=dhcp".to_string()),
                        DhcpSetting::V6 => kargs.push("ip=dhcp6".to_string()),
                        DhcpSetting::Both => kargs.push("ip=dhcp,dhcp6".to_string()),
                    }
                }

                // Add nameservers
                if !iface.nameservers.is_empty() {
                    let nameservers = iface
                        .nameservers
                        .iter()
                        .map(|ns| ns.to_string())
                        .collect::<Vec<_>>()
                        .join(",");
                    kargs.push(format!("nameserver={}", nameservers));
                }
            }
        }

        if kargs.is_empty() {
            Ok(None)
        } else {
            Ok(Some(kargs.join(" ")))
        }
    }

    fn netplan_config(&self) -> Result<Option<String>> {
        // Convert network config to netplan format
        if let Ok(networks) = self.networks() {
            let mut netplan = serde_yaml::Mapping::new();
            let mut network = serde_yaml::Mapping::new();
            let mut ethernets = serde_yaml::Mapping::new();

            for iface in networks {
                let mut eth_config = serde_yaml::Mapping::new();

                // Add DHCP settings
                if let Some(dhcp) = iface.dhcp {
                    match dhcp {
                        DhcpSetting::V4 => {
                            eth_config.insert("dhcp4".into(), true.into());
                        }
                        DhcpSetting::V6 => {
                            eth_config.insert("dhcp6".into(), true.into());
                        }
                        DhcpSetting::Both => {
                            eth_config.insert("dhcp4".into(), true.into());
                            eth_config.insert("dhcp6".into(), true.into());
                        }
                    }
                }

                // Add static addresses if any
                if !iface.ip_addresses.is_empty() {
                    let addresses: Vec<String> = iface
                        .ip_addresses
                        .iter()
                        .map(|addr| addr.to_string())
                        .collect();
                    eth_config.insert("addresses".into(), addresses.into());
                }

                // Add nameservers if any
                if !iface.nameservers.is_empty() {
                    let nameservers: Vec<String> =
                        iface.nameservers.iter().map(|ns| ns.to_string()).collect();
                    eth_config.insert(
                        "nameservers".into(),
                        serde_yaml::Value::Mapping(serde_yaml::Mapping::from_iter(vec![(
                            "addresses".into(),
                            nameservers.into(),
                        )])),
                    );
                }

                if let Some(name) = iface.name {
                    ethernets.insert(name.into(), eth_config.into());
                }
            }

            network.insert("ethernets".into(), ethernets.into());
            netplan.insert("network".into(), network.into());

            Ok(Some(serde_yaml::to_string(&netplan)?))
        } else {
            Ok(None)
        }
    }
}

impl ProxmoxVECloudNetworkConfigEntry {
    pub fn to_interface(&self) -> Result<network::Interface> {
        if self.network_type != "physical" {
            return Err(anyhow::anyhow!(
                "cannot convert config to interface: unsupported config type \"{}\"",
                self.network_type
            ));
        }

        let mut iface = network::Interface {
            name: self.name.clone(),

            // filled later
            nameservers: vec![],
            // filled below
            ip_addresses: vec![],
            // filled below
            routes: vec![],
            // filled below
            dhcp: None,
            // filled below because Option::try_map doesn't exist yet
            mac_address: None,

            // unsupported by proxmox ve
            bond: None,

            // default values
            path: None,
            priority: 20,
            unmanaged: false,
            required_for_online: None,
        };

        for subnet in &self.subnets {
            if subnet.subnet_type.contains("static") {
                if subnet.address.is_none() {
                    return Err(anyhow::anyhow!(
                        "cannot convert static subnet to interface: missing address"
                    ));
                }

                if let Some(netmask) = &subnet.netmask {
                    iface.ip_addresses.push(IpNetwork::with_netmask(
                        IpAddr::from_str(subnet.address.as_ref().unwrap())?,
                        IpAddr::from_str(netmask)?,
                    )?);
                } else {
                    iface
                        .ip_addresses
                        .push(IpNetwork::from_str(subnet.address.as_ref().unwrap())?);
                }

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
                } else {
                    warn!("found subnet type \"static\" without gateway");
                }
            }

            if subnet.subnet_type == "dhcp" || subnet.subnet_type == "dhcp4" {
                iface.dhcp = Some(DhcpSetting::V4)
            }
            if subnet.subnet_type == "dhcp6" {
                iface.dhcp = Some(DhcpSetting::V6)
            }
            if subnet.subnet_type == "ipv6_slaac" {
                warn!("subnet type \"ipv6_slaac\" not supported, ignoring");
            }
        }

        if let Some(mac) = &self.mac_address {
            iface.mac_address = Some(MacAddr::from_str(mac)?);
        }

        Ok(iface)
    }
}
