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

//! Metadata fetcher for Packet.net.
//!
//! Metadata JSON schema is described in their
//! [knowledge base](https://help.packet.net/article/37-metadata).

use std::collections::HashMap;
use std::fs::File;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

use openssh_keys::PublicKey;
use pnet::util::MacAddr;
use serde_json;
use update_ssh_keys::AuthorizedKeyEntry;

use errors::*;
use network::{self, Interface, Device, Section, NetworkRoute};
use providers::MetadataProvider;
use retry;
use util;

use ipnetwork::{self, IpNetwork, Ipv4Network, Ipv6Network};

#[cfg(test)]
mod mock_tests;

#[derive(Clone, Debug, Deserialize)]
struct PacketData {
    id: String,
    hostname: String,
    iqn: String,
    plan: String,
    facility: String,
    tags: Vec<String>,
    ssh_keys: Vec<String>,
    network: PacketNetworkInfo,

    error: Option<String>,
    phone_home_url: String,
}

#[derive(Clone, Debug, Deserialize)]
struct PacketNetworkInfo {
    interfaces: Vec<PacketInterfaceInfo>,
    addresses: Vec<PacketAddressInfo>,
    bonding: PacketBondingMode,
}

#[derive(Clone, Debug, Deserialize)]
struct PacketBondingMode {
    mode: u32,
}

#[derive(Clone, Debug, Deserialize)]
struct PacketInterfaceInfo {
    name: String,
    mac: String,
    bond: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct PacketAddressInfo {
    id: String,
    address_family: i32,
    public: bool,
    management: bool,
    address: IpAddr,
    netmask: IpAddr,
    gateway: IpAddr,
}

/// Custom user-state that can be posted on instance
/// boot to Packet phone-home events endpoint.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct PacketUserState {
    state: String,
    code: u16,
    message: String,
}

#[derive(Clone, Debug)]
pub struct PacketProvider {
    data: PacketData,
}

impl PacketProvider {
    pub fn try_new() -> Result<PacketProvider> {
        let client = retry::Client::try_new()?;

        let data: PacketData = client
            .get(retry::Json, "http://metadata.packet.net/metadata".to_string())
            .send()?
            .ok_or("not found")?;

        Ok(PacketProvider{ data })
    }

    fn get_attrs(&self) -> Result<Vec<(String,String)>> {
        let mut attrs = Vec::new();
        let mut v4_public_counter = 0;
        let mut v4_private_counter = 0;
        let mut v6_public_counter = 0;
        let mut v6_private_counter = 0;
        for a in self.data.network.addresses.clone() {
            match (a.address,a.public) {
                (IpAddr::V4(a),true) => {
                    attrs.push((format!("PACKET_IPV4_PUBLIC_{}", v4_public_counter), format!("{}", a)));
                    v4_public_counter += 1;
                }
                (IpAddr::V4(a),false) => {
                    attrs.push((format!("PACKET_IPV4_PRIVATE_{}", v4_private_counter), format!("{}", a)));
                    v4_private_counter += 1;
                }
                (IpAddr::V6(a),true) => {
                    attrs.push((format!("PACKET_IPV6_PUBLIC_{}", v6_public_counter), format!("{}", a)));
                    v6_public_counter += 1;
                }
                (IpAddr::V6(a),false) => {
                    attrs.push((format!("PACKET_IPV6_PRIVATE_{}", v6_private_counter), format!("{}", a)));
                    v6_private_counter += 1;
                }
            }
        }
        attrs.push(("PACKET_HOSTNAME".to_owned(), self.data.hostname.clone()));
        attrs.push(("PACKET_PHONE_HOME_URL".to_owned(), self.data.phone_home_url.clone()));
        Ok(attrs)
    }

    fn get_dns_servers() -> Result<Vec<IpAddr>> {
        let f = File::open("/run/systemd/netif/state")
            .chain_err(|| "failed to open /run/systemd/netif/state")?;
        let ip_strings = util::key_lookup('=', "DNS", f)
            .chain_err(|| "failed to parse /run/systemd/netif/state")?
            .ok_or("DNS not found in netif state file")?;
        let mut addrs = Vec::new();
        for ip_string in ip_strings.split(' ') {
            addrs.push(IpAddr::from_str(ip_string)
                .chain_err(|| "failed to parse IP address")?);
        }
        if addrs.is_empty() {
            return Err("no DNS servers in /run/systemd/netif/state".into());
        }
        Ok(addrs)
    }

