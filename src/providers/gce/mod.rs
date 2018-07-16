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

//! google compute engine metadata fetcher

use std::collections::HashMap;

use openssh_keys::PublicKey;
use update_ssh_keys::AuthorizedKeyEntry;

use errors::*;
use network;
use providers::MetadataProvider;
use retry;

header! {(MetadataFlavor, "Metadata-Flavor") => [String]}
const GOOGLE: &str = "Google";

#[derive(Clone, Debug)]
pub struct GceProvider {
    client: retry::Client,
}

impl GceProvider {
    pub fn new() -> Result<GceProvider> {
        let client = retry::Client::new()?
            .header(MetadataFlavor(GOOGLE.to_owned()))
            .return_on_404(true);

        Ok(GceProvider { client })
    }

    fn endpoint_for(name: &str) -> String {
        format!("http://metadata.google.internal/computeMetadata/v1/{}", name)
    }

    fn fetch_all_ssh_keys(&self) -> Result<Vec<String>> {
        let keys = self.fetch_ssh_keys("instance/attributes/sshKeys")?;
        if !keys.is_empty() {
            return Ok(keys);
        }
        let mut keys = self.fetch_ssh_keys("instance/attributes/ssh-keys")?;

        let block_project_keys: Option<String> = self.client
            .clone()
            .get(retry::Raw, GceProvider::endpoint_for("instance/attributes/block-project-ssh-keys"))
            .send()?;

        if block_project_keys == Some("true".to_owned()) {
            return Ok(keys);
        }

        keys.append(&mut self.fetch_ssh_keys("project/attributes/sshKeys")?);

        Ok(keys)
    }

    fn fetch_ssh_keys(&self, key: &str) -> Result<Vec<String>> {
        let key_data: Option<String> = self.client.get(retry::Raw, GceProvider::endpoint_for(key)).send()?;
        if let Some(key_data) = key_data {
            let mut keys = Vec::new();
            for l in key_data.lines() {
                if l.is_empty() {
                    continue
                }
                let mut l = l.to_owned();
                let index = l.find(':')
                    .ok_or("character ':' not found in line in key data")?;
                keys.push(l.split_off(index+1));
            }
            Ok(keys)
        } else {
            // The user must have not provided any keys
            Ok(Vec::new())
        }
    }
}

impl MetadataProvider for GceProvider {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        let mut out = HashMap::with_capacity(3);

        let add_value = |map: &mut HashMap<_, _>, key: &str, name| -> Result<()> {
            let value: Option<String> = self.client.get(retry::Raw, GceProvider::endpoint_for(name)).send()?;

            if let Some(value) = value {
                if !value.is_empty() {
                    map.insert(key.to_string(), value);
                }
            }

            Ok(())
        };

        add_value(&mut out, "GCE_HOSTNAME", "instance/hostname")?;
        add_value(&mut out, "GCE_IP_EXTERNAL_0", "instance/network-interfaces/0/access-configs/0/external-ip")?;
        add_value(&mut out, "GCE_IP_LOCAL_0", "instance/network-interfaces/0/ip")?;

        Ok(out)
    }

    fn hostname(&self) -> Result<Option<String>> {
        self.client.get(retry::Raw, GceProvider::endpoint_for("instance/hostname")).send()
    }

    fn ssh_keys(&self) -> Result<Vec<AuthorizedKeyEntry>> {
        let mut out = Vec::new();

        for key in &self.fetch_all_ssh_keys()? {
            let key = PublicKey::parse(&key)?;
            out.push(AuthorizedKeyEntry::Valid{key});
        }

        Ok(out)
    }

    fn networks(&self) -> Result<Vec<network::Interface>> {
        Ok(vec![])
    }

    fn network_devices(&self) -> Result<Vec<network::Device>> {
        Ok(vec![])
    }
}
