// Copyright 2023 CoreOS, Inc.
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

//! Metadata fetcher for the hetzner provider
//! https://docs.hetzner.cloud/#server-metadata

use std::collections::HashMap;

use anyhow::Result;
use openssh_keys::PublicKey;
use serde::Deserialize;

use crate::retry;

use super::MetadataProvider;

#[cfg(test)]
mod mock_tests;

const HETZNER_METADATA_BASE_URL: &str = "http://169.254.169.254/hetzner/v1/metadata";

/// Metadata provider for Hetzner Cloud
///
/// See: https://docs.hetzner.cloud/#server-metadata
#[derive(Clone, Debug)]
pub struct HetznerProvider {
    client: retry::Client,
}

impl HetznerProvider {
    pub fn try_new() -> Result<Self> {
        let client = retry::Client::try_new()?;
        Ok(Self { client })
    }

    fn endpoint_for(key: &str) -> String {
        format!("{HETZNER_METADATA_BASE_URL}/{key}")
    }
}

impl MetadataProvider for HetznerProvider {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        let metadata: Metadata = self
            .client
            .get(retry::Yaml, HETZNER_METADATA_BASE_URL.to_string())
            .send()?
            .unwrap();

        let private_networks: Vec<PrivateNetwork> = self
            .client
            .get(retry::Yaml, Self::endpoint_for("private-networks"))
            .send()?
            .unwrap();

        Ok(Attributes {
            metadata,
            private_networks,
        }
        .into())
    }

    fn hostname(&self) -> Result<Option<String>> {
        let hostname: String = self
            .client
            .get(retry::Raw, Self::endpoint_for("hostname"))
            .send()?
            .unwrap_or_default();

        if hostname.is_empty() {
            return Ok(None);
        }

        Ok(Some(hostname))
    }

    fn ssh_keys(&self) -> Result<Vec<PublicKey>> {
        let keys: Vec<String> = self
            .client
            .get(retry::Json, Self::endpoint_for("public-keys"))
            .send()?
            .unwrap_or_default();

        let keys = keys
            .iter()
            .map(|s| PublicKey::parse(s))
            .collect::<Result<_, _>>()?;

        Ok(keys)
    }
}

#[derive(Debug, Deserialize)]
struct PrivateNetwork {
    ip: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Metadata {
    hostname: Option<String>,
    instance_id: Option<i64>,
    public_ipv4: Option<String>,
    availability_zone: Option<String>,
    region: Option<String>,
}

struct Attributes {
    metadata: Metadata,
    private_networks: Vec<PrivateNetwork>,
}

impl From<Attributes> for HashMap<String, String> {
    fn from(attributes: Attributes) -> Self {
        let mut out = HashMap::with_capacity(5);

        let add_value = |map: &mut HashMap<_, _>, key: &str, value: Option<String>| {
            if let Some(value) = value {
                map.insert(key.to_string(), value);
            }
        };

        add_value(
            &mut out,
            "HETZNER_AVAILABILITY_ZONE",
            attributes.metadata.availability_zone,
        );
        add_value(&mut out, "HETZNER_HOSTNAME", attributes.metadata.hostname);
        add_value(
            &mut out,
            "HETZNER_INSTANCE_ID",
            attributes.metadata.instance_id.map(|i| i.to_string()),
        );
        add_value(
            &mut out,
            "HETZNER_PUBLIC_IPV4",
            attributes.metadata.public_ipv4,
        );
        add_value(&mut out, "HETZNER_REGION", attributes.metadata.region);

        for (i, a) in attributes.private_networks.iter().enumerate() {
            add_value(
                &mut out,
                format!("HETZNER_PRIVATE_IPV4_{i}").as_str(),
                a.ip.clone(),
            );
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::{Metadata, PrivateNetwork};

    #[test]
    fn test_metadata_deserialize() {
        let body = r#"availability-zone: hel1-dc2
hostname: my-server
instance-id: 42
public-ipv4: 1.2.3.4
region: eu-central
public-keys: []"#;

        let meta: Metadata = serde_yaml::from_str(body).unwrap();

        assert_eq!(meta.availability_zone.unwrap(), "hel1-dc2");
        assert_eq!(meta.hostname.unwrap(), "my-server");
        assert_eq!(meta.instance_id.unwrap(), 42);
        assert_eq!(meta.public_ipv4.unwrap(), "1.2.3.4");
    }

    #[test]
    fn test_private_networks_deserialize() {
        let body = r"- ip: 10.0.0.2
  alias_ips: []
  interface_num: 2
  mac_address: 86:00:00:98:40:6e
  network_id: 4124728
  network_name: foo
  network: 10.0.0.0/16
  subnet: 10.0.0.0/24
  gateway: 10.0.0.1
- ip: 10.128.0.2
  alias_ips: []
  interface_num: 1
  mac_address: 86:00:00:98:40:6d
  network_id: 4451335
  network_name: bar
  network: 10.128.0.0/16
  subnet: 10.128.0.0/16
  gateway: 10.128.0.1";

        let private_networks: Vec<PrivateNetwork> = serde_yaml::from_str(body).unwrap();

        assert_eq!(private_networks.len(), 2);
        assert_eq!(private_networks[0].ip.clone().unwrap(), "10.0.0.2");
        assert_eq!(private_networks[1].ip.clone().unwrap(), "10.128.0.2");
    }
}
