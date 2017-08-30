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
use std::io::Read;
use std::str::FromStr;

use util;

use network;
use network::{MacAddr,Interface,Device,Section,NetworkRoute};
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
        .add_ssh_keys(data.ssh_keys)
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
    let mut state = String::new();
    let mut f = File::open("/run/systemd/netif/state")?;
    f.read_to_string(&mut state)?;

    let ip_strings = util::key_lookup('=', "DNS", &state)
        .ok_or("DNS not found in netif state file")?;
    let mut addrs = Vec::new();
    for ip_string in ip_strings.split(' ') {
        addrs.push(IpAddr::from_str(&ip_string)
            .chain_err(|| format!("failed to parse IP address"))?);
    }
    if addrs.len() == 0 {
        return Err(format!("no DNS servers in /run/systemd/netif/state").into());
    }
    Ok(addrs)
}

fn parse_network(netinfo: &PacketNetworkInfo) -> Result<(Vec<Interface>,Vec<Device>)> {
    let mut interfaces = Vec::new();
    for i in netinfo.interfaces.clone() {
        interfaces.push(Interface {
            mac_address: Some(MacAddr::from_string(i.mac)?),
            bond: Some("bond0".to_owned()),
            name: None,
            priority: None,
            nameservers: Vec::new(),
            ip_addresses: Vec::new(),
            routes: Vec::new(),
        });
    }
    let mut iface = Interface{
        name: Some("bond0".to_owned()),
        priority: Some(5),
        nameservers: get_dns_servers()?,
        mac_address: None,
        bond: None,
        ip_addresses: Vec::new(),
        routes: Vec::new(),
    };
    for a in netinfo.addresses.clone() {
        let prefix = ipnetwork::ip_mask_to_prefix(a.netmask)
            .chain_err(|| format!("invalid network mask"))?;
        iface.ip_addresses.push(
            match a.address {
                IpAddr::V4(addrv4) => IpNetwork::V4(Ipv4Network::new(addrv4, prefix)
                    .chain_err(|| format!("invalid IP address or prefix"))?),
                IpAddr::V6(addrv6) => IpNetwork::V6(Ipv6Network::new(addrv6, prefix)
                    .chain_err(|| format!("invalid IP address or prefix"))?),
            }
        );
        let dest = match (a.public,a.address.clone()) {
            (false,IpAddr::V4(_)) =>
                    IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(10,0,0,0),8).unwrap()),
            (true,IpAddr::V4(_)) =>
                    IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(0,0,0,0),0).unwrap()),
            (_,IpAddr::V6(_)) =>
                    IpNetwork::V6(Ipv6Network::new(Ipv6Addr::new(0,0,0,0,0,0,0,0),0).unwrap()),
        };
        iface.routes.push(
            NetworkRoute {
                destination: dest,
                gateway: a.gateway,
            }

        );
    }
    interfaces.push(iface);

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
    let network_devices = vec![
        Device{
            name: "bond0".to_owned(),
            kind: "bond".to_owned(),
            mac_address: interfaces[0].mac_address
                .ok_or("first interface doesn't have a mac address, should be impossible")?
                .clone(),
            priority: Some(5),
            sections: vec![
                Section{
                    name: "Bond".to_owned(),
                    attributes: attrs,
                }
            ],
        },
    ];

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
                v4_public_counter = v4_public_counter + 1;
            }
            (IpAddr::V4(a),false) => {
                attrs.push((format!("PACKET_IPV4_PRIVATE_{}", v4_private_counter), format!("{}", a)));
                v4_private_counter = v4_private_counter + 1;
            }
            (IpAddr::V6(a),true) => {
                attrs.push((format!("PACKET_IPV6_PUBLIC_{}", v6_public_counter), format!("{}", a)));
                v6_public_counter = v6_public_counter + 1;
            }
            (IpAddr::V6(a),false) => {
                attrs.push((format!("PACKET_IPV6_PRIVATE_{}", v6_private_counter), format!("{}", a)));
                v6_private_counter = v6_private_counter + 1;
            }
        }
    }
    attrs.push(("PACKET_HOSTNAME".to_owned(), data.hostname.clone()));
    attrs.push(("PACKET_PHONE_HOME_URL".to_owned(), data.phone_home_url.clone()));
    Ok(attrs)
}
