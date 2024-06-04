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
    fn attributes(&self) -> Result<std::collections::HashMap<String, String>> {
        let meta: HetznerMetadata = self
            .client
            .get(retry::Yaml, HETZNER_METADATA_BASE_URL.to_string())
            .send()?
            .unwrap();

        Ok(meta.into())
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
#[serde(rename_all = "kebab-case")]
struct HetznerMetadata {
    hostname: Option<String>,
    instance_id: Option<i64>,
    public_ipv4: Option<String>,
    availability_zone: Option<String>,
    region: Option<String>,
}

impl From<HetznerMetadata> for HashMap<String, String> {
    fn from(meta: HetznerMetadata) -> Self {
        let mut out = HashMap::with_capacity(5);

        let add_value = |map: &mut HashMap<_, _>, key: &str, value: Option<String>| {
            if let Some(value) = value {
                map.insert(key.to_string(), value);
            }
        };

        add_value(
            &mut out,
            "HETZNER_AVAILABILITY_ZONE",
            meta.availability_zone,
        );
        add_value(&mut out, "HETZNER_HOSTNAME", meta.hostname);
        add_value(
            &mut out,
            "HETZNER_INSTANCE_ID",
            meta.instance_id.map(|i| i.to_string()),
        );
        add_value(&mut out, "HETZNER_PUBLIC_IPV4", meta.public_ipv4);
        add_value(&mut out, "HETZNER_REGION", meta.region);

        out
    }
}

#[cfg(test)]
mod tests {
    use super::HetznerMetadata;

    #[test]
    fn test_metadata_deserialize() {
        let body = r#"availability-zone: hel1-dc2
hostname: my-server
instance-id: 42
public-ipv4: 1.2.3.4
region: eu-central
public-keys: []"#;

        let meta: HetznerMetadata = serde_yaml::from_str(body).unwrap();

        assert_eq!(meta.availability_zone.unwrap(), "hel1-dc2");
        assert_eq!(meta.hostname.unwrap(), "my-server");
        assert_eq!(meta.instance_id.unwrap(), 42);
        assert_eq!(meta.public_ipv4.unwrap(), "1.2.3.4");
    }
}
