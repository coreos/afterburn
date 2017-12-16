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

//! network deals abstracts away the manipulation of network device and
//! interface unit files. All that is left is to write the resulting string to
//! the necessary unit.

use std::net::IpAddr;
use pnet::util::MacAddr;
use std::string::String;
use std::string::ToString;

use ipnetwork::IpNetwork;
use errors::*;

pub const BONDING_MODE_BALANCE_RR: u32 = 0;
pub const BONDING_MODE_ACTIVE_BACKUP: u32 = 1;
pub const BONDING_MODE_BALANCE_XOR: u32 = 2;
pub const BONDING_MODE_BROADCAST: u32 = 3;
pub const BONDING_MODE_LACP: u32 = 4;
pub const BONDING_MODE_BALANCE_TLB: u32 = 5;
pub const BONDING_MODE_BALANCE_ALB: u32 = 6;

const BONDING_MODES: [(u32,&str); 7] = [
    (BONDING_MODE_BALANCE_RR,"balance-rr"),
    (BONDING_MODE_ACTIVE_BACKUP,"active-backup"),
    (BONDING_MODE_BALANCE_XOR,"balance-xor"),
    (BONDING_MODE_BROADCAST,"broadcast"),
    (BONDING_MODE_LACP,"802.3ad"),
    (BONDING_MODE_BALANCE_TLB,"balance-tlb"),
    (BONDING_MODE_BALANCE_ALB,"balance-alb"),
];

