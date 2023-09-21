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

//! Metadata fetcher for Scaleway.
//!
//! The metadata API specification follows the instance one described
//! [their docs](https://www.scaleway.com/en/developers/api/instance/#path-instances-get-an-instance)
//!
//! An implementation for the metadata retrival and boot check-in can be found
//! in the image-tools
//! [`scw-metadata-json`](https://github.com/scaleway/image-tools/blob/cloud-init-18.3%2B24.gf6249277/bases/overlay-common/usr/local/bin/scw-metadata-json)
//! and
//! [`scw-signal-state`](https://github.com/scaleway/image-tools/blob/cloud-init-18.3%2B24.gf6249277/bases/overlay-common/usr/local/sbin/scw-signal-state)

use std::collections::HashMap;

use anyhow::{anyhow, Result};
use openssh_keys::PublicKey;
use serde::Deserialize;

use crate::providers::MetadataProvider;
use crate::retry;

#[cfg(test)]
mod mock_tests;

#[derive(Clone, Deserialize)]
struct ScalewaySSHPublicKey {
    key: String,
}

#[derive(Clone, Deserialize)]
struct ScalwayInterfaces {
    private_ip: Option<String>,
    public_ip: Option<ScalewayIPv4Public>,
    ipv6: Option<ScalewayIPv6Public>,
}

#[derive(Clone, Deserialize)]
struct ScalewayIPv4Public {
    address: String,
}

#[derive(Clone, Deserialize)]
struct ScalewayIPv6Public {
    address: String,
}

#[derive(Clone, Deserialize)]
struct ScalewayLocation {
    zone_id: String,
}

#[derive(Clone, Deserialize)]
struct ScalewayInstanceMetadata {
    commercial_type: String,
    hostname: String,
    id: String,
    #[serde(flatten)]
    interfaces: ScalwayInterfaces,
    location: ScalewayLocation,
    ssh_public_keys: Vec<ScalewaySSHPublicKey>,
}

pub struct ScalewayProvider {
    client: retry::Client,
}

impl ScalewayProvider {
    pub fn try_new() -> Result<ScalewayProvider> {
        let client = retry::Client::try_new()?;
        Ok(ScalewayProvider { client })
    }

    fn fetch_metadata(&self) -> Result<ScalewayInstanceMetadata> {
        let data: ScalewayInstanceMetadata = self
            .client
            .get(
                retry::Json,
                "http://169.254.42.42/conf?format=json".to_string(),
            )
            .send()?
            .ok_or_else(|| anyhow!("not found"))?;

        Ok(data)
    }

    fn parse_attrs(&self) -> Result<Vec<(String, String)>> {
        let data = self.fetch_metadata()?;

        let instance_type = data.commercial_type;
        let zone_id = data.location.zone_id;

        let mut attrs = vec![
            ("SCALEWAY_HOSTNAME".to_string(), data.hostname.clone()),
            ("SCALEWAY_INSTANCE_ID".to_string(), data.id.clone()),
            ("SCALEWAY_INSTANCE_TYPE".to_string(), instance_type.clone()),
            ("SCALEWAY_ZONE_ID".to_string(), zone_id.clone()),
        ];

        if let Some(ref ip) = data.interfaces.private_ip {
            attrs.push(("SCALEWAY_IPV4_PRIVATE".to_string(), ip.clone()));
        }

        if let Some(ref ip) = data.interfaces.public_ip {
            attrs.push(("SCALEWAY_IPV4_PUBLIC".to_string(), ip.address.clone()));
        }

        if let Some(ref ip) = data.interfaces.ipv6 {
            attrs.push(("SCALEWAY_IPV6_PUBLIC".to_string(), ip.address.clone()));
        }

        Ok(attrs)
    }
}

impl MetadataProvider for ScalewayProvider {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        let attrs = self.parse_attrs()?;
        Ok(attrs.into_iter().collect())
    }

    fn boot_checkin(&self) -> Result<()> {
        self.client
            .patch(
                retry::Json,
                "http://169.254.42.42/state".to_string(),
                Some(r#"{"state_detail":"booted"}"#.into()),
            )
            .dispatch_patch()?;
        Ok(())
    }

    fn hostname(&self) -> Result<Option<String>> {
        let data = self.fetch_metadata()?;
        Ok(Some(data.hostname.clone()))
    }

    fn ssh_keys(&self) -> Result<Vec<PublicKey>> {
        let mut out = Vec::new();

        let data = self.fetch_metadata()?;

        for key in data.ssh_public_keys {
            let key = PublicKey::parse(&key.key)?;
            out.push(key);
        }

        Ok(out)
    }
}
