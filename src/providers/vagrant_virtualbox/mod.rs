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

use openssh_keys::PublicKey;
use slog_scope::{info, warn};

use crate::errors::*;
use crate::network;
use crate::providers::MetadataProvider;

#[derive(Clone, Copy, Debug)]
pub struct VagrantVirtualboxProvider;

impl VagrantVirtualboxProvider {
    pub fn new() -> Self {
        Self
    }

    fn get_ip() -> Result<String> {
        let max_attempts = 30;
        for _ in 0..max_attempts {
            if let Some(iface) = Self::find_eth1() {
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

    fn find_eth1() -> Option<pnet_datalink::NetworkInterface> {
        pnet_datalink::interfaces()
            .into_iter()
            .find(|i| i.name == "eth1")
    }

    /// Get the hostname from local system settings.
    fn system_hostname() -> Result<String> {
        let hostname = hostname::get()
            .chain_err(|| "unable to get hostname")?
            .to_string_lossy()
            .into_owned();
        Ok(hostname)
    }
}

impl MetadataProvider for VagrantVirtualboxProvider {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        let hostname = Self::system_hostname()?;
        let ip = Self::get_ip()?;

        let attributes = maplit::hashmap! {
            "VAGRANT_VIRTUALBOX_HOSTNAME".to_string() => hostname,
            "VAGRANT_VIRTUALBOX_PRIVATE_IPV4".to_string() => ip,
        };

        Ok(attributes)
    }

    fn ssh_keys(&self) -> Result<Vec<PublicKey>> {
        Ok(vec![])
    }
}
