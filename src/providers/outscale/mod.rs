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

//! Metadata fetcher for the OUTSCALE provider.
//!
//! OUTSCALE exposes an EC2-compatible metadata service at:
//! http://169.254.169.254/latest/meta-data/

use std::collections::HashMap;

use anyhow::Result;
use openssh_keys::PublicKey;

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

    fn endpoint_for(&self, key: &str) -> String {
        format!("http://169.254.169.254/latest/meta-data/{key}")
    }
}

impl MetadataProvider for OutscaleProvider {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        let mut out = HashMap::with_capacity(7);

        let add_value = |map: &mut HashMap<_, _>, key: &str, name: &str| -> Result<()> {
            let value = self
                .client
                .get(retry::Raw, self.endpoint_for(name))
                .send()?;

            if let Some(value) = value {
                map.insert(key.to_string(), value);
            }

            Ok(())
        };

        add_value(&mut out, "OUTSCALE_AMI_ID", "ami-id")?;
        add_value(&mut out, "OUTSCALE_INSTANCE_ID", "instance-id")?;
        add_value(&mut out, "OUTSCALE_INSTANCE_TYPE", "instance-type")?;
        add_value(&mut out, "OUTSCALE_IPV4_LOCAL", "local-ipv4")?;
        add_value(&mut out, "OUTSCALE_HOSTNAME", "hostname")?;
        add_value(&mut out, "OUTSCALE_PUBLIC_HOSTNAME", "public-hostname")?;
        add_value(
            &mut out,
            "OUTSCALE_AVAILABILITY_ZONE",
            "placement/availability-zone",
        )?;

        Ok(out)
    }

    fn hostname(&self) -> Result<Option<String>> {
        self.client
            .get(retry::Raw, self.endpoint_for("hostname"))
            .send()
    }

    fn ssh_keys(&self) -> Result<Vec<PublicKey>> {
        let key: Option<String> = self
            .client
            .get(retry::Raw, self.endpoint_for("public-keys/0/openssh-key"))
            .send()?;

        match key {
            Some(key) => Ok(vec![PublicKey::parse(&key)?]),
            None => Ok(vec![]),
        }
    }
}
