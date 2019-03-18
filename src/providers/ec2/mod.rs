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

use std::collections::HashMap;

#[cfg(test)]
use mockito;
use openssh_keys::PublicKey;

use errors::*;
use network;
use providers::MetadataProvider;
use retry;

#[cfg(test)]
mod mock_tests;

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct InstanceIdDoc {
    region: String,
}

#[derive(Clone, Debug)]
pub struct Ec2Provider {
    client: retry::Client,
}

impl Ec2Provider {
    pub fn try_new() -> Result<Ec2Provider> {
        let client = retry::Client::try_new()?.return_on_404(true);

        Ok(Ec2Provider { client })
    }

    #[cfg(test)]
    fn endpoint_for(key: &str) -> String {
        let url = mockito::server_url();
        format!("{}/{}", url, key)
    }

    #[cfg(not(test))]
    fn endpoint_for(key: &str) -> String {
        const URL: &str = "http://169.254.169.254/2009-04-04";
        format!("{}/{}", URL, key)
    }

    fn fetch_ssh_keys(&self) -> Result<Vec<String>> {
        let keydata: Option<String> = self
            .client
            .get(
                retry::Raw,
                Ec2Provider::endpoint_for("meta-data/public-keys"),
            )
            .send()?;

        let mut keys = Vec::new();
        if let Some(keys_list) = keydata {
            for l in keys_list.lines() {
                let tokens: Vec<&str> = l.split('=').collect();
                if tokens.len() != 2 {
                    return Err("error parsing keyID".into());
                }
                let key: String = self
                    .client
                    .get(
                        retry::Raw,
                        Ec2Provider::endpoint_for(&format!(
                            "meta-data/public-keys/{}/openssh-key",
                            tokens[0]
                        )),
                    )
                    .send()?
                    .ok_or("missing ssh key")?;
                keys.push(key)
            }
        }
        Ok(keys)
    }
}

impl MetadataProvider for Ec2Provider {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        let mut out = HashMap::with_capacity(6);

        let add_value = |map: &mut HashMap<_, _>, key: &str, name| -> Result<()> {
            let value = self
                .client
                .get(retry::Raw, Ec2Provider::endpoint_for(name))
                .send()?;

            if let Some(value) = value {
                map.insert(key.to_string(), value);
            }

            Ok(())
        };

        add_value(&mut out, "EC2_INSTANCE_ID", "meta-data/instance-id")?;
        add_value(&mut out, "EC2_IPV4_LOCAL", "meta-data/local-ipv4")?;
        add_value(&mut out, "EC2_IPV4_PUBLIC", "meta-data/public-ipv4")?;
        add_value(
            &mut out,
            "EC2_AVAILABILITY_ZONE",
            "meta-data/placement/availability-zone",
        )?;
        add_value(&mut out, "EC2_HOSTNAME", "meta-data/hostname")?;
        add_value(&mut out, "EC2_PUBLIC_HOSTNAME", "meta-data/public-hostname")?;

        let region = self
            .client
            .get(
                retry::Json,
                Ec2Provider::endpoint_for("dynamic/instance-identity/document"),
            )
            .send()?
            .map(|instance_id_doc: InstanceIdDoc| instance_id_doc.region);
        if let Some(region) = region {
            out.insert("EC2_REGION".to_string(), region);
        }

        Ok(out)
    }

    fn hostname(&self) -> Result<Option<String>> {
        self.client
            .get(retry::Raw, Ec2Provider::endpoint_for("meta-data/hostname"))
            .send()
    }

    fn ssh_keys(&self) -> Result<Vec<PublicKey>> {
        self.fetch_ssh_keys().map(|keys| {
            keys.into_iter()
                .map(|key| {
                    let key = PublicKey::parse(&key)?;
                    Ok(key)
                })
                .collect::<Result<Vec<_>>>()
        })?
    }

    fn networks(&self) -> Result<Vec<network::Interface>> {
        Ok(vec![])
    }

    fn network_devices(&self) -> Result<Vec<network::Device>> {
        Ok(vec![])
    }

    fn boot_checkin(&self) -> Result<()> {
        warn!("boot check-in requested, but not supported on this platform");
        Ok(())
    }
}
