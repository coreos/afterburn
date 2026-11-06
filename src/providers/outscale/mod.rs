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

//! Outscale provider metadata fetcher
//! This provider is selected via the platform ID `outscale`.
//! The metadata endpoint is documented at https://docs.outscale.com/en/userguide/Accessing-the-Metadata-and-User-Data-of-a-VM.html.

use anyhow::Result;
use openssh_keys::PublicKey;
use std::collections::HashMap;

use crate::providers::MetadataProvider;
use crate::retry;

#[cfg(test)]
mod mock_tests;

#[derive(Clone, Debug)]
pub struct OutscaleProvider {
    client: retry::Client,
}

impl OutscaleProvider {
    pub fn try_new() -> Result<OutscaleProvider> {
        let client = retry::Client::try_new()?.return_on_404(true);

        Ok(OutscaleProvider { client })
    }

    fn endpoint_for(name: &str) -> String {
        format!("http://169.254.169.254/latest/meta-data/{name}")
    }

    fn fetch_attribute(&self, endpoint: &str) -> Result<Option<String>> {
        let content: Option<String> = self
            .client
            .get(retry::Raw, Self::endpoint_for(endpoint))
            .send()?;

        Ok(content)
    }
}

impl MetadataProvider for OutscaleProvider {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        let attributes = [
            ("OUTSCALE_INSTANCE_ID", "instance-id"),
            ("OUTSCALE_AMI_ID", "ami-id"),
            ("OUTSCALE_INSTANCE_TYPE", "instance-type"),
            ("OUTSCALE_AVAILABILITY_ZONE", "placement/availability-zone"),
            ("OUTSCALE_LOCAL_IPV4", "local-ipv4"),
            ("OUTSCALE_RESERVATION_ID", "reservation-id"),
            ("OUTSCALE_HOSTNAME", "hostname"),
        ];
        let mut out = HashMap::with_capacity(attributes.len());

        for (key, endpoint) in attributes {
            if let Some(value) = self.fetch_attribute(endpoint)? {
                out.insert(key.to_string(), value);
            }
        }

        Ok(out)
    }

    fn hostname(&self) -> Result<Option<String>> {
        Ok(self
            .fetch_attribute("hostname")?
            .filter(|value| !value.is_empty()))
    }

    fn ssh_keys(&self) -> Result<Vec<PublicKey>> {
        let Some(keys) = self.fetch_attribute("public-keys/0/openssh-key")? else {
            return Ok(Vec::new());
        };

        let keys = keys
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(PublicKey::parse)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(keys)
    }
}
