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

//! oraclecloud provider metadata fetcher
//! This provider is selected via the platform ID `oraclecloud`.
//! The metadata endpoint is documented at
//! https://docs.oracle.com/en-us/iaas/Content/Compute/Tasks/gettingmetadata.htm.

use anyhow::{Context, Result};
use openssh_keys::PublicKey;
use reqwest::header::{HeaderName, HeaderValue};
use serde::Deserialize;
use std::collections::HashMap;

use crate::providers::MetadataProvider;
use crate::retry;

#[cfg(test)]
mod mock_tests;

const ORACLECLOUD_METADATA_BASE_URL: &str = "http://169.254.169.254/opc/v2";

#[derive(Clone, Debug)]
pub struct OracleCloudProvider {
    instance: Instance,
}

impl OracleCloudProvider {
    pub fn try_new() -> Result<OracleCloudProvider> {
        let client = retry::Client::try_new()?;
        Self::try_new_with_client(&client)
    }

    pub(crate) fn try_new_with_client(client: &retry::Client) -> Result<OracleCloudProvider> {
        let instance = OracleCloudProvider::fetch_instance_metadata(client)?;
        Ok(OracleCloudProvider { instance })
    }

    fn endpoint_for(name: &str) -> String {
        format!("{ORACLECLOUD_METADATA_BASE_URL}/{name}")
    }

    fn fetch_instance_metadata(client: &retry::Client) -> Result<Instance> {
        client
            .get(retry::Json, Self::endpoint_for("instance"))
            .header(
                HeaderName::from_static("authorization"),
                HeaderValue::from_static("Bearer Oracle"),
            )
            .send()?
            .context("fetch instance metadata")
    }

    fn parse_attrs(&self) -> Vec<(String, String)> {
        vec![
            (
                "ORACLECLOUD_AVAILABILITY_DOMAIN".to_string(),
                self.instance.availability_domain.clone(),
            ),
            (
                "ORACLECLOUD_COMPARTMENT_ID".to_string(),
                self.instance.compartment_id.clone(),
            ),
            (
                "ORACLECLOUD_FAULT_DOMAIN".to_string(),
                self.instance.fault_domain.clone(),
            ),
            (
                "ORACLECLOUD_HOSTNAME".to_string(),
                self.instance.hostname.clone(),
            ),
            (
                "ORACLECLOUD_INSTANCE_ID".to_string(),
                self.instance.id.clone(),
            ),
            (
                "ORACLECLOUD_INSTANCE_SHAPE".to_string(),
                self.instance.shape.clone(),
            ),
            (
                "ORACLECLOUD_REGION_ID".to_string(),
                self.instance.canonical_region_name.clone(),
            ),
        ]
    }
}

impl MetadataProvider for OracleCloudProvider {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        Ok(self.parse_attrs().into_iter().collect())
    }

    fn hostname(&self) -> Result<Option<String>> {
        Ok(Some(self.instance.hostname.clone()))
    }

    fn ssh_keys(&self) -> Result<Vec<PublicKey>> {
        self.instance
            .metadata
            .get("ssh_authorized_keys")
            .unwrap_or(&String::new())
            .split_terminator('\n')
            .map(PublicKey::parse)
            .collect::<Result<_, _>>()
            .map_err(anyhow::Error::from)
    }

    fn networks(&self) -> Result<Vec<crate::network::Interface>> {
        Ok(std::vec![])
    }

    fn virtual_network_devices(&self) -> Result<Vec<crate::network::VirtualNetDev>> {
        Ok(std::vec![])
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Instance {
    availability_domain: String,
    canonical_region_name: String,
    compartment_id: String,
    fault_domain: String,
    hostname: String,
    id: String,
    shape: String,
    #[serde(default)]
    metadata: HashMap<String, String>,
}
