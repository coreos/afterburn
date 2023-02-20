// Copyright 2020 CoreOS, Inc.
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

//! vultr provider metadata fetcher
//! This provider is selected via the platform ID `vultr`.
//! The metadata endpoint is documented at https://www.vultr.com/metadata/.

use anyhow::Result;
#[cfg(test)]
use mockito;
use openssh_keys::PublicKey;
use slog_scope::error;
use std::collections::HashMap;

use crate::providers::MetadataProvider;
use crate::retry;

#[cfg(test)]
mod mock_tests;

#[derive(Clone, Debug)]
pub struct VultrProvider {
    client: retry::Client,
}

impl VultrProvider {
    pub fn try_new() -> Result<VultrProvider> {
        let client = retry::Client::try_new()?.return_on_404(true);

        Ok(VultrProvider { client })
    }

    #[cfg(test)]
    fn endpoint_for(name: &str) -> String {
        let url = mockito::server_url();
        format!("{url}/{name}")
    }

    #[cfg(not(test))]
    fn endpoint_for(name: &str) -> String {
        format!("http://169.254.169.254/v1/{name}")
    }

    fn fetch_attribute(
        &self,
        map: &mut HashMap<String, String>,
        key: &str,
        endpoint: &str,
    ) -> Result<()> {
        let content: Option<String> = self
            .client
            .get(retry::Raw, Self::endpoint_for(endpoint))
            .send()?;

        if let Some(value) = content {
            if !value.is_empty() {
                map.insert(key.to_string(), value);
            }
        }

        Ok(())
    }

    fn fetch_ssh_keys(&self) -> Result<Vec<String>> {
        let entries: Option<String> = self
            .client
            .get(retry::Raw, VultrProvider::endpoint_for("public-keys"))
            .send()?;
        let keys_list = entries.unwrap_or_default();

        let mut keys = Vec::new();
        for key in keys_list.lines() {
            let key = key.to_string();
            keys.push(key)
        }

        Ok(keys)
    }

    fn fetch_hostname(&self) -> Result<Option<String>> {
        let value: Option<String> = self
            .client
            .get(retry::Raw, VultrProvider::endpoint_for("hostname"))
            .send()?;

        let hostname = value.unwrap_or_default();
        if hostname.is_empty() {
            return Ok(None);
        }

        Ok(Some(hostname))
    }
}

impl MetadataProvider for VultrProvider {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        let mut out = HashMap::with_capacity(3);

        self.fetch_attribute(&mut out, "VULTR_HOSTNAME", "hostname")?;
        self.fetch_attribute(&mut out, "VULTR_INSTANCE_ID", "instanceid")?;
        self.fetch_attribute(&mut out, "VULTR_REGION_CODE", "region/regioncode")?;

        Ok(out)
    }

    fn hostname(&self) -> Result<Option<String>> {
        self.fetch_hostname()
    }

    fn ssh_keys(&self) -> Result<Vec<PublicKey>> {
        let keys = self.fetch_ssh_keys()?;
        let mut out = Vec::with_capacity(keys.len());
        for key in keys {
            match PublicKey::parse(&key) {
                Ok(pk) => out.push(pk),
                Err(e) => error!("failed to parse SSH Public Key: {}", e),
            };
        }

        Ok(out)
    }
}
