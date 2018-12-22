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

//! digital ocean metadata fetcher

use std::str::FromStr;
use std::net::{IpAddr,Ipv4Addr,Ipv6Addr};
use std::collections::HashMap;

use ipnetwork;
use ipnetwork::{IpNetwork,Ipv4Network,Ipv6Network};
use openssh_keys::PublicKey;
use pnet::util::MacAddr;
use update_ssh_keys::AuthorizedKeyEntry;

use errors::*;
use network;
use providers::MetadataProvider;
use retry;

#[derive(Clone,Deserialize)]
struct Address {
    ip_address: IpAddr,
    netmask: Option<IpAddr>,
    cidr: Option<u8>,
    gateway: IpAddr,
}

#[derive(Clone,Deserialize)]
struct Interface {
    ipv4: Option<Address>,
    ipv6: Option<Address>,
    anchor_ipv4: Option<Address>,
    mac: String,
    #[serde(rename = "type")]
    type_name: String,
}

#[derive(Clone,Deserialize)]
struct Interfaces {
    public: Option<Vec<Interface>>,
    private: Option<Vec<Interface>>,
}

#[derive(Clone,Deserialize)]
struct DNS {
    nameservers: Vec<IpAddr>
}

#[derive(Clone,Deserialize)]
pub struct DigitalOceanProvider {
    hostname: String,
    interfaces: Interfaces,
    public_keys: Vec<String>,
    region: String,
    dns: DNS,
}

impl DigitalOceanProvider {
    pub fn try_new() -> Result<DigitalOceanProvider> {
        let client = retry::Client::try_new()?;
        let data: DigitalOceanProvider = client.get(retry::Json, "http://169.254.169.254/metadata/v1.json".to_owned()).send()?
            .ok_or("not found")?;

        Ok(data)
    }

    fn parse_attrs(&self) -> Result<Vec<(String,String)>> {
        let mut attrs = Vec::new();

        attrs.push(("DIGITALOCEAN_HOSTNAME".to_owned(), self.hostname.clone()));
        attrs.push(("DIGITALOCEAN_REGION".to_owned(), self.region.clone()));

        if let Some(ref ifaces) = self.interfaces.public {
            for (i, ref a) in ifaces.iter().enumerate() {
                if let Some(ref v4) = a.ipv4 {
                    attrs.push((
                        format!("DIGITALOCEAN_IPV4_PUBLIC_{}", i),
                        format!("{}", v4.ip_address)
                    ));
                }
                if let Some(ref v6) = a.ipv6 {
                    attrs.push((
                        format!("DIGITALOCEAN_IPV6_PUBLIC_{}", i),
                        format!("{}", v6.ip_address)
                    ));
                }
                if let Some(ref anchor_v4) = a.anchor_ipv4 {
                    attrs.push((
                        format!("DIGITALOCEAN_IPV4_ANCHOR_{}", i),
                        format!("{}", anchor_v4.ip_address)
                    ));
                }
            }
        }

        if let Some(ref ifaces) = self.interfaces.private {
            for (i, ref a) in ifaces.iter().enumerate() {
                if let Some(ref v4) = a.ipv4 {
                    attrs.push((
                        format!("DIGITALOCEAN_IPV4_PRIVATE_{}", i),
                        format!("{}", v4.ip_address)
                    ));
                }
                if let Some(ref v6) = a.ipv6 {
                    attrs.push((
                        format!("DIGITALOCEAN_IPV6_PRIVATE_{}", i),
                        format!("{}", v6.ip_address)
                    ));
                }
            }
        }

        Ok(attrs)
    }

    fn parse_network(&self) -> Result<Vec<network::Interface>> {
        let mut interfaces = Vec::new();
        if let Some(ifaces) = self.interfaces.public.clone() {
            interfaces.extend(self.parse_interfaces(ifaces)?);
        }
        if let Some(ifaces) = self.interfaces.private.clone() {
            interfaces.extend(self.parse_interfaces(ifaces)?);
        }
        Ok(interfaces)
    }

    fn parse_interfaces(&self, interfaces: Vec<Interface>) -> Result<Vec<network::Interface>> {
        let mut iface_config_map: HashMap<MacAddr,network::Interface> = HashMap::new();
        for iface in interfaces {
            let mac = MacAddr::from_str(&iface.mac)
                .map_err(|e| Error::from(format!("{:?}", e)))
                .chain_err(|| "failed to parse mac address")?;
            let (mut addrs, mut routes) = DigitalOceanProvider::parse_interface(&iface)?;

            if let Some(existing_iface) = iface_config_map.get_mut(&mac) {
                addrs.extend(existing_iface.ip_addresses.clone());
                routes.extend(existing_iface.routes.clone());
            }
            iface_config_map.insert(mac, network::Interface{
                mac_address: Some(mac),
                nameservers: self.dns.nameservers.clone(),
                ip_addresses: addrs,
                routes,
                bond: None,
                name: None,
                priority: None,
                unmanaged: false,
            });
        }
        let mut iface_configs = Vec::new();
        for i in iface_config_map.values() {
            iface_configs.push(i.clone());
        }
        Ok(iface_configs)
    }

