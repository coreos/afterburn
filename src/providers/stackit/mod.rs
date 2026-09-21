// Copyright 2026 CoreOS, Inc.
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

//! Metadata fetcher for STACKIT, selected via the platform ID `stackit`.
//!
//! STACKIT exposes OpenStack-compatible instance metadata, also used by its
//! cloud controller:
//! https://github.com/stackitcloud/cloud-provider-stackit/blob/main/pkg/stackit/metadata/metadata.go
//! The metadata format is documented at
//! https://docs.openstack.org/nova/latest/user/metadata.html.

use std::collections::{BTreeMap, HashMap};

use anyhow::{ensure, Context, Result};
use openssh_keys::PublicKey;
use serde::Deserialize;

use crate::providers::MetadataProvider;
use crate::retry;

#[cfg(test)]
mod mock_tests;

const METADATA_URL: &str = "http://169.254.169.254/openstack/latest/meta_data.json";

#[derive(Debug)]
pub struct StackitProvider {
    client: retry::Client,
}

impl StackitProvider {
    pub fn try_new() -> Result<Self> {
        let client = retry::Client::try_new()?.no_proxy()?;
        Ok(Self { client })
    }

    fn fetch_metadata(&self) -> Result<Metadata> {
        let metadata: Metadata = self
            .client
            .get(retry::Json, METADATA_URL.to_owned())
            .send()
            .context("failed to fetch STACKIT metadata")?
            .context("missing STACKIT metadata")?;
        ensure!(!metadata.uuid.is_empty(), "missing STACKIT instance UUID");
        Ok(metadata)
    }
}

impl MetadataProvider for StackitProvider {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        let metadata = self.fetch_metadata()?;
        let mut attributes = HashMap::from([("STACKIT_INSTANCE_ID".to_owned(), metadata.uuid)]);

        for (key, value) in [
            ("STACKIT_HOSTNAME", metadata.hostname),
            ("STACKIT_AVAILABILITY_ZONE", metadata.availability_zone),
        ] {
            if let Some(value) = value.filter(|value| !value.is_empty()) {
                attributes.insert(key.to_owned(), value);
            }
        }

        Ok(attributes)
    }

    fn hostname(&self) -> Result<Option<String>> {
        Ok(self
            .fetch_metadata()?
            .hostname
            .filter(|hostname| !hostname.is_empty()))
    }

    fn ssh_keys(&self) -> Result<Vec<PublicKey>> {
        self.fetch_metadata()?
            .public_keys
            .unwrap_or_default()
            .values()
            .map(|key| PublicKey::parse(key).context("failed to parse STACKIT SSH public key"))
            .collect()
    }
}

#[derive(Debug, Deserialize)]
struct Metadata {
    uuid: String,
    hostname: Option<String>,
    availability_zone: Option<String>,
    public_keys: Option<BTreeMap<String, String>>,
}
