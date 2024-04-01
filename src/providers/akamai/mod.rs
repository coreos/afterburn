// Copyright 2024 CoreOS, Inc.
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

//! Metadata fetcher for Akamai Connected Cloud (Linode).
//!
//! The Metadata Service's API specification is described in [Guides - Overview of the Metadata
//! Service](https://www.linode.com/docs/products/compute/compute-instances/guides/metadata/).

#[cfg(test)]
mod mock_tests;

use anyhow::{Context, Result};
use openssh_keys::PublicKey;
use reqwest::header::{HeaderName, HeaderValue};
use serde::Deserialize;
use std::collections::HashMap;

use crate::providers::MetadataProvider;
use crate::retry;

/// Default TTL for the metadata token, in seconds.
static TOKEN_TTL: &str = "300";

pub struct AkamaiProvider {
    client: retry::Client,
}

impl AkamaiProvider {
    /// Instantiate a new `AkamaiProvider`.
    pub fn try_new() -> Result<Self> {
        // Get a metadata token.
        let client = retry::Client::try_new()?;
        let token = get_token(client)?;

        // Create the new client with the token pre-loaded into a header.
        // All of the other endpoints accept "text/plain" and "application/json".
        // Let's prefer JSON.
        let client = retry::Client::try_new()?
            .header(
                HeaderName::from_static("metadata-token"),
                HeaderValue::from_str(&token)?,
            )
            .header(
                HeaderName::from_static("accept"),
                HeaderValue::from_static("application/json"),
            )
            .return_on_404(true);
        Ok(Self { client })
    }

    /// Instantiate a new `AkamaiProvider` with a specific client.
    ///
    /// NOTE: This method solely exists for testing.
    #[cfg(test)]
    pub fn with_base_url(url: String) -> Result<Self> {
        let client = retry::Client::try_new()?
            .mock_base_url(url.clone())
            .return_on_404(true)
            .max_retries(0);
        let token = get_token(client)?;

        let client = retry::Client::try_new()?
            .header(
                HeaderName::from_static("metadata-token"),
                HeaderValue::from_str(&token)?,
            )
            .header(
                HeaderName::from_static("accept"),
                HeaderValue::from_static("application/json"),
            )
            .mock_base_url(url)
            .return_on_404(true)
            .max_retries(0);
        Ok(Self { client })
    }

    fn endpoint_for(key: &str) -> String {
        const URL: &str = "http://169.254.169.254/v1";
        format!("{URL}/{key}")
    }

    /// Fetch the instance metadata.
    fn fetch_instance_metadata(&self) -> Result<Instance> {
        let instance: Instance = self
            .client
            .get(retry::Json, AkamaiProvider::endpoint_for("instance"))
            .send()?
            .context("get instance metadata")?;
        Ok(instance)
    }

    /// Fetch the network metadata.
    fn fetch_network_metadata(&self) -> Result<Network> {
        let network: Network = self
            .client
            .get(retry::Json, AkamaiProvider::endpoint_for("network"))
            .send()?
            .context("get network metadata")?;
        Ok(network)
    }

    /// Fetch the SSH keys.
    /// The returned [HashMap] is a mapping of usernames, to SSH public keys.
    fn fetch_ssh_keys(&self) -> Result<HashMap<String, Vec<String>>> {
        let ssh_keys: SshKeys = self
            .client
            .get(retry::Json, AkamaiProvider::endpoint_for("ssh-keys"))
            .send()?
            .context("get ssh keys")?;
        Ok(ssh_keys.users)
    }

