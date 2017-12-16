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

//! packet metadata fetcher

use metadata::Metadata;

use errors::*;

use retry;
use std::fs::File;
use std::str::FromStr;

use util;

use network;
use network::{Interface,Device,Section,NetworkRoute};
use pnet::util::MacAddr;
use std::net::{IpAddr,Ipv4Addr,Ipv6Addr};
use ipnetwork;
use ipnetwork::{IpNetwork,Ipv4Network,Ipv6Network};

#[derive(Clone,Deserialize)]
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

#[derive(Clone,Deserialize)]
struct PacketNetworkInfo {
    interfaces: Vec<PacketInterfaceInfo>,
    addresses: Vec<PacketAddressInfo>,
    bonding: PacketBondingMode,
}

#[derive(Clone,Deserialize)]
struct PacketBondingMode {
    mode: u32,
}

#[derive(Clone,Deserialize)]
struct PacketInterfaceInfo {
    name: String,
    mac: String,
    bond: Option<String>,
}

#[derive(Clone,Deserialize)]
struct PacketAddressInfo {
    id: String,
    address_family: i32,
    public: bool,
    management: bool,
    address: IpAddr,
    netmask: IpAddr,
    gateway: IpAddr,
}

pub fn fetch_metadata() -> Result<Metadata> {
    let client = retry::Client::new()?;
    let data: PacketData = client.get(retry::Json, "http://metadata.packet.net/metadata".to_owned()).send()?
        .ok_or("not found")?;

    let (interfaces,network_devices) = parse_network(&data.network)?;

    let attrs = get_attrs(&data)?;

    let mut m = Metadata::builder()
        .add_ssh_keys(data.ssh_keys)?
        .set_hostname(data.hostname);

    for (key,val) in attrs {
        m = m.add_attribute(key,val);
    }
    for iface in interfaces {
        m = m.add_network_interface(iface);
    }
    for netdev in network_devices {
        m = m.add_network_device(netdev);
    }
    Ok(m.build())
}

fn get_dns_servers() -> Result<Vec<IpAddr>> {
    let f = File::open("/run/systemd/netif/state")
        .chain_err(|| "failed to open /run/systemd/netif/state")?;
    let ip_strings = util::key_lookup_reader('=', "DNS", f)
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

fn parse_network(netinfo: &PacketNetworkInfo) -> Result<(Vec<Interface>,Vec<Device>)> {
    let mut interfaces = Vec::new();
    let mut bonds = Vec::new();
    let dns_servers = get_dns_servers()?;
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

    // according to the folks from packet, all the addresses given to us in the
    // network section should be attached to the first bond we find in the list
    // of interfaces. we should always have at least one bond listed, but if we
    // don't find any, we just print out a scary warning and don't attach the
    // addresses to anything.
    if bonds.is_empty() {
        warn!("no bond interfaces. addresses are left unassigned.");
        // the rest of the function operates on bonds, so just return
        return Ok((interfaces, vec![]));
    }

    // remove panics if the index is out of bounds, but we know that there is at
    // least one bond in the vector because we return if it's empty
    let (first_mac, mut first_bond) = bonds.remove(0);
    for a in netinfo.addresses.clone() {
        let prefix = ipnetwork::ip_mask_to_prefix(a.netmask)
            .chain_err(|| "invalid network mask")?;
        first_bond.ip_addresses.push(IpNetwork::new(a.address, prefix)
                                .chain_err(|| "invalid IP address or prefix")?);
        let dest = match (a.public,a.address) {
            (false,IpAddr::V4(_)) =>
                IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(10,0,0,0),8).unwrap()),
            (true,IpAddr::V4(_)) =>
                IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(0,0,0,0),0).unwrap()),
            (_,IpAddr::V6(_)) =>
                IpNetwork::V6(Ipv6Network::new(Ipv6Addr::new(0,0,0,0,0,0,0,0),0).unwrap()),
        };
        first_bond.routes.push(NetworkRoute {
            destination: dest,
            gateway: a.gateway,
        });
    }
    bonds.push((first_mac, first_bond));

    let mut attrs = vec![
        ("TransmitHashPolicy".to_owned(), "layer3+4".to_owned()),
        ("MIIMonitorSec".to_owned(), ".1".to_owned()),
        ("UpDelaySec".to_owned(), ".2".to_owned()),
        ("DownDelaySec".to_owned(), ".2".to_owned()),
        ("Mode".to_owned(), network::bonding_mode_to_string(&netinfo.bonding.mode)?),
    ];
    if netinfo.bonding.mode == network::BONDING_MODE_LACP {
        attrs.push(("LACPTransmitRate".to_owned(), "fast".to_owned()));
    }

    let mut network_devices = vec![];
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

fn get_attrs(data: &PacketData) -> Result<Vec<(String,String)>> {
    let mut attrs = Vec::new();
    let mut v4_public_counter = 0;
    let mut v4_private_counter = 0;
    let mut v6_public_counter = 0;
    let mut v6_private_counter = 0;
    for a in data.network.addresses.clone() {
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
    attrs.push(("PACKET_HOSTNAME".to_owned(), data.hostname.clone()));
    attrs.push(("PACKET_PHONE_HOME_URL".to_owned(), data.phone_home_url.clone()));
    Ok(attrs)
}
