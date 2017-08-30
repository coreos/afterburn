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

//! aws ec2 metadata fetcher
//!
use errors::*;
use metadata::Metadata;
use retry;

#[cfg(test)]
mod mock_tests;
#[cfg(test)]
use self::mock_tests::URL;

#[cfg(not(test))]
const URL: &'static str = "http://169.254.169.254/2009-04-04";

fn url_for_key(key: &str) -> String {
    format!("{}/{}", URL, key)
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct InstanceIdDoc {
    region: String,
}

pub fn fetch_metadata() -> Result<Metadata> {
    let client = retry::Client::new()
        .chain_err(|| "ec2: failed to create http client")?
        .return_on_404(true);

    let instance_id: Option<String> = client.get(retry::Raw, url_for_key("meta-data/instance-id")).send()?;
    let public: Option<String> = client.get(retry::Raw, url_for_key("meta-data/public-ipv4")).send()?;
    let local: Option<String> = client.get(retry::Raw, url_for_key("meta-data/local-ipv4")).send()?;
    let hostname: Option<String> = client.get(retry::Raw, url_for_key("meta-data/hostname")).send()?;
    let availability_zone: Option<String> = client.get(retry::Raw, url_for_key("meta-data/placement/availability-zone")).send()?;
    let region: Option<String> = client.get(retry::Json, url_for_key("dynamic/instance-identity/document")).send()?
        .map(|instance_id_doc: InstanceIdDoc| instance_id_doc.region);

    let ssh_keys: Vec<String> = fetch_ssh_keys(&client)?;

    Ok(Metadata::builder()
        .add_attribute_if_exists("EC2_REGION".to_owned(), region)
        .add_attribute_if_exists("EC2_INSTANCE_ID".to_owned(), instance_id)
        .add_attribute_if_exists("EC2_IPV4_PUBLIC".to_owned(), public)
        .add_attribute_if_exists("EC2_IPV4_LOCAL".to_owned(), local)
        .add_attribute_if_exists("EC2_HOSTNAME".to_owned(), hostname.clone())
        .add_attribute_if_exists("EC2_AVAILABILITY_ZONE".to_owned(), availability_zone)
        .set_hostname_if_exists(hostname)
        .add_ssh_keys(ssh_keys)?
        .build())
}

fn fetch_ssh_keys(client: &retry::Client) -> Result<Vec<String>> {
    let keydata: Option<String> = client.get(retry::Raw, url_for_key("meta-data/public-keys")).send()?;
    let mut keys = Vec::new();
    if let Some(keys_list) = keydata {
        for l in keys_list.lines() {
            let tokens: Vec<&str> = l.split('=').collect();
            if tokens.len() != 2 {
                return Err("error parsing keyID".into());
            }
            let key: String = client.get(retry::Raw, url_for_key(&format!("meta-data/public-keys/{}/openssh-key", tokens[0]))).send()?
                    .ok_or("missing ssh key")?;
            keys.push(key)
        }
    }
    Ok(keys)
}
