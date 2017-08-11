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
use std::fmt;
use std::string::String;
use std::string::ToString;

#[derive(Clone, Copy, Debug)]
pub struct IpNetwork {
    addr: IpAddr,
    prefix: u8,
}

impl fmt::Display for IpNetwork {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.addr, self.prefix)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct NetworkRoute {
    destination: IpNetwork,
    gateway: IpAddr,
}

#[derive(Clone, Copy, Debug)]
pub struct MacAddr(pub u8, pub u8, pub u8,pub u8, pub u8, pub u8, pub u8, pub u8);

impl fmt::Display for MacAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}", self.0, self.1, self.2, self.3, self.4, self.5, self.6, self.7)
    }
}

/// for naming purposes an interface needs either a name or an address.
/// it can have both. but it can't have neither.
/// there isn't really a way to express this in the type system
/// so we just panic! if it's not what we expected.
/// I guess that there aren't really type systems with inclusive disjunction
/// so it's not really that big of a deal.
#[derive(Clone, Debug)]
pub struct Interface {
    name: Option<String>,
    mac_address: Option<MacAddr>,
    priority: Option<u32>,
    nameservers: Vec<IpAddr>,
    ip_addresses: Vec<IpNetwork>,
    routes: Vec<NetworkRoute>,
    bond: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Section {
    name: String,
    attributes: Vec<(String, String)>,
}

#[derive(Clone, Debug)]
pub struct Device {
    name: String,
    kind: String,
    mac_address: MacAddr,
    priority: Option<u32>,
    sections: Vec<Section>
}

impl Interface {
    pub fn unit_name(&self) -> String {
        format!("{:02}-{}.network",
                self.priority.unwrap_or(10),
                self.name.clone().unwrap_or(
                    self.mac_address.unwrap_or_else(
                        // needs to be a lambda or we panic immediately
                        // yay, manual thunking!
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