    fn parse_interface(interface: &Interface) -> Result<(Vec<IpNetwork>,Vec<network::NetworkRoute>)> {
        let mut addrs = Vec::new();
        let mut routes = Vec::new();

        if interface.ipv4.is_some() {
            let netmask = interface.clone().ipv4.unwrap().netmask
                .ok_or("missing netmask for ipv4 address")?;
            let prefix = ipnetwork::ip_mask_to_prefix(netmask)
                .chain_err(|| "invalid network mask")?;
            let a = match interface.clone().ipv4.unwrap().ip_address {
                IpAddr::V4(a) => Some(a),
                IpAddr::V6(_) => None,
            }.ok_or("ipv6 address in ipv4 field")?;
            let net = IpNetwork::V4(Ipv4Network::new(a, prefix)
                .chain_err(|| "invalid ip address or prefix")?);
            addrs.push(net);
            routes.push(network::NetworkRoute{
                destination: net,
                gateway: interface.clone().ipv4.unwrap().gateway,
            });

            if interface.type_name == "public" {
                routes.push(network::NetworkRoute{
                    destination: IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(0,0,0,0),0)
                                        .chain_err(|| "invalid ip address or prefix")?),
                    gateway: interface.clone().ipv4.unwrap().gateway,
                });
            }
        }
        if interface.ipv6.is_some() {
            let cidr = interface.clone().ipv6.unwrap().cidr
                .ok_or("missing cidr for ipv6 address")?;
            let a = match interface.clone().ipv6.unwrap().ip_address {
                IpAddr::V4(_) => None,
                IpAddr::V6(a) => Some(a),
            }.ok_or("ipv4 address in ipv6 field")?;
            let net = IpNetwork::V6(Ipv6Network::new(a, cidr)
                .chain_err(|| "invalid ip address or prefix")?);
            addrs.push(net);
            routes.push(network::NetworkRoute{
                destination: net,
                gateway: interface.clone().ipv6.unwrap().gateway,
            });
            if interface.type_name == "public" {
                routes.push(network::NetworkRoute{
                    destination: IpNetwork::V6(Ipv6Network::new(Ipv6Addr::new(0,0,0,0,0,0,0,0),0)
                                        .chain_err(|| "invalid ip address or prefix")?),
                    gateway: interface.clone().ipv6.unwrap().gateway,
                });
            }
        }
        if interface.anchor_ipv4.is_some() {
            let netmask = interface.clone().anchor_ipv4.unwrap().netmask
                .ok_or("missing netmask for anchor ipv4 address")?;
            let prefix = ipnetwork::ip_mask_to_prefix(netmask)
                .chain_err(|| "invalid network mask")?;
            let a = match interface.clone().anchor_ipv4.unwrap().ip_address {
                IpAddr::V4(a) => Some(a),
                IpAddr::V6(_) => None,
            }.ok_or("ipv6 address in ipv4 field")?;
            let net = IpNetwork::V4(Ipv4Network::new(a, prefix)
                .chain_err(|| "invalid ip address or prefix")?);
            addrs.push(net);
            routes.push(network::NetworkRoute{
                destination: net,
                gateway: interface.clone().anchor_ipv4.unwrap().gateway,
            });
        }
        Ok((addrs,routes))
    }
}

impl MetadataProvider for DigitalOceanProvider {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        self.parse_attrs().map(|attrs| attrs.into_iter().collect())
    }

    fn hostname(&self) -> Result<Option<String>> {
        Ok(Some(self.hostname.clone()))
    }

    fn ssh_keys(&self) -> Result<Vec<AuthorizedKeyEntry>> {
        let mut out = Vec::new();

        for key in &self.public_keys {
            let key = PublicKey::parse(&key)?;
            out.push(AuthorizedKeyEntry::Valid{key});
        }

        Ok(out)
    }

    fn networks(&self) -> Result<Vec<network::Interface>> {
        self.parse_network()
    }

    fn network_devices(&self) -> Result<Vec<network::Device>> {
        Ok(vec![])
    }
}
