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

pub struct IpNetwork {
    addr: IpAddr,
    prefix: u8,
}

pub struct NetworkRoute {
    destination: IpNetwork,
    gateway: IpAddr,
}

pub struct MacAddr(pub u8, pub u8, pub u8,pub u8, pub u8, pub u8, pub u8, pub u8);

pub struct Interface {
    name: String,
    mac_address: MacAddr,
    priority: u32,
    nameservers: Vec<IpAddr>,
    ip_addresses: Vec<IpNetwork>,
    routes: Vec<NetworkRoute>,
    bond: String
}

pub struct Section {
    name: String,
    attributes: Vec<(String, String)>,
}

pub struct Device {
    name: String,
    kind: String,
    mac_address: MacAddr,
    priority: u32,
    sections: Vec<Section>
}

impl Interface {
    pub fn unit_name(&self) -> String {
        String::new()
    }
    pub fn config(&self) -> String {
        String::new()
    }
}

impl Device {
    pub fn unit_name(&self) -> String {
        String::new()
    }
    pub fn config(&self) -> String {
        String::new()
    }
}
