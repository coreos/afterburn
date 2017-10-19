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

use metadata::Metadata;

use errors::*;

use retry;

header! {(MetadataFlavor, "Metadata-Flavor") => [String]}
const GOOGLE: &str = "Google";

fn url_for_key(key: &str) -> String {
    format!("http://metadata.google.internal/computeMetadata/v1/{}", key)
}

// Google's metadata service returns a 200 success even if there is no resource. If an empty body
// was returned, it means there was no result
fn empty_to_none(s: Option<String>) -> Option<String> {
    match s {
        Some(s) => if &s == "" { None } else { Some(s) },
        x => x,
    }
}

pub fn fetch_metadata() -> Result<Metadata> {
    let client = retry::Client::new()?
        .header(MetadataFlavor(GOOGLE.to_owned()))
        .return_on_404(true);
    let public: Option<String> = client.get(retry::Raw, url_for_key("instance/network-interfaces/0/access-configs/0/external-ip")).send()?;
    let local: Option<String> = client.get(retry::Raw, url_for_key("instance/network-interfaces/0/ip")).send()?;
    let hostname: Option<String> = client.get(retry::Raw, url_for_key("instance/hostname")).send()?;

    let ssh_keys = fetch_all_ssh_keys(&client)?;

    Ok(Metadata::builder()
        .add_attribute_if_exists("GCE_IP_LOCAL_0".to_owned(), empty_to_none(local))
        .add_attribute_if_exists("GCE_IP_EXTERNAL_0".to_owned(), empty_to_none(public))
        .add_attribute_if_exists("GCE_HOSTNAME".to_owned(), empty_to_none(hostname.clone()))
        .set_hostname_if_exists(hostname)
        .add_ssh_keys(ssh_keys)?
        .build())
}

fn fetch_all_ssh_keys(client: &retry::Client) -> Result<Vec<String>> {
    let keys = fetch_ssh_keys(client, "instance/attributes/sshKeys")?;
    if !keys.is_empty() {
        return Ok(keys);
    }
    let mut keys = fetch_ssh_keys(client, "instance/attributes/ssh-keys")?;

    let block_project_keys: Option<String> = client.clone().get(retry::Raw, url_for_key("instance/attributes/block-project-ssh-keys")).send()?;

    if block_project_keys == Some("true".to_owned()) {
        return Ok(keys);
    }

    keys.append(&mut fetch_ssh_keys(client, "project/attributes/sshKeys")?);

    Ok(keys)
}

fn fetch_ssh_keys(client: &retry::Client, key: &str) -> Result<Vec<String>> {
    let key_data: Option<String> = client.get(retry::Raw, url_for_key(key)).send()?;
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
