//! KubeVirt cloud config parsing.
//!
//! This provider supports platforms based on KubeVirt.
//! It provides a config-drive as the only metadata source, whose layout
//! follows the `cloud-init ConfigDrive v2` [datasource][configdrive], with
//! the following details:
//!  - disk filesystem label is `config-2` (lowercase)
//!  - filesystem is `iso9660`
//!  - drive contains a single directory at `/openstack/latest/`
//!  - content is exposed as JSON or YAML files called `meta_data.json`.
//!
//! configdrive: https://cloudinit.readthedocs.io/en/latest/topics/datasources/configdrive.html

use crate::{
    network::{DhcpSetting, Interface, VirtualNetDev},
    providers::{
        kubevirt::networkdata::{network_interfaces, NetworkData},
        MetadataProvider,
    },
};
use anyhow::{bail, Context, Result};
use ipnetwork::IpNetwork;
use openssh_keys::PublicKey;
use serde::Deserialize;
use slog_scope::warn;
use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

/// Partial object for `meta_data.json`
#[derive(Debug, Deserialize)]
pub struct MetaData {
    /// Local hostname
    pub hostname: String,
    /// Instance ID (UUID).
    #[serde(rename = "uuid")]
    pub instance_id: String,
    /// Instance type.
    pub instance_type: Option<String>,
    /// SSH public keys.
    pub public_keys: Option<HashMap<String, String>>,
}

#[derive(Debug)]
pub struct KubeVirtCloudConfig {
    pub meta_data: MetaData,
    pub network_data: Option<NetworkData>,
}

impl KubeVirtCloudConfig {
    pub fn try_new(path: &Path) -> Result<Self> {
        let meta_data = match Self::read_cloud_config_file(path, "meta_data.json")? {
            Some(reader) => Self::parse_metadata(reader)?,
            None => bail!("meta_data.json file not found"),
        };

        let network_data = Self::read_cloud_config_file(path, "network_data.json")?
            .map(Self::parse_network_data)
            .transpose()?;

        Ok(Self {
            meta_data,
            network_data,
        })
    }
    pub fn read_cloud_config_file(path: &Path, file: &str) -> Result<Option<BufReader<File>>> {
        let cloudconfig_dir = path.join("openstack").join("latest");
        let filename = cloudconfig_dir.join(file);
        if !filename.exists() {
            return Ok(None);
        }
        let file =
            File::open(&filename).with_context(|| format!("failed to open file '{filename:?}'"))?;
        Ok(Some(BufReader::new(file)))
    }

    /// Parse metadata attributes.
    ///
    /// Metadata file contains a JSON or YAML object, corresponding to `MetaDataJSON`.
    pub fn parse_metadata(input: BufReader<File>) -> Result<MetaData> {
        serde_yaml::from_reader(input).context("failed to parse metadata")
    }

    /// Parse network configuration.
    ///
    /// Network configuration file contains a JSON object in OpenStack network metadata format.
    /// This format uses links, networks, and services sections to describe network configuration.
    fn parse_network_data(input: BufReader<File>) -> Result<NetworkData> {
        serde_json::from_reader(input).context("failed to parse JSON network data")
    }
}

impl MetadataProvider for KubeVirtCloudConfig {
    /// Extract supported cloud config values and convert to Afterburn attributes.
    ///
    /// The `AFTERBURN_` prefix is added later on, so it is not part of the
    /// key-labels here.
    fn attributes(&self) -> Result<HashMap<String, String>> {
        if self.meta_data.instance_id.is_empty() {
            bail!("empty instance ID");
        }

        if self.meta_data.hostname.is_empty() {
            bail!("empty local hostname");
        }

        let mut attrs = maplit::hashmap! {
            "KUBEVIRT_INSTANCE_ID".to_string() => self.meta_data.instance_id.clone(),
            "KUBEVIRT_HOSTNAME".to_string() => self.meta_data.hostname.clone(),
        };

        if let Some(instance_type) = &self.meta_data.instance_type {
            attrs.insert("KUBEVIRT_INSTANCE_TYPE".to_string(), instance_type.clone());
        }

        if let Some(interface_with_ips) = self
            .networks()?
            .iter()
            .find(|iface| !iface.ip_addresses.is_empty())
        {
            interface_with_ips
                .ip_addresses
                .iter()
                .for_each(|ip| match ip {
                    IpNetwork::V4(network) => {
                        attrs
                            .entry("KUBEVIRT_IPV4".to_owned())
                            .or_insert_with(|| network.ip().to_string());
                    }
                    IpNetwork::V6(network) => {
                        attrs
                            .entry("KUBEVIRT_IPV6".to_owned())
                            .or_insert_with(|| network.ip().to_string());
                    }
                });
        }

        Ok(attrs)
    }

