// Copyright 2023 CoreOS, Inc.
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

//! Metadata fetcher for the hetzner provider
//! https://docs.hetzner.cloud/#server-metadata

use std::{
    collections::HashMap,
    net::{AddrParseError, IpAddr},
    str::FromStr,
};

use anyhow::Result;
use openssh_keys::PublicKey;
use pnet_base::MacAddr;
use serde::Deserialize;

use ipnetwork::IpNetwork;
use slog_scope::{error, warn};

use crate::{
    network::{self, DhcpSetting, NetworkRoute},
    retry,
};

use super::MetadataProvider;

#[cfg(test)]
mod mock_tests;

// IPv4 address only.
// cloud-init tries http://[fe80::a9fe:a9fe%<interface>]/hetzner/v1/metadata
// first but Hetzner has no IPv6 address. However, even VMs without
// public IPv4 addresses still have link-local access to fetch this
// metadata.
const HETZNER_METADATA_BASE_URL: &str = "http://169.254.169.254/hetzner/v1/metadata";

/// Metadata provider for Hetzner Cloud
///
/// See: https://docs.hetzner.cloud/#server-metadata
#[derive(Debug)]
pub struct HetznerProvider {
    client: retry::Client,
}

impl HetznerProvider {
    pub fn try_new() -> Result<Self> {
        let client = retry::Client::try_new()?;
        Ok(Self { client })
    }

    fn endpoint_for(key: &str) -> String {
        format!("{HETZNER_METADATA_BASE_URL}/{key}")
    }
}

impl MetadataProvider for HetznerProvider {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        let metadata: Metadata = self
            .client
            .get(retry::Yaml, HETZNER_METADATA_BASE_URL.to_string())
            .send()?
            .unwrap();

        let private_networks: Vec<PrivateNetwork> = self
            .client
            .get(retry::Yaml, Self::endpoint_for("private-networks"))
            .send()?
            .unwrap();

        Ok(Attributes {
            metadata,
            private_networks,
        }
        .into())
    }

    fn hostname(&self) -> Result<Option<String>> {
        let hostname: String = self
            .client
            .get(retry::Raw, Self::endpoint_for("hostname"))
            .send()?
            .unwrap_or_default();

        if hostname.is_empty() {
            return Ok(None);
        }

        Ok(Some(hostname))
    }

    fn ssh_keys(&self) -> Result<Vec<PublicKey>> {
        let keys: Vec<String> = self
            .client
            .get(retry::Json, Self::endpoint_for("public-keys"))
            .send()?
            .unwrap_or_default();

        let keys = keys
            .iter()
            .map(|s| PublicKey::parse(s))
            .collect::<Result<_, _>>()?;

        Ok(keys)
    }

    fn networks(&self) -> Result<Vec<crate::network::Interface>> {
        let network_config: NetworkConfig = self
            .client
            .get(retry::Yaml, Self::endpoint_for("network-config"))
            .send()?
            .unwrap_or_default();

        network_config
            .config
            .iter()
            .filter(|config| config.network_type == "physical")
            .map(NetworkConfigEntry::to_interface)
            .collect()
    }

    fn netplan_config(&self) -> Result<Option<String>> {
        let networks = self.networks()?;

        let mut ethernets = serde_yaml::Mapping::new();

        for iface in networks {
            let mut eth_config = serde_yaml::Mapping::new();
            let Some(name) = iface.name else {
                warn!("Skipping interface, no name specified: {iface:?}");
                continue;
            };

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

            if !iface.ip_addresses.is_empty() {
                let addresses: Vec<String> = iface
                    .ip_addresses
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect();
                eth_config.insert("addresses".into(), addresses.into());
            }

            if !iface.routes.is_empty() {
                let routes: Vec<serde_yaml::Value> = iface
                    .routes
                    .iter()
                    .map(|route| {
                        let mut route_map = serde_yaml::Mapping::new();
                        route_map.insert("to".into(), route.destination.to_string().into());
                        route_map.insert("via".into(), route.gateway.to_string().into());
                        serde_yaml::Value::Mapping(route_map)
                    })
                    .collect();
                eth_config.insert("routes".into(), routes.into());
            }

            if !iface.nameservers.is_empty() {
                let nameservers: Vec<_> = iface
                    .nameservers
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect();
                eth_config.insert(
                    "nameservers".into(),
                    serde_yaml::Value::Mapping(serde_yaml::Mapping::from_iter([(
                        "addresses".into(),
                        nameservers.into(),
                    )])),
                );
            }

            ethernets.insert(name.into(), eth_config.into());
        }

        let network = serde_yaml::Mapping::from_iter([
            ("version".into(), 2.into()),
            ("ethernets".into(), ethernets.into()),
        ]);
        let netplan = serde_yaml::Mapping::from_iter([("network".into(), network.into())]);

        Ok(Some(serde_yaml::to_string(&netplan)?))
    }

    fn rd_network_kargs(&self) -> Result<Option<String>> {
        warn!(
            "initrd network kargs requested, but not supported on this platform due to network \
            requirements for fetching metadata"
        );
        Ok(None)
    }
}

