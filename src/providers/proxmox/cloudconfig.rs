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
pub struct ProxmoxCloudConfig {
    pub meta_data: ProxmoxCloudMetaData,
    pub user_data: ProxmoxCloudUserData,
    pub vendor_data: ProxmoxCloudVendorData,
    pub network_config: ProxmoxCloudNetworkConfig,
}

#[derive(Debug, Deserialize)]
pub struct ProxmoxCloudMetaData {
    #[serde(rename = "instance-id")]
    pub instance_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ProxmoxCloudUserData {
    pub hostname: String,
    pub manage_etc_hosts: bool,
    pub fqdn: String,
    pub chpasswd: ProxmoxCloudChpasswdConfig,
    pub users: Vec<String>,
    pub package_upgrade: bool,
    #[serde(default)]
    pub ssh_authorized_keys: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProxmoxCloudChpasswdConfig {
    pub expire: bool,
}

#[derive(Debug, Deserialize)]
pub struct ProxmoxCloudVendorData {}

#[derive(Debug, Deserialize)]
pub struct ProxmoxCloudNetworkConfig {
    pub version: u32,
    pub config: Vec<ProxmoxCloudNetworkConfigEntry>,
}

#[derive(Debug, Deserialize)]
pub struct ProxmoxCloudNetworkConfigEntry {
    #[serde(rename = "type")]
    pub network_type: String,
    pub name: Option<String>,
    pub mac_address: Option<String>,
    #[serde(default)]
    pub address: Vec<String>,
    #[serde(default)]
    pub search: Vec<String>,
    #[serde(default)]
    pub subnets: Vec<ProxmoxCloudNetworkConfigSubnet>,
}

#[derive(Debug, Deserialize)]
pub struct ProxmoxCloudNetworkConfigSubnet {
    #[serde(rename = "type")]
    pub subnet_type: String,
    pub address: Option<String>,
    pub netmask: Option<String>,
    pub gateway: Option<String>,
}

impl ProxmoxCloudConfig {
    pub fn try_new(path: &Path) -> Result<Self> {
        Ok(Self {
            meta_data: serde_yaml::from_reader(File::open(path.join("meta-data"))?)?,
            user_data: serde_yaml::from_reader(File::open(path.join("user-data"))?)?,
            vendor_data: serde_yaml::from_reader(File::open(path.join("vendor-data"))?)?,
            network_config: serde_yaml::from_reader(File::open(path.join("network-config"))?)?,
        })
    }
}

impl MetadataProvider for ProxmoxCloudConfig {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        let mut out = HashMap::new();

        out.insert(
            "AFTERBURN_PROXMOX_HOSTNAME".to_owned(),
            self.hostname()?.unwrap_or_default(),
        );

        out.insert(
            "AFTERBURN_PROXMOX_INSTANCE_ID".to_owned(),
            self.meta_data.instance_id.clone(),
        );

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
            .filter(|config: &&ProxmoxCloudNetworkConfigEntry| config.network_type == "nameserver")
            .collect::<Vec<_>>();

        if nameservers.len() > 1 {
            return Err(anyhow::anyhow!("too many nameservers, only one supported"));
        }

        let mut interfaces = self
            .network_config
            .config
            .iter()
            .filter(|config: &&ProxmoxCloudNetworkConfigEntry| config.network_type == "physical")
            .map(|entry| entry.to_interface())
            .collect::<Result<Vec<_>, _>>()?;

        if let Some(nameserver) = nameservers.first() {
            interfaces[0].nameservers = nameserver
                .address
                .iter()
                .map(|ip| IpAddr::from_str(ip))
                .collect::<Result<Vec<IpAddr>, AddrParseError>>()?;
        }

        Ok(interfaces)
    }
}

impl ProxmoxCloudNetworkConfigEntry {
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

            // unsupported by proxmox
            bond: None,

            // default values
            path: None,
            priority: 20,
            unmanaged: false,
            required_for_online: None,
        };

        for subnet in &self.subnets {
            if subnet.subnet_type == "static" {
                if subnet.address.is_none() || subnet.netmask.is_none() {
                    return Err(anyhow::anyhow!(
                        "cannot convert static subnet to interface: missing address and/or netmask"
                    ));
                }

                iface.ip_addresses.push(IpNetwork::with_netmask(
                    IpAddr::from_str(subnet.address.as_ref().unwrap())?,
                    IpAddr::from_str(subnet.netmask.as_ref().unwrap())?,
                )?);

                if let Some(gateway) = &subnet.gateway {
                    iface.routes.push(NetworkRoute {
                        destination: IpNetwork::from_str("0.0.0.0/0")?,
                        gateway: IpAddr::from_str(gateway)?,
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
            iface.mac_address = Some(MacAddr::from_str(&mac)?);
        }

        Ok(iface)
    }
}
