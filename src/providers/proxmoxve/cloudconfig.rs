use crate::{
    network::{self, NetworkRoute},
    providers::MetadataProvider,
};
use anyhow::Result;
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
    pub user_data: ProxmoxVECloudUserData,
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
        Ok(Self {
            meta_data: serde_yaml::from_reader(File::open(path.join("meta-data"))?)?,
            user_data: serde_yaml::from_reader(File::open(path.join("user-data"))?)?,
            vendor_data: serde_yaml::from_reader(File::open(path.join("vendor-data"))?)?,
            network_config: serde_yaml::from_reader(File::open(path.join("network-config"))?)?,
        })
    }
}

impl MetadataProvider for ProxmoxVECloudConfig {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        let mut out = HashMap::new();

        out.insert(
            "PROXMOXVE_HOSTNAME".to_owned(),
            self.hostname()?.unwrap_or_default(),
        );

        out.insert(
            "PROXMOXVE_INSTANCE_ID".to_owned(),
            self.meta_data.instance_id.clone(),
        );

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
        Ok(Some(self.user_data.hostname.clone()))
    }

    fn ssh_keys(&self) -> Result<Vec<PublicKey>> {
        Ok(self
            .user_data
            .ssh_authorized_keys
            .iter()
            .map(|key| PublicKey::from_str(key))
            .collect::<Result<Vec<_>, _>>()?)
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