#[derive(Debug, Deserialize)]
struct PrivateNetwork {
    ip: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Metadata {
    hostname: Option<String>,
    instance_id: Option<i64>,
    public_ipv4: Option<String>,
    availability_zone: Option<String>,
    region: Option<String>,
    network_config: Option<NetworkConfig>,
}

// NOTE: Hetzner's network config seems to mostly follow the cloud-init v1 network config, but with
// some minor deviations. See https://docs.cloud-init.io/en/latest/reference/network-config-format-v1.html
#[derive(Debug, Deserialize)]
struct NetworkConfig {
    #[serde(default)]
    config: Vec<NetworkConfigEntry>,
}

#[derive(Debug, Deserialize)]
struct NetworkConfigEntry {
    #[serde(rename = "type")]
    network_type: String,
    mac_address: Option<String>,
    name: Option<String>,
    #[serde(default)]
    subnets: Vec<SubnetConfig>,
}

impl NetworkConfigEntry {
    fn to_interface(&self) -> Result<network::Interface> {
        if self.network_type != "physical" {
            return Err(anyhow::anyhow!(
                "cannot convert config to interface: unsupported network type \"{}\"",
                self.network_type
            ));
        }

        let mut iface = network::Interface {
            name: self.name.clone(),
            ip_addresses: vec![],
            routes: vec![],
            dhcp: None,
            mac_address: None,
            nameservers: vec![],
            bond: None,
            path: None,
            priority: 20,
            unmanaged: false,
            required_for_online: None,
        };

        if let Some(mac) = &self.mac_address {
            iface.mac_address = Some(MacAddr::from_str(mac)?);
        }

        for subnet in &self.subnets {
            match subnet.subnet_type.as_ref() {
                "static" | "static6" => {
                    let Some(ref address_str) = subnet.address else {
                        return Err(anyhow::anyhow!(
                            "cannot convert static subnet to interface: missing address"
                        ));
                    };

                    iface.nameservers.extend(
                        subnet
                            .dns_nameservers
                            .iter()
                            .map(|ip| IpAddr::from_str(ip))
                            .collect::<Result<Vec<IpAddr>, AddrParseError>>()?,
                    );

                    if let Some(netmask) = &subnet.netmask {
                        iface.ip_addresses.push(IpNetwork::with_netmask(
                            IpAddr::from_str(address_str)?,
                            IpAddr::from_str(netmask)?,
                        )?);
                    } else {
                        iface.ip_addresses.push(IpNetwork::from_str(address_str)?);
                    }

                    let Some(gateway_str) = &subnet.gateway else {
                        warn!("found subnet type \"static\" without gateway - address added but will not be routable to the internet");
                        continue;
                    };

                    let gateway = IpAddr::from_str(gateway_str)?;
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

                "dhcp" | "dhcp4" | "dhcp6" => {
                    let dhcp = if subnet.ipv6.is_some_and(|b| b) {
                        DhcpSetting::V6
                    } else {
                        DhcpSetting::V4
                    };
                    iface.dhcp = iface.dhcp.map(|d| d.merge(dhcp.clone())).or(Some(dhcp))
                }

                subnet_type => warn!("Ignoring unsupported subnet type: \"{subnet_type}\""),
            }
        }

        Ok(iface)
    }
}

#[derive(Debug, Deserialize)]
struct SubnetConfig {
    #[serde(rename = "type")]
    subnet_type: String,
    #[allow(unused)]
    ipv4: Option<bool>,
    ipv6: Option<bool>,
    netmask: Option<String>,
    address: Option<String>,
    gateway: Option<String>,
    #[serde(default)]
    dns_nameservers: Vec<String>,
}

struct Attributes {
    metadata: Metadata,
    private_networks: Vec<PrivateNetwork>,
}

impl From<Attributes> for HashMap<String, String> {
    fn from(attributes: Attributes) -> Self {
        let mut out = HashMap::with_capacity(5);

        let add_value = |map: &mut HashMap<_, _>, key: &str, value: Option<String>| {
            if let Some(value) = value {
                map.insert(key.to_owned(), value);
            }
        };

        add_value(
            &mut out,
            "HETZNER_AVAILABILITY_ZONE",
            attributes.metadata.availability_zone,
        );
        add_value(&mut out, "HETZNER_HOSTNAME", attributes.metadata.hostname);
        add_value(
            &mut out,
            "HETZNER_INSTANCE_ID",
            attributes.metadata.instance_id.map(|i| i.to_string()),
        );
        add_value(
            &mut out,
            "HETZNER_PUBLIC_IPV4",
            attributes.metadata.public_ipv4,
        );
        add_value(&mut out, "HETZNER_REGION", attributes.metadata.region);

        for (i, a) in attributes.private_networks.iter().enumerate() {
            add_value(
                &mut out,
                format!("HETZNER_PRIVATE_IPV4_{i}").as_str(),
                a.ip.clone(),
            );
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{self};

    use super::*;

    #[test]
    fn test_metadata_deserialize() {
        let body = fs::read("./tests/fixtures/hetzner/metadata.yaml")
            .expect("Unable to read metadata fixture");
        let meta: Metadata = serde_yaml::from_slice(body.as_slice()).unwrap();

        assert_eq!(meta.availability_zone.unwrap(), "hel1-dc2");
        assert_eq!(meta.hostname.unwrap(), "my-server");
        assert_eq!(meta.instance_id.unwrap(), 42);
        assert_eq!(meta.public_ipv4.unwrap(), "1.2.3.4");
        assert_eq!(meta.network_config.unwrap().config.len(), 1);
    }

    #[test]
    fn test_private_networks_deserialize() {
        let body = fs::read("./tests/fixtures/hetzner/private-networks.yaml")
            .expect("Unable to read metadata fixture");
        let private_networks: Vec<PrivateNetwork> =
            serde_yaml::from_slice(body.as_slice()).unwrap();

        assert_eq!(private_networks.len(), 2);
        assert_eq!(private_networks[0].ip.clone().unwrap(), "10.0.0.2");
        assert_eq!(private_networks[1].ip.clone().unwrap(), "10.128.0.2");
    }

    #[test]
    fn test_network_config_deserialize() {
        let body = fs::read("./tests/fixtures/hetzner/network-config.yaml")
            .expect("Unable to read network-config fixture");
        let network_config: NetworkConfig = serde_yaml::from_slice(body.as_slice()).unwrap();

        assert_eq!(network_config.config.len(), 1);

        let entry = network_config.config.first().unwrap();
        assert_eq!(entry.name.as_ref().unwrap(), "eth0");
        assert_eq!(entry.mac_address.as_ref().unwrap(), "00:00:00:00:00:00");
        assert_eq!(entry.network_type, "physical");
        assert_eq!(entry.subnets.len(), 2);
    }
}