    fn hostname(&self) -> Result<Option<String>> {
        let hostname = if self.meta_data.hostname.is_empty() {
            None
        } else {
            Some(self.meta_data.hostname.clone())
        };
        Ok(hostname)
    }

    /// The public key is stored as key:value pair in openstack/latest/meta_data.json file
    fn ssh_keys(&self) -> Result<Vec<PublicKey>> {
        self.meta_data
            .public_keys
            .iter()
            .flat_map(|keys| keys.values())
            .map(|key| PublicKey::parse(key).map_err(anyhow::Error::from))
            .collect()
    }

    fn networks(&self) -> Result<Vec<Interface>> {
        match &self.network_data {
            Some(network_data) => network_interfaces(network_data),
            None => Ok(Vec::<Interface>::new()),
        }
    }

    fn rd_network_kargs(&self) -> Result<Option<String>> {
        let mut kargs = Vec::new();
        let mut all_nameservers = Vec::new();

        let networks = self.networks()?;
        for iface in networks {
            // Use interface name as identifier if there is one
            // else use mac address or continue
            let id = if let Some(iface_name) = iface.name {
                iface_name
            } else if let Some(iface_mac) = iface.mac_address {
                format!("{}", iface_mac)
            } else {
                continue;
            };

            // Add IP configuration if static
            for addr in iface.ip_addresses {
                let (ip, netmask_or_prefix) = match addr {
                    IpNetwork::V4(n) => (n.ip().to_string(), n.mask().to_string()),
                    IpNetwork::V6(n) => (n.ip().to_string(), n.prefix().to_string()),
                };

                let gateway = iface.routes.iter().find(|r| {
                    r.destination.prefix() == 0 && r.destination.is_ipv4() == addr.is_ipv4()
                });

                if let Some(gateway) = gateway {
                    kargs.push(format!(
                        "ip={}::{}:{}::{}:static",
                        ip, gateway.gateway, netmask_or_prefix, id,
                    ));
                } else {
                    kargs.push(format!("ip={}:::{}::{}:static", ip, netmask_or_prefix, id));
                }
            }

            // Add DHCP configuration
            if let Some(dhcp) = iface.dhcp {
                match dhcp {
                    DhcpSetting::V4 => kargs.push(format!("ip={}:dhcp", id)),
                    DhcpSetting::V6 => kargs.push(format!("ip={}:dhcp6", id)),
                    DhcpSetting::Both => kargs.push(format!("ip={}:dhcp,dhcp6", id)),
                }
            }

            // Collect nameservers from all interfaces
            for nameserver in &iface.nameservers {
                if !all_nameservers.contains(nameserver) {
                    all_nameservers.push(*nameserver);
                }
            }
        }

        // Add nameservers as separate arguments
        for nameserver in &all_nameservers {
            kargs.push(format!("nameserver={}", nameserver));
        }

        if kargs.is_empty() {
            Ok(None)
        } else {
            Ok(Some(kargs.join(" ")))
        }
    }

    fn virtual_network_devices(&self) -> Result<Vec<VirtualNetDev>> {
        warn!("virtual network devices metadata requested, but not supported on this platform");
        Ok(vec![])
    }

    fn boot_checkin(&self) -> Result<()> {
        warn!("boot check-in requested, but not supported on this platform");
        Ok(())
    }
}
