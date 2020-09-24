/*
Copyright 2020 Google LLC

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
use slog_scope::warn;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::path::{Path, PathBuf};

use nix::mount;
use openssh_keys::PublicKey;
use tempfile;

use crate::errors::*;
use crate::network;
use crate::providers::MetadataProvider;
use pnet_base::MacAddr;

#[cfg(test)]
mod tests;

const ENV_PREFIX: &str = "ONE_";
const CONTEXT_DRIVE_LABEL: &str = "CONTEXT";
const CONTEXT_SCRIPT_NAME: &str = "context.sh";

#[derive(Debug)]
pub struct ContextDrive {
    contents: String,
    attributes: HashMap<String, String>,
    device: Option<PathBuf>,
    mount_point: Option<PathBuf>,
}

impl ContextDrive {
    pub fn try_new() -> Result<Self> {
        // Mount disk by label to a new tempdir
        let target = tempfile::Builder::new()
            .prefix("afterburn-")
            .tempdir()
            .chain_err(|| "failed to create temporary directory")?;
        let device = Path::new("/dev/disk/by-label/").join(CONTEXT_DRIVE_LABEL);
        ContextDrive::mount_ro(&device, target.path(), "iso9660")?;
        let filename = target.path().join(CONTEXT_SCRIPT_NAME);
        let mut file =
            File::open(&filename).chain_err(|| format!("failed to open file '{:?}'", filename))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .chain_err(|| format!("failed to read from file '{:?}'", filename))?;
        Ok(ContextDrive {
            contents: contents.to_string(),
            attributes: ContextDrive::fetch_all_values(contents.to_string()),
            device: Some(device.to_owned()),
            mount_point: Some(target.path().to_owned()),
        })
    }

    #[allow(dead_code)]
    pub fn try_new_from_string(contents: &String) -> Result<Self> {
        Ok(ContextDrive {
            contents: contents.to_owned(),
            attributes: ContextDrive::fetch_all_values(contents.to_string()),
            device: None,
            mount_point: None,
        })
    }

    fn fetch_all_values(contents: String) -> HashMap<String, String> {
        let mut res = HashMap::new();
        for line in contents.lines() {
            let l = line.trim();
            if !l.starts_with("#") && l.len() > 2 {
                let v: Vec<&str> = l.split("=").collect();
                if v.len() == 2 {
                    // Line are formatted as KEY='value', for bash-usability. This should extract
                    // them fairly safely by stripping off surrounding ' marks and trimming
                    res.insert(
                        v[0].to_string(),
                        v[1].to_string()
                            .strip_prefix("'")
                            .unwrap_or("")
                            .strip_suffix("'")
                            .unwrap_or("")
                            .trim()
                            .to_string(),
                    );
                }
            }
        }
        res
    }

    fn fetch_value(&self, key: &str) -> Option<&String> {
        self.attributes.get(key)
    }

    fn fetch_publickeys(&self) -> Result<Vec<PublicKey>> {
        let val = self.fetch_value("SSH_PUBLIC_KEY");
        if val.is_none() {
            return Ok(vec![]);
        }
        ContextDrive::parse_publickeys(val.unwrap())
    }

    fn parse_publickeys(s: &str) -> Result<Vec<PublicKey>> {
        let res = PublicKey::parse(s)?;
        Ok(vec![res])
    }

    fn fetch_networks(&self) -> Result<Vec<network::Interface>> {
        let mut interfaces: HashMap<String, network::Interface> = HashMap::new();
        for (k, v) in self.attributes.iter() {
            let chunks: Vec<&str> = k.splitn(2, "_").collect();
            let name = chunks[0].to_string();
            if name.starts_with("ETH") {
                if !interfaces.contains_key(&name) {
                    interfaces.insert(
                        name.to_string(),
                        network::Interface {
                            name: None,
                            mac_address: None,
                            nameservers: vec![],
                            ip_addresses: vec![],
                            routes: vec![],
                            bond: None,
                            priority: 10,
                            unmanaged: false,
                        },
                    );
                }
                let int = interfaces.get_mut(chunks[0]).unwrap();
                match chunks[1] {
                    "MAC" => {
                        int.mac_address = Some(v.parse::<MacAddr>().unwrap());
                    }
                    "IP" => {
                        // Break out the mask value into a prefix-length from a different attribute
                        let mask_attr_name = &(name.clone() + "_MASK");
                        let prefix_length = ipnetwork::ip_mask_to_prefix(
                            self.fetch_value(mask_attr_name)
                                .unwrap()
                                .parse::<IpAddr>()
                                .unwrap(),
                        )
                        .unwrap();
                        let address = IpNetwork::V4(
                            Ipv4Network::new(v.parse::<Ipv4Addr>().unwrap(), prefix_length)
                                .unwrap(),
                        );
                        int.ip_addresses.push(address);
                    }
                    "GATEWAY" => int.routes.push(network::NetworkRoute {
                        destination: IpNetwork::V4(
                            Ipv4Network::new(Ipv4Addr::new(0, 0, 0, 0), 0).unwrap(),
                        ),
                        gateway: v.parse().unwrap(),
                    }),
                    "IP6" => {
                        let mask_attr_name = &(name.clone() + "_IP6_PREFIX_LENGTH");
                        let prefix_length = self
                            .fetch_value(mask_attr_name)
                            .unwrap()
                            .parse::<u8>()
                            .unwrap();
                        let address = IpNetwork::V6(
                            Ipv6Network::new(v.parse::<Ipv6Addr>().unwrap(), prefix_length)
                                .unwrap(),
                        );
                        int.ip_addresses.push(address);
                    }
                    "DNS" => {
                        let nameservers: Vec<IpAddr> =
                            v.split(" ").map(|d| d.parse::<IpAddr>().unwrap()).collect();
                        int.nameservers.extend_from_slice(&nameservers);
                    }
                    _ => {}
                };
            }
        }
        let mut res: Vec<network::Interface> = Vec::new();
        for v in interfaces.values() {
            res.push(v.to_owned());
        }
        Ok(res)
    }

    fn mount_ro(source: &Path, target: &Path, fstype: &str) -> Result<()> {
        mount::mount(
            Some(source),
            target,
            Some(fstype),
            mount::MsFlags::MS_RDONLY,
            None::<&str>,
        )
        .chain_err(|| {
            format!(
                "failed to read-only mount source '{:?}' to target '{:?}' with filetype '{}'",
                source, target, fstype
            )
        })
    }

    fn unmount(target: &Path) -> Result<()> {
        mount::umount(target).chain_err(|| format!("failed to unmount target '{:?}'", target))
    }
}

impl MetadataProvider for ContextDrive {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        let mut res: HashMap<String, String> = HashMap::new();
        for (k, v) in self.attributes.clone() {
            res.insert(format!("{}{}", ENV_PREFIX, k), v);
        }
        Ok(res)
    }

    fn hostname(&self) -> Result<Option<String>> {
        let hostname = self.fetch_value("SET_HOSTNAME");
        if hostname.is_some() {
            return Ok(Some(hostname.unwrap().to_owned()));
        }
        Ok(None)
    }

    fn ssh_keys(&self) -> Result<Vec<PublicKey>> {
        self.fetch_publickeys()
    }

    fn networks(&self) -> Result<Vec<network::Interface>> {
        self.fetch_networks()
    }

    fn virtual_network_devices(&self) -> Result<Vec<network::VirtualNetDev>> {
        warn!("virtual network devices metadata requested, but not supported on this platform");
        Ok(vec![])
    }

    fn boot_checkin(&self) -> Result<()> {
        warn!("boot check-in requested, but not supported on this platform");
        Ok(())
    }
}

impl ::std::ops::Drop for ContextDrive {
    fn drop(&mut self) {
        if self.mount_point.is_some() {
            let path = self.mount_point.as_ref();
            ContextDrive::unmount(&path.unwrap()).unwrap();
        }
    }
}