    /// Convert instance and network metadata into environment variables.
    fn parse_attrs(&self) -> Result<Vec<(String, String)>> {
        // Instance metadata.
        let data = self.fetch_instance_metadata()?;
        let mut attrs = vec![
            ("AKAMAI_INSTANCE_ID".to_string(), data.id.to_string()),
            (
                "AKAMAI_INSTANCE_HOST_UUID".to_string(),
                data.host_uuid.clone(),
            ),
            ("AKAMAI_INSTANCE_LABEL".to_string(), data.label.clone()),
            ("AKAMAI_INSTANCE_REGION".to_string(), data.region.clone()),
            ("AKAMAI_INSTANCE_TYPE".to_string(), data.r#type.clone()),
            ("AKAMAI_INSTANCE_TAGS".to_string(), data.tags.join(":")),
        ];

        // Network metadata.
        let data = self.fetch_network_metadata()?;

        // IPv4
        for (i, addr) in data.ipv4.public.iter().enumerate() {
            attrs.push((format!("AKAMAI_PUBLIC_IPV4_{i}"), addr.to_string()));
        }

        for (i, addr) in data.ipv4.private.iter().enumerate() {
            attrs.push((format!("AKAMAI_PRIVATE_IPV4_{i}"), addr.to_string()));
        }

        for (i, addr) in data.ipv4.shared.iter().enumerate() {
            attrs.push((format!("AKAMAI_SHARED_IPV4_{i}"), addr.to_string()));
        }

        // IPv6
        attrs.push(("AKAMAI_IPV6_SLAAC".to_string(), data.ipv6.slaac.clone()));
        attrs.push((
            "AKAMAI_IPV6_LINK_LOCAL".to_string(),
            data.ipv6.link_local.clone(),
        ));
        for (i, v) in data.ipv6.ranges.iter().enumerate() {
            attrs.push((format!("AKAMAI_IPV6_RANGE_{i}"), v.to_string()));
        }
        for (i, v) in data.ipv6.shared_ranges.iter().enumerate() {
            attrs.push((format!("AKAMAI_IPV6_SHARED_RANGE_{i}"), v.to_string()));
        }

        Ok(attrs)
    }
}

// Retrieve a token we can use to authenticate future requests to the Linode Metadata Service.
fn get_token(client: retry::Client) -> Result<String> {
    let token: String = client
        .header(
            HeaderName::from_static("metadata-token-expiry-seconds"),
            HeaderValue::from_static(TOKEN_TTL),
        )
        .put(retry::Raw, AkamaiProvider::endpoint_for("token"), None)
        .dispatch_put()?
        .context("get metadata token")?;
    Ok(token)
}

impl MetadataProvider for AkamaiProvider {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        let attrs = self.parse_attrs()?;
        Ok(attrs.into_iter().collect())
    }

    fn ssh_keys(&self) -> Result<Vec<PublicKey>> {
        Ok(self
            .fetch_ssh_keys()?
            .values()
            .flatten()
            .map(|k| PublicKey::parse(k))
            .collect::<Result<_, _>>()?)
    }
}

#[derive(Clone, Deserialize)]
struct Instance {
    id: i64,
    host_uuid: String,
    label: String,
    region: String,
    r#type: String,
    tags: Vec<String>,
    #[allow(dead_code)]
    specs: Specs,
    #[allow(dead_code)]
    backups: Backups,
}

#[allow(dead_code)]
#[derive(Clone, Deserialize)]
struct Specs {
    // Total number of virtual CPU cores on the instance.
    // Currently, the largest offering is 64 vCPUs on a `g6-dedicated-64` instance type.
    vcpus: u8,

    // Total amount of instance memory, in MB (not MiB).
    memory: u64,

    // Total amount of local disk, in MB.
    //
    // NOTE: This is a strange number. For example, an instance with 25GB of disk has a reported
    // size of `25600`.
    disk: u64,

    // The monthly network transfer limit for the instance, in GB (not GiB).
    // For a 1TB monthly transfer limit, this value would be `1000`.
    transfer: u64,

    // Total number of available GPUs.
    gpus: u8,
}

#[allow(dead_code)]
#[derive(Clone, Deserialize)]
struct Backups {
    enabled: bool,
    status: Option<String>, // pending, running, complete
}

#[derive(Clone, Deserialize)]
struct Network {
    #[allow(dead_code)]
    interfaces: Vec<NetworkInterface>,
    ipv4: Ipv4,
    ipv6: Ipv6,
}

#[allow(dead_code)]
#[derive(Clone, Deserialize)]
struct NetworkInterface {
    id: u64,
    purpose: Option<String>, // public, vlan
    label: Option<String>,
    ipam_address: Option<String>,
}

#[derive(Clone, Deserialize)]
struct Ipv4 {
    public: Vec<String>,
    private: Vec<String>,
    shared: Vec<String>,
}

#[derive(Clone, Deserialize)]
struct Ipv6 {
    slaac: String,              // undocumented
    ranges: Vec<String>,        // ???
    link_local: String,         // snake_case is correct, documentation is wrong
    shared_ranges: Vec<String>, // undocumented, might be "elastic-ranges" in the doc
}

/// Used for deserializing a JSON response from the /v1/ssh-keys endpoint.
#[derive(Clone, Deserialize)]
struct SshKeys {
    // Mapping of user names, to a list of public keys.
    users: HashMap<String, Vec<String>>,
}
