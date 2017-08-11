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
use ssh;
use network;

pub struct Metadata {
    attributes: HashMap<String, String>,
    hostname: Option<String>,
    ssh_keys: Vec<String>,
    network: Vec<network::Interface>,
    net_dev: Vec<network::Device>,
}

fn create_file(filename: String) -> Result<File, String> {
    let file_path = Path::new(&filename);
    // create the directories if they don't exist
    let folder = file_path.parent()
        .ok_or(format!("could not get parent directory of {:?}", file_path))?;
    fs::create_dir_all(&folder)
        .map_err(wrap_error!("failed to create directory {:?}", folder))?;
    // create (or truncate) the file we want to write to
    File::create(file_path)
        .map_err(wrap_error!("failed to create file {:?}", file_path))
}

impl Metadata {
    pub fn write_attributes(&self, attributes_file_path: String) -> Result<(), String> {
        let mut attributes_file = create_file(attributes_file_path)?;
        for (k,v) in &self.attributes {
            write!(&mut attributes_file, "COREOS_{}={}\n", k, v)
                .map_err(wrap_error!("failed to write attributes to file {:?}", attributes_file))?;
        }
        Ok(())
    }
    pub fn write_ssh_keys(&self, ssh_keys_user: String) -> Result<(), String> {
        // this function actually needs to be pretty complicated
        // and we need a new tool that does this generically for rust anyway
        // so I actually just have to rewrite update-ssh-keys
        let user = users::get_user_by_name(ssh_keys_user.as_str())
            .ok_or(format!("could not find user with username {:?}", ssh_keys_user))?;
        let authorized_keys_dir = ssh::create_authorized_keys_dir(user)?;
        let mut authorized_keys_file = File::create(authorized_keys_dir.join("coreos-metadata"))
            .map_err(wrap_error!("failed to create the file {:?} in the ssh authorized users directory", "coreos-metadata"))?;
        for ssh_key in &self.ssh_keys {
            write!(&mut authorized_keys_file, "{}\n", ssh_key)
                .map_err(wrap_error!("failed to write ssh key to file {:?}", authorized_keys_file))?;
        }
        ssh::sync_authorized_keys(authorized_keys_dir)
    }
    pub fn write_hostname(&self, hostname_file_path: String) -> Result<(), String> {
        match self.hostname {
            Some(ref hostname) => {
                let mut hostname_file = create_file(hostname_file_path)?;
                write!(&mut hostname_file, "{}\n", hostname)
                    .map_err(wrap_error!("failed to write hostname {:?} to file {:?}", self.hostname, hostname_file))
            }
            None => Ok(())
        }
    }
    pub fn write_network_units(&self, network_units_dir: String) -> Result<(), String> {
        let dir_path = Path::new(&network_units_dir);
        fs::create_dir_all(&dir_path)
            .map_err(wrap_error!("failed to create directory {:?}", dir_path))?;
        for interface in &self.network {
            let file_path = dir_path.join(interface.unit_name());
            let mut unit_file = File::create(&file_path)
                .map_err(wrap_error!("failed to create file {:?}", file_path))?;
            write!(&mut unit_file, "{}", interface.config())
                .map_err(wrap_error!("failed to write network interface unit file {:?}", unit_file))?;
        }
        for device in &self.net_dev {
            let file_path = dir_path.join(device.unit_name());
            let mut unit_file = File::create(&file_path)
                .map_err(wrap_error!("failed to create file {:?}", file_path))?;
            write!(&mut unit_file, "{}", device.config())
                .map_err(wrap_error!("failed to write network device unit file {:?}", unit_file))?;
        }
        Ok(())
    }
}

/// fetch_metadata is the generic, top-level function that is used by the main
/// function to fetch metadata. The configured provider is passed in and this
/// function dispatches the call to the correct provider-specific fetch function
pub fn fetch_metadata(provider: &str) -> Result<Metadata, String> {
    match provider {
        _ => Err(format!("unknown provider '{}'", provider)),
    }
}
