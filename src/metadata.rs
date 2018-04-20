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

use std::fs;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;
use std::collections::HashMap;
use users;
use openssh_keys::PublicKey;
use update_ssh_keys::{AuthorizedKeys, AuthorizedKeyEntry};
use network;

use errors::*;

#[derive(Default, Debug, Clone)]
pub struct MetadataBuilder {
    metadata: Metadata,
}

#[derive(Default, Debug, Clone)]
pub struct Metadata {
    attributes: HashMap<String, String>,
    hostname: Option<String>,
    ssh_keys: Vec<AuthorizedKeyEntry>,
    network: Vec<network::Interface>,
    net_dev: Vec<network::Device>,
}

fn create_file(filename: &str) -> Result<File> {
    let file_path = Path::new(&filename);
    // create the directories if they don't exist
    let folder = file_path.parent()
        .ok_or_else(|| format!("could not get parent directory of {:?}", file_path))?;
    fs::create_dir_all(&folder)
        .chain_err(|| format!("failed to create directory {:?}", folder))?;
    // create (or truncate) the file we want to write to
    File::create(file_path)
        .chain_err(|| format!("failed to create file {:?}", file_path))
}

impl MetadataBuilder {
    pub fn new() -> Self {
        MetadataBuilder {
            metadata: Metadata::new(),
        }
    }

    pub fn add_attribute(mut self, key: String, value: String) -> Self {
        self.metadata.attributes.insert(key, value);
        self
    }

    pub fn add_attribute_if_exists(self, key: String, value: Option<String>) -> Self {
        match value {
            Some(v) => self.add_attribute(key, v),
            None => self
        }
    }

    pub fn set_hostname(mut self, hostname: String) -> Self {
        self.metadata.hostname = Some(hostname);
        self
    }

    pub fn set_hostname_if_exists(self, hostname: Option<String>) -> Self {
        match hostname {
            Some(v) => self.set_hostname(v),
            None => self
        }
    }

    pub fn add_ssh_keys(mut self, ssh_keys: Vec<String>) -> Result<Self> {
        for key in ssh_keys {
            let key = PublicKey::parse(&key)?;
            self.metadata.ssh_keys.push(AuthorizedKeyEntry::Valid{key});
        }
        Ok(self)
    }

    pub fn add_publickeys(mut self, ssh_keys: Vec<PublicKey>) -> Self {
        self.metadata.ssh_keys.extend(ssh_keys.into_iter().map(|key| AuthorizedKeyEntry::Valid{key}));
        self
    }

    pub fn add_network_interface(mut self, interface: network::Interface) -> Self {
        self.metadata.network.push(interface);
        self
    }

    pub fn add_network_device(mut self, device: network::Device) -> Self {
        self.metadata.net_dev.push(device);
        self
    }

    pub fn build(self) -> Metadata {
        self.metadata
    }
}

impl Metadata {
    pub fn builder() -> MetadataBuilder {
        MetadataBuilder::new()
    }

    pub fn new() -> Self {
        Metadata {
            attributes: HashMap::new(),
            hostname: None,
            ssh_keys: vec![],
            network: vec![],
            net_dev: vec![],
        }
    }

    pub fn write_attributes(&self, attributes_file_path: String) -> Result<()> {
        let mut attributes_file = create_file(&attributes_file_path)?;
        for (k,v) in &self.attributes {
            write!(&mut attributes_file, "COREOS_{}={}\n", k, v)
                .chain_err(|| format!("failed to write attributes to file {:?}", attributes_file))?;
        }
        Ok(())
    }
    pub fn write_ssh_keys(&self, ssh_keys_user: String) -> Result<()> {
        // find the ssh keys user and open their ssh authorized keys directory
        let user = users::get_user_by_name(&ssh_keys_user)
            .ok_or_else(|| format!("could not find user with username {:?}", ssh_keys_user))?;
        let mut authorized_keys_dir = AuthorizedKeys::open(user, true, None)
            .chain_err(|| format!("failed to open authorzied keys directory for user '{}'", ssh_keys_user))?;

        // add the ssh keys to the directory
        authorized_keys_dir.add_keys("coreos-metadata", self.ssh_keys.clone(), true, true)?;

        // write the changes and sync the directory
        authorized_keys_dir.write()
            .chain_err(|| "failed to update authorized keys directory")?;
        authorized_keys_dir.sync()
            .chain_err(|| "failed to update authorized keys")
    }
    pub fn write_hostname(&self, hostname_file_path: String) -> Result<()> {
        match self.hostname {
            Some(ref hostname) => {
                let mut hostname_file = create_file(&hostname_file_path)?;
                write!(&mut hostname_file, "{}\n", hostname)
                    .chain_err(|| format!("failed to write hostname {:?} to file {:?}", self.hostname, hostname_file))
            }
            None => Ok(())
        }
    }
    pub fn write_network_units(&self, network_units_dir: String) -> Result<()> {
        let dir_path = Path::new(&network_units_dir);
        fs::create_dir_all(&dir_path)
            .chain_err(|| format!("failed to create directory {:?}", dir_path))?;
        for interface in &self.network {
            let file_path = dir_path.join(interface.unit_name());
            let mut unit_file = File::create(&file_path)
                .chain_err(|| format!("failed to create file {:?}", file_path))?;
            write!(&mut unit_file, "{}", interface.config())
                .chain_err(|| format!("failed to write network interface unit file {:?}", unit_file))?;
        }
        for device in &self.net_dev {
            let file_path = dir_path.join(device.unit_name());
            let mut unit_file = File::create(&file_path)
                .chain_err(|| format!("failed to create file {:?}", file_path))?;
            write!(&mut unit_file, "{}", device.config())
                .chain_err(|| format!("failed to write network device unit file {:?}", unit_file))?;
        }
        Ok(())
    }
}
