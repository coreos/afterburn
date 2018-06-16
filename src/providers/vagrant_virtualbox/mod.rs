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

//! vagrant/virtualbox metadata fetcher

use std::collections::HashMap;
use std::net::IpAddr;
use std::thread;
use std::time::Duration;

use hostname;
use pnet;
use update_ssh_keys::AuthorizedKeyEntry;

use errors::*;
use network;
use providers::MetadataProvider;

#[derive(Clone, Copy, Debug)]
pub struct VagrantVirtualboxProvider;

impl VagrantVirtualboxProvider {
    pub fn new() -> Result<VagrantVirtualboxProvider> {
        Ok(VagrantVirtualboxProvider)
    }

    fn get_ip() -> Result<String> {
        let max_attempts = 30;
        for _ in 0..max_attempts {
            let iface = VagrantVirtualboxProvider::find_eth1();
            if let Some(iface) = iface {
                for a in iface.ips {
                    if let IpAddr::V4(a) = a.ip() {
                        return Ok(format!("{}", a));
                    }
                }
            }
            info!("eth1 not found or is lacking an ipv4 address; waiting 2 seconds");
            thread::sleep(Duration::from_secs(2));
        }
        Err("eth1 was not found!".into())
    }

    fn find_eth1() -> Option<pnet::datalink::NetworkInterface> {
        let mut ifaces = pnet::datalink::interfaces();
        ifaces.retain(|i| i.name == "eth1");
        if !ifaces.is_empty() {
            Some(ifaces[0].clone())
        } else {
            None
        }
    }
}

impl MetadataProvider for VagrantVirtualboxProvider {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        let mut out = HashMap::with_capacity(2);

        let hostname = hostname::get_hostname().ok_or("unable to get hostname")?;
        let ip = VagrantVirtualboxProvider::get_ip()?;

        out.insert("VAGRANT_VIRTUALBOX_HOSTNAME".to_string(), hostname);
        out.insert("VAGRANT_VIRTUALBOX_PRIVATE_IPV4".to_string(), ip);

        Ok(out)
    }

    fn hostname(&self) -> Result<Option<String>> {
        Ok(hostname::get_hostname())
    }

    fn ssh_keys(&self) -> Result<Vec<AuthorizedKeyEntry>> {
        Ok(vec![])
    }

    fn networks(&self) -> Result<Vec<network::Interface>> {
        Ok(vec![])
    }

    fn network_devices(&self) -> Result<Vec<network::Device>> {
        Ok(vec![])
    }
}