pub fn bonding_mode_to_string(mode: &u32) -> Result<String> {
    for &(m,s) in &BONDING_MODES {
        if m == *mode {
            return Ok(s.to_owned())
        }
    }
    Err(format!("no such bonding mode: {}", mode).into())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NetworkRoute {
    pub destination: IpNetwork,
    pub gateway: IpAddr,
}

/// for naming purposes an interface needs either a name or an address.
/// it can have both. but it can't have neither.
/// there isn't really a way to express this in the type system
/// so we just panic! if it's not what we expected.
/// I guess that there aren't really type systems with inclusive disjunction
/// so it's not really that big of a deal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Interface {
    pub name: Option<String>,
    pub mac_address: Option<MacAddr>,
    pub priority: Option<u32>,
    pub nameservers: Vec<IpAddr>,
    pub ip_addresses: Vec<IpNetwork>,
    pub routes: Vec<NetworkRoute>,
    pub bond: Option<String>,
    pub unmanaged: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Section {
    pub name: String,
    pub attributes: Vec<(String, String)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Device {
    pub name: String,
    pub kind: String,
    pub mac_address: MacAddr,
    pub priority: Option<u32>,
    pub sections: Vec<Section>
}

impl Interface {
    pub fn unit_name(&self) -> String {
        format!("{:02}-{}.network",
                self.priority.unwrap_or(10),
                self.name.clone().unwrap_or_else(
                    // needs to be a lambda or we panic immediately
                    // yay, manual thunking!
                    ||self.mac_address.unwrap_or_else(
                        ||panic!("interface needs either name or mac address (or both)")
                    ).to_string()
                ))
    }
    pub fn config(&self) -> String {
        let mut config = String::new();

        // [Match] section
        config.push_str("[Match]\n");
        self.name.clone().map(|name| config.push_str(&format!("Name={}\n", name)));
        self.mac_address.map(|mac| config.push_str(&format!("MACAddress={}\n", mac)));

        // [Network] section
        config.push_str("\n[Network]\n");
        for ns in &self.nameservers {
            config.push_str(&format!("DNS={}\n", ns))
        }
        self.bond.clone().map(|bond| config.push_str(&format!("Bond={}\n", bond)));

        // [Link] section
        if self.unmanaged {
            config.push_str("\n[Link]\nUnmanaged=yes\n");
        }

        // [Address] sections
        for addr in &self.ip_addresses {
            config.push_str(&format!("\n[Address]\nAddress={}\n", addr));
        }

        // [Route] sections
        for route in &self.routes {
            config.push_str(&format!("\n[Route]\nDestination={}\nGateway={}\n", route.destination, route.gateway));
        }

        config
    }
}

impl Device {
    pub fn unit_name(&self) -> String {
        format!("{:02}-{}.netdev", self.priority.unwrap_or(10), self.name)
    }
    pub fn config(&self) -> String {
        let mut config = String::new();

        // [NetDev] section
        config.push_str("[NetDev]\n");
        config.push_str(&format!("Name={}\n", self.name));
        config.push_str(&format!("Kind={}\n", self.kind));
        config.push_str(&format!("MACAddress={}\n", self.mac_address));

        // custom sections
        for section in &self.sections {
            config.push_str(&format!("\n[{}]\n", section.name));
            for attr in &section.attributes {
                config.push_str(&format!("{}={}\n", attr.0, attr.1));
            }
        }

        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};
    use ipnetwork::{Ipv4Network,Ipv6Network};

    #[test]
    fn mac_addr_display() {
        let m = MacAddr(0xf4,0x00,0x34,0x09,0x73,0xee);
        assert_eq!(m.to_string(), "f4:00:34:09:73:ee");
    }

    #[test]
    fn interface_unit_name() {
        let is = vec![
            (Interface {
                name: Some(String::from("lo")),
                mac_address: Some(MacAddr(0,0,0,0,0,0)),
                priority: Some(20),
                nameservers: vec![],
                ip_addresses: vec![],
                routes: vec![],
                bond: None,
                unmanaged: false,
            }, "20-lo.network"),
            (Interface {
                name: Some(String::from("lo")),
                mac_address: Some(MacAddr(0,0,0,0,0,0)),
                priority: None,
                nameservers: vec![],
                ip_addresses: vec![],
                routes: vec![],
                bond: None,
                unmanaged: false,
            }, "10-lo.network"),
            (Interface {
                name: None,
                mac_address: Some(MacAddr(0,0,0,0,0,0)),
                priority: Some(20),
                nameservers: vec![],
                ip_addresses: vec![],
                routes: vec![],
                bond: None,
                unmanaged: false,
            }, "20-00:00:00:00:00:00.network"),
            (Interface {
                name: Some(String::from("lo")),
                mac_address: None,
                priority: Some(20),
                nameservers: vec![],
                ip_addresses: vec![],
                routes: vec![],
                bond: None,
                unmanaged: false,
            }, "20-lo.network"),
        ];

        for (i, s) in is {
            assert_eq!(i.unit_name(), s);
        }
    }

    #[test]
    #[should_panic]
    fn interface_unit_name_no_name_no_mac() {
        let i = Interface {
            name: None,
            mac_address: None,
            priority: Some(20),
            nameservers: vec![],
            ip_addresses: vec![],
            routes: vec![],
            bond: None,
            unmanaged: false,
        };
        let _name = i.unit_name();
    }

    #[test]
    fn device_unit_name() {
        let ds = vec![
            (Device {
                name: String::from("vlan0"),
                kind: String::from("vlan"),
                mac_address: MacAddr(0,0,0,0,0,0),
                priority: Some(20),
                sections: vec![],
            }, "20-vlan0.netdev"),
            (Device {
                name: String::from("vlan0"),
                kind: String::from("vlan"),
                mac_address: MacAddr(0,0,0,0,0,0),
                priority: None,
                sections: vec![],
            }, "10-vlan0.netdev"),
        ];

        for (d, s) in ds {
            assert_eq!(d.unit_name(), s);
        }
    }

    #[test]
    fn interface_config() {
        let is = vec![
            (Interface {
                name: Some(String::from("lo")),
                mac_address: Some(MacAddr(0,0,0,0,0,0)),
                priority: Some(20),
                nameservers: vec![
                    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                    IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)),
                ],
                ip_addresses: vec![
                    IpNetwork::V4(Ipv4Network::new(
                            Ipv4Addr::new(127, 0, 0, 1),
                            8
                        ).unwrap()),
                    IpNetwork::V6(Ipv6Network::new(
                            Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1),
                            128
                        ).unwrap()),
                ],
                routes: vec![
                    NetworkRoute {
                        destination: IpNetwork::V4(Ipv4Network::new(
                                Ipv4Addr::new(127, 0, 0, 1),
                                8
                            ).unwrap()),
                        gateway: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                    }
                ],
                bond: Some(String::from("james")),
                unmanaged: false,
            }, "[Match]
Name=lo
MACAddress=00:00:00:00:00:00

[Network]
DNS=127.0.0.1
DNS=::1
Bond=james

[Address]
Address=127.0.0.1/8

[Address]
Address=::1/128

[Route]
Destination=127.0.0.1/8
Gateway=127.0.0.1
"),
            // this isn't really a valid interface object, but it's testing
            // the minimum possible configuration for all peices at the same
            // time, so I'll allow it. (sdemos)
            (Interface {
                name: None,
                mac_address: None,
                priority: None,
                nameservers: vec![],
                ip_addresses: vec![],
                routes: vec![],
                bond: None,
                unmanaged: false,
            }, "[Match]

[Network]
")
        ];

        for (i, s) in is {
            assert_eq!(i.config(), s);
        }
    }

    #[test]
    fn device_config() {
        let ds = vec![
            (Device {
                name: String::from("vlan0"),
                kind: String::from("vlan"),
                mac_address: MacAddr(0,0,0,0,0,0),
                priority: Some(20),
                sections: vec![
                    Section {
                        name: String::from("Test"),
                        attributes: vec![
                            (String::from("foo"), String::from("bar")),
                            (String::from("oingo"), String::from("boingo")),
                        ]
                    },
                    Section {
                        name: String::from("Empty"),
                        attributes: vec![],
                    }
                ],
            }, "[NetDev]
Name=vlan0
Kind=vlan
MACAddress=00:00:00:00:00:00

[Test]
foo=bar
oingo=boingo

[Empty]
"),
            (Device {
                name: String::from("vlan0"),
                kind: String::from("vlan"),
                mac_address: MacAddr(0,0,0,0,0,0),
                priority: Some(20),
                sections: vec![],
            }, "[NetDev]
Name=vlan0
Kind=vlan
MACAddress=00:00:00:00:00:00
")
        ];

        for (d, s) in ds {
            assert_eq!(d.config(), s);
        }
    }
}
