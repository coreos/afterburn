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

//! Providers
//!
//! These are the providers which coreos-metadata knows how to retrieve metadata
//! from. Internally, they handle the ins and outs of each providers metadata
//! services, and externally, they provide a function to fetch that metadata in
//! a regular format.
//!
//! To add a provider, put a `pub mod provider;` line in this file, export a
//! function to fetch the metadata, and then add a match line in the top-level
//! `fetch_metadata()` function in metadata.rs.

pub mod azure;
pub mod digitalocean;
pub mod cloudstack;
pub mod ec2;
pub mod gce;
pub mod openstack;
pub mod packet;
pub mod vagrant_virtualbox;

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::Path;

use update_ssh_keys::{AuthorizedKeys, AuthorizedKeyEntry};
use users;

use errors::*;
use network;

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

pub trait MetadataProvider {
    fn attributes(&self) -> Result<HashMap<String, String>>;
    fn hostname(&self) -> Result<Option<String>>;
    fn ssh_keys(&self) -> Result<Vec<AuthorizedKeyEntry>>;
    fn networks(&self) -> Result<Vec<network::Interface>>;
    fn network_devices(&self) -> Result<Vec<network::Device>>;

    fn write_attributes(&self, attributes_file_path: String) -> Result<()> {
        let mut attributes_file = create_file(&attributes_file_path)?;
        for (k,v) in self.attributes()? {
            writeln!(&mut attributes_file, "COREOS_{}={}", k, v)
                .chain_err(|| format!("failed to write attributes to file {:?}", attributes_file))?;
        }
        Ok(())
    }

    fn write_ssh_keys(&self, ssh_keys_user: String) -> Result<()> {
        let ssh_keys = self.ssh_keys()?;

        if !ssh_keys.is_empty() {
            // find the ssh keys user and open their ssh authorized keys directory
            let user = users::get_user_by_name(&ssh_keys_user)
                .ok_or_else(|| format!("could not find user with username {:?}", ssh_keys_user))?;
            let mut authorized_keys_dir = AuthorizedKeys::open(user, true, None)
                .chain_err(|| format!("failed to open authorized keys directory for user '{}'", ssh_keys_user))?;

            // add the ssh keys to the directory
            authorized_keys_dir.add_keys("coreos-metadata", ssh_keys, true, true)?;

            // write the changes and sync the directory
            authorized_keys_dir.write()
                .chain_err(|| "failed to update authorized keys directory")?;
            authorized_keys_dir.sync()
                .chain_err(|| "failed to update authorized keys")?;
        }

        Ok(())
    }

    fn write_hostname(&self, hostname_file_path: String) -> Result<()> {
        match self.hostname()? {
            Some(ref hostname) => {
                let mut hostname_file = create_file(&hostname_file_path)?;
                writeln!(&mut hostname_file, "{}", hostname)
                    .chain_err(|| format!("failed to write hostname {:?} to file {:?}", hostname, hostname_file))
            }
            None => Ok(())
        }
    }

    fn write_network_units(&self, network_units_dir: String) -> Result<()> {
        let dir_path = Path::new(&network_units_dir);
        fs::create_dir_all(&dir_path)
            .chain_err(|| format!("failed to create directory {:?}", dir_path))?;
        for interface in &self.networks()? {
            let file_path = dir_path.join(interface.unit_name());
            let mut unit_file = File::create(&file_path)
                .chain_err(|| format!("failed to create file {:?}", file_path))?;
            write!(&mut unit_file, "{}", interface.config())
                .chain_err(|| format!("failed to write network interface unit file {:?}", unit_file))?;
        }
        for device in &self.network_devices()? {
            let file_path = dir_path.join(device.unit_name());
            let mut unit_file = File::create(&file_path)
                .chain_err(|| format!("failed to create file {:?}", file_path))?;
            write!(&mut unit_file, "{}", device.config())
                .chain_err(|| format!("failed to write network device unit file {:?}", unit_file))?;
        }
        Ok(())
    }
}