    fn parse_network(&self) -> Result<(Vec<Interface>,Vec<Device>)> {
        let netinfo = &self.data.network;
        let mut interfaces = Vec::new();
        let mut bonds = Vec::new();
        let dns_servers = PacketProvider::get_dns_servers()?;
        for i in netinfo.interfaces.clone() {
            let mac = MacAddr::from_str(&i.mac)
                .map_err(|err| Error::from(format!("{:?}", err)))
                .chain_err(|| format!("failed to parse mac address: '{}'", i.mac))?;
            interfaces.push(Interface {
                mac_address: Some(mac),
                bond: i.bond.clone(),
                name: None,
                priority: None,
                nameservers: Vec::new(),
                ip_addresses: Vec::new(),
                routes: Vec::new(),
                // the interface should be unmanaged if it doesn't have a bond
                // section
                unmanaged: i.bond.is_none(),
            });

            // if there is a bond key, make sure we have a bond device for it
            if let Some(ref bond_name) = i.bond {
                let bond = Interface {
                    name: Some(bond_name.clone()),
                    priority: Some(5),
                    nameservers: dns_servers.clone(),
                    mac_address: None,
                    bond: None,
                    ip_addresses: Vec::new(),
                    routes: Vec::new(),
                    unmanaged: false,
                };
                if !bonds.iter().any(|&(_, ref b): &(MacAddr, Interface)| &bond == b) {
                    bonds.push((mac, bond));
                }
            }
        }

        // According to the folks from packet, all the addresses given to us in the
        // network section should be attached to the first bond we find in the list
        // of interfaces. We should always have at least one bond listed, but if we
        // don't find any, we just print out a scary warning and don't attach the
        // addresses to anything.
        if let Some((_mac, ref mut first_bond)) = bonds.get_mut(0) {
            for a in netinfo.addresses.clone() {
                let prefix = ipnetwork::ip_mask_to_prefix(a.netmask)
                    .chain_err(|| "invalid network mask")?;
                first_bond.ip_addresses.push(IpNetwork::new(a.address, prefix)
                                        .chain_err(|| "invalid IP address or prefix")?);
                let dest = match (a.public, a.address) {
                    (false, IpAddr::V4(_)) =>
                        IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(10,0,0,0),8).unwrap()),
                    (true, IpAddr::V4(_)) =>
                        IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(0,0,0,0),0).unwrap()),
                    (_, IpAddr::V6(_)) =>
                        IpNetwork::V6(Ipv6Network::new(Ipv6Addr::new(0,0,0,0,0,0,0,0),0).unwrap()),
                };
                first_bond.routes.push(NetworkRoute {
                    destination: dest,
                    gateway: a.gateway,
                });
            }
        } else {
            warn!("no bond interfaces. addresses are left unassigned.");
            // the rest of the function operates on bonds, so just return
            return Ok((interfaces, vec![]));
        }

        let mut attrs = vec![
            ("TransmitHashPolicy".to_owned(), "layer3+4".to_owned()),
            ("MIIMonitorSec".to_owned(), ".1".to_owned()),
            ("UpDelaySec".to_owned(), ".2".to_owned()),
            ("DownDelaySec".to_owned(), ".2".to_owned()),
            ("Mode".to_owned(), network::bonding_mode_to_string(netinfo.bonding.mode)?),
        ];
        if netinfo.bonding.mode == network::BONDING_MODE_LACP {
            attrs.push(("LACPTransmitRate".to_owned(), "fast".to_owned()));
        }

        let mut network_devices = Vec::with_capacity(bonds.len());
        for (mac, bond) in bonds {
            network_devices.push(Device {
                name: bond.name.clone()
                    .ok_or("bond doesn't have a name, should be impossible")?,
                kind: "bond".to_owned(),
                mac_address: mac,
                priority: Some(5),
                sections: vec![
                    Section{
                        name: "Bond".to_owned(),
                        attributes: attrs.clone(),
                    }
                ],
            });
            // finally, make sure the bond interfaces are in the interface list
            interfaces.push(bond)
        }

        Ok((interfaces,network_devices))
    }
}

impl MetadataProvider for PacketProvider {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        self.get_attrs().map(|attrs| attrs.into_iter().collect())
    }

    fn hostname(&self) -> Result<Option<String>> {
        Ok(Some(self.data.hostname.clone()))
    }

    fn ssh_keys(&self) -> Result<Vec<AuthorizedKeyEntry>> {
        let mut out = Vec::new();

        for key in &self.data.ssh_keys {
            let key = PublicKey::parse(&key)?;
            out.push(AuthorizedKeyEntry::Valid{key});
        }

        Ok(out)
    }

    fn networks(&self) -> Result<Vec<network::Interface>> {
        let (interfaces, _devices) = self.parse_network()?;

        Ok(interfaces)
    }

    fn network_devices(&self) -> Result<Vec<network::Device>> {
        let (_interfaces, devices) = self.parse_network()?;

        Ok(devices)
    }

    fn boot_checkin(&self) -> Result<()> {
        let user_state = PacketUserState {
            state: "succeeded".into(),
            code: 1042,
            message: "coreos-metadata: boot check-in".into(),
        };
        let client = retry::Client::try_new()?;
        let url = self.data.phone_home_url.clone() + "/events";
        let body = serde_json::to_string(&user_state)?;
        client.post(retry::Json, url, body.into())
            .dispatch_post()?;
        Ok(())
    }
}
