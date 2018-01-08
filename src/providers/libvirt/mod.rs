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

//! libvirt metadata fetcher

use metadata::Metadata;

use errors::*;

use std::net::IpAddr;
use std::time::Duration;
use std::thread;

use hostname;
use pnet;

pub fn fetch_metadata() -> Result<Metadata> {
    let h = hostname::get_hostname().ok_or("unable to get hostname")?;
    let ip = get_ip()?;

    Ok(Metadata::builder()
       .add_attribute("LIBVIRT_PRIVATE_IPV4".to_owned(), ip)
       .add_attribute("LIBVIRT_HOSTNAME".to_owned(), h.clone())
       .set_hostname(h)
       .build())
}

fn get_ip() -> Result<String> {
    let max_attempts = 30;
    for _ in 0..max_attempts {
        let iface = find_eth0();
        if let Some(iface) = iface {
            for a in iface.ips {
                if let IpAddr::V4(a) = a.ip() {
                    return Ok(format!("{}", a));
                }
            }
        }
        info!("eth0 not found or is lacking an ipv4 address; waiting 2 seconds");
        thread::sleep(Duration::from_secs(2));
    }
    Err("eth0 was not found!".into())
}

fn find_eth0() -> Option<pnet::datalink::NetworkInterface> {
    let mut ifaces = pnet::datalink::interfaces();
    ifaces.retain(|i| i.name == "eth1");
    if !ifaces.is_empty() {
        Some(ifaces[0].clone())
    } else {
        None
    }
}
