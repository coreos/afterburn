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

use super::provider::NetworkConfigurationFormat;
use crate::{
    network::{DhcpSetting, Interface, VirtualNetDev},
    providers::{kubevirt::configdrive::NetworkData, MetadataProvider},
};
use anyhow::{bail, Context, Result};
use ipnetwork::IpNetwork;
use openssh_keys::PublicKey;
use serde::Deserialize;
use slog_scope::warn;
use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

/// Partial object for `meta_data.json` (ConfigDrive) or `meta-data` (NoCloud)
#[derive(Debug, Deserialize)]
pub struct MetaData {
    /// Local hostname (ConfigDrive format)
    #[serde(default)]
    pub hostname: Option<String>,
    /// Local hostname (NoCloud format)
    #[serde(rename = "local-hostname", default)]
    pub local_hostname: Option<String>,
    /// Instance ID (ConfigDrive format - UUID)
    #[serde(rename = "uuid", default)]
    pub uuid: Option<String>,
    /// Instance ID (NoCloud format)
    #[serde(rename = "instance-id", default)]
    pub instance_id: Option<String>,
    /// Instance type.
    pub instance_type: Option<String>,
    /// SSH public keys.
    pub public_keys: Option<HashMap<String, String>>,
}

#[derive(Debug)]
pub struct KubeVirtCloudConfig {
    pub meta_data: MetaData,
    pub configdrive_network_data: Option<super::configdrive::NetworkData>,
    pub nocloud_network_config: Option<super::nocloud::NetworkConfig>,
}

impl KubeVirtCloudConfig {
    pub fn try_new(path: &Path, format: NetworkConfigurationFormat) -> Result<Self> {
        let meta_data = match format {
            NetworkConfigurationFormat::ConfigDrive => {
                match super::configdrive::read_config_file(path, "meta_data.json")? {
                    Some(reader) => Self::parse_metadata(reader)?,
                    None => bail!("meta_data.json file not found"),
                }
            }
            NetworkConfigurationFormat::NoCloud => {
                match super::nocloud::read_config_file(path, "meta-data")? {
                    Some(reader) => Self::parse_metadata(reader)?,
                    None => bail!("meta-data file not found"),
                }
            }
        };

        let (configdrive_network_data, nocloud_network_config) = match format {
            NetworkConfigurationFormat::ConfigDrive => {
                let config_drive_network_data = super::cloudconfig::NetworkData::from_file(path)?;
                (config_drive_network_data, None)
            }
            NetworkConfigurationFormat::NoCloud => {
                let nocloud_network_config = super::nocloud::NetworkConfig::from_file(path)?;
                (None, nocloud_network_config)
            }
        };

        Ok(Self {
            meta_data,
            configdrive_network_data,
            nocloud_network_config,
        })
    }

    /// Parse metadata attributes.
    ///
    /// Metadata file contains a JSON or YAML object, corresponding to `MetaDataJSON`.
    pub fn parse_metadata(input: BufReader<File>) -> Result<MetaData> {
        serde_yaml::from_reader(input).context("failed to parse metadata")
    }
}

impl MetadataProvider for KubeVirtCloudConfig {
    /// Extract supported cloud config values and convert to Afterburn attributes.
    ///
    /// The `AFTERBURN_` prefix is added later on, so it is not part of the
    /// key-labels here.
    fn attributes(&self) -> Result<HashMap<String, String>> {
        // Get instance ID from either format
        let instance_id = self
            .meta_data
            .instance_id
            .as_ref()
            .or(self.meta_data.uuid.as_ref())
            .ok_or_else(|| anyhow::anyhow!("missing instance ID"))?;

        // Get hostname from either format (prioritize ConfigDrive format for backwards compatibility)
        let hostname_value = self
            .meta_data
            .hostname
            .as_ref()
            .or(self.meta_data.local_hostname.as_ref())
            .ok_or_else(|| anyhow::anyhow!("missing hostname"))?;

        let mut attrs = maplit::hashmap! {
            "KUBEVIRT_INSTANCE_ID".to_string() => instance_id.clone(),
            "KUBEVIRT_HOSTNAME".to_string() => hostname_value.clone(),
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
        // Prefer ConfigDrive format hostname, fall back to NoCloud format
        Ok(self
            .meta_data
            .hostname
            .clone()
            .or_else(|| self.meta_data.local_hostname.clone()))
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
        if let Some(configdrive_network_data) = &self.configdrive_network_data {
            return configdrive_network_data.to_interfaces();
        }

        if let Some(nocloud_config) = &self.nocloud_network_config {
            return nocloud_config.to_interfaces();
        }

        Ok(Vec::<Interface>::new())
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
