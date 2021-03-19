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

//! metadata fetcher for the exoscale provider
//! https://cloudinit.readthedocs.io/en/latest/topics/datasources/exoscale.html#crawling-of-metadata
//! https://community.exoscale.com/documentation/compute/cloud-init/#querying-the-user-data-and-meta-data-from-the-instance

use std::collections::HashMap;

use anyhow::Result;
use openssh_keys::PublicKey;

use crate::providers::MetadataProvider;
use crate::retry;

#[cfg(test)]
mod mock_tests;

#[derive(Clone, Debug)]
pub struct ExoscaleProvider {
    client: retry::Client,
}

impl ExoscaleProvider {
    pub fn try_new() -> Result<ExoscaleProvider> {
        let client = retry::Client::try_new()?;

        Ok(ExoscaleProvider { client })
    }

    #[cfg(test)]
    fn endpoint_for(&self, key: &str) -> String {
        let url = mockito::server_url();
        format!("{}/{}", url, key)
    }

    #[cfg(not(test))]
    fn endpoint_for(&self, key: &str) -> String {
        format!("http://169.254.169.254/1.0/meta-data/{}", key)
    }
}

impl MetadataProvider for ExoscaleProvider {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        let mut out = HashMap::with_capacity(9);
        let add_value = |map: &mut HashMap<_, _>, key: &str, name| -> Result<()> {
            let value = self
                .client
                .get(retry::Raw, self.endpoint_for(name))
                .send()?;

            if let Some(value) = value {
                map.insert(key.to_string(), value);
            }

            Ok(())
        };

        add_value(&mut out, "EXOSCALE_INSTANCE_ID", "instance-id")?;
        add_value(&mut out, "EXOSCALE_LOCAL_HOSTNAME", "local-hostname")?;
        add_value(&mut out, "EXOSCALE_PUBLIC_HOSTNAME", "public-hostname")?;
        add_value(&mut out, "EXOSCALE_AVAILABILITY_ZONE", "availability-zone")?;
        add_value(&mut out, "EXOSCALE_PUBLIC_IPV4", "public-ipv4")?;
        add_value(&mut out, "EXOSCALE_LOCAL_IPV4", "local-ipv4")?;
        add_value(&mut out, "EXOSCALE_SERVICE_OFFERING", "service-offering")?;
        add_value(&mut out, "EXOSCALE_CLOUD_IDENTIFIER", "cloud-identifier")?;
        add_value(&mut out, "EXOSCALE_VM_ID", "vm-id")?;

        Ok(out)
    }

    fn hostname(&self) -> Result<Option<String>> {
        let value: Option<String> = self
            .client
            .get(retry::Raw, self.endpoint_for("local-hostname"))
            .send()?;

        let hostname = value.unwrap_or_default();
        if hostname.is_empty() {
            return Ok(None);
        }

        Ok(Some(hostname))
    }

    fn ssh_keys(&self) -> Result<Vec<PublicKey>> {
        let keys: Option<String> = self
            .client
            .get(retry::Raw, self.endpoint_for("public-keys"))
            .send()?;

        Ok(keys
            .map(|s| PublicKey::read_keys(s.as_bytes()))
            .unwrap_or_else(|| Ok(vec![]))?)
    }
}
