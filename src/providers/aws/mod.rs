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
use serde_derive::Deserialize;
use slog_scope::warn;

use crate::errors::*;
use crate::network;
use crate::providers::MetadataProvider;
use crate::retry;

#[cfg(test)]
mod mock_tests;

#[cfg(not(feature = "cl-legacy"))]
static ENV_PREFIX: &str = "AWS";
#[cfg(feature = "cl-legacy")]
static ENV_PREFIX: &str = "EC2";

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct InstanceIdDoc {
    region: String,
}

#[derive(Clone, Debug)]
pub struct AwsProvider {
    client: retry::Client,
}

impl AwsProvider {
    pub fn try_new() -> Result<AwsProvider> {
        let client = retry::Client::try_new()?.return_on_404(true);

        Ok(AwsProvider { client })
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
                AwsProvider::endpoint_for("meta-data/public-keys"),
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
                        AwsProvider::endpoint_for(&format!(
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

impl MetadataProvider for AwsProvider {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        let mut out = HashMap::with_capacity(6);

        let add_value = |map: &mut HashMap<_, _>, key: &str, name| -> Result<()> {
            let value = self
                .client
                .get(retry::Raw, AwsProvider::endpoint_for(name))
                .send()?;

            if let Some(value) = value {
                map.insert(key.to_string(), value);
            }

            Ok(())
        };

        add_value(
            &mut out,
            &format!("{}_INSTANCE_ID", ENV_PREFIX),
            "meta-data/instance-id",
        )?;
        add_value(
            &mut out,
            &format!("{}_INSTANCE_TYPE", ENV_PREFIX),
            "meta-data/instance-type",
        )?;
        add_value(
            &mut out,
            &format!("{}_IPV4_LOCAL", ENV_PREFIX),
            "meta-data/local-ipv4",
        )?;
        add_value(
            &mut out,
            &format!("{}_IPV4_PUBLIC", ENV_PREFIX),
            "meta-data/public-ipv4",
        )?;
        add_value(
            &mut out,
            &format!("{}_AVAILABILITY_ZONE", ENV_PREFIX),
            "meta-data/placement/availability-zone",
        )?;
        add_value(
            &mut out,
            &format!("{}_HOSTNAME", ENV_PREFIX),
            "meta-data/hostname",
        )?;
        add_value(
            &mut out,
            &format!("{}_PUBLIC_HOSTNAME", ENV_PREFIX),
            "meta-data/public-hostname",
        )?;

        let region = self
            .client
            .get(
                retry::Json,
                AwsProvider::endpoint_for("dynamic/instance-identity/document"),
            )
            .send()?
            .map(|instance_id_doc: InstanceIdDoc| instance_id_doc.region);
        if let Some(region) = region {
            out.insert(format!("{}_REGION", ENV_PREFIX), region);
        }

        Ok(out)
    }

    fn hostname(&self) -> Result<Option<String>> {
        self.client
            .get(retry::Raw, AwsProvider::endpoint_for("meta-data/hostname"))
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

    fn virtual_network_devices(&self) -> Result<Vec<network::VirtualNetDev>> {
        warn!("virtual network devices metadata requested, but not supported on this platform");
        Ok(vec![])
    }

    fn boot_checkin(&self) -> Result<()> {
        warn!("boot check-in requested, but not supported on this platform");
        Ok(())
    }
}
