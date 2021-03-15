// Copyright 2017 CoreOS, Inc.
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

//! aws ec2 metadata fetcher
//!

use std::collections::HashMap;

#[cfg(test)]
use mockito;
use openssh_keys::PublicKey;
use reqwest::header;
use serde_derive::Deserialize;
use slog_scope::warn;

use crate::errors::*;
use crate::providers::MetadataProvider;
use crate::retry;

#[cfg(test)]
mod mock_tests;

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct InstanceIdDoc {
    region: String,
}

#[derive(Clone, Debug)]
pub struct AwsProvider {
    client: retry::Client,
}

impl AwsProvider {
    pub fn try_new() -> Result<AwsProvider> {
        let client = retry::Client::try_new()?.return_on_404(true);
        AwsProvider::with_client(client)
    }

    fn with_client(client: retry::Client) -> Result<AwsProvider> {
        let mut client = client;
        let token = AwsProvider::fetch_imdsv2_token(client.clone());

        // If IMDSv2 token is fetched successfully, set the header.
        // Otherwise, proceed with IMDSv1 mechanism.
        match token {
            Ok(t) => {
                client = client.header(
                    header::HeaderName::from_bytes(b"X-aws-ec2-metadata-token")
                        .chain_err(|| "setting header name for aws imdsv2 metadata")?,
                    header::HeaderValue::from_bytes(t.as_bytes())
                        .chain_err(|| "setting header value for aws imdsv2 metadata")?,
                );
            }
            Err(err) => {
                warn!("failed to fetch aws imdsv2 session token with: {}", err);
            }
        }

        Ok(AwsProvider { client })
    }

    #[cfg(test)]
    fn endpoint_for(key: &str, _use_latest: bool) -> String {
        let url = mockito::server_url();
        format!("{}/{}", url, key)
    }

    #[cfg(not(test))]
    fn endpoint_for(key: &str, use_latest: bool) -> String {
        const URL: &str = "http://169.254.169.254/2019-10-01";
        const URL_LATEST: &str = "http://169.254.169.254/latest";
        if use_latest {
            format!("{}/{}", URL_LATEST, key)
        } else {
            format!("{}/{}", URL, key)
        }
    }

    fn fetch_imdsv2_token(client: retry::Client) -> Result<String> {
        let token: String = client
            .header(
                header::HeaderName::from_bytes(b"X-aws-ec2-metadata-token-ttl-seconds")
                    .chain_err(|| "setting header name for aws imdsv2 token")?,
                header::HeaderValue::from_bytes(b"21600")
                    .chain_err(|| "setting header value for aws imdsv2 token")?,
            )
            .put(
                retry::Raw,
                // NOTE(zonggen): Use `latest` here since other versions would return "403 - Forbidden"
                AwsProvider::endpoint_for("api/token", true),
                None,
            )
            .dispatch_put()?
            .chain_err(|| "unwrapping aws imdsv2 token")?;
        Ok(token)
    }

    fn fetch_ssh_keys(&self) -> Result<Vec<String>> {
        let keydata: Option<String> = self
            .client
            .get(
                retry::Raw,
                AwsProvider::endpoint_for("meta-data/public-keys", false),
            )
            .send()?;

        let mut keys = Vec::new();
        if let Some(keys_list) = keydata {
            for l in keys_list.lines() {
                let tokens: Vec<&str> = l.split('=').collect();
                if tokens.len() != 2 {
                    return Err("error parsing keyID".into());
                }
                let key: String = self
                    .client
                    .get(
                        retry::Raw,
                        AwsProvider::endpoint_for(
                            &format!("meta-data/public-keys/{}/openssh-key", tokens[0]),
                            false,
                        ),
                    )
                    .send()?
                    .ok_or("missing ssh key")?;
                keys.push(key)
            }
        }
        Ok(keys)
    }
}

impl MetadataProvider for AwsProvider {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        let mut out = HashMap::with_capacity(6);

        let add_value = |map: &mut HashMap<_, _>, key: &str, name| -> Result<()> {
            let value = self
                .client
                .get(retry::Raw, AwsProvider::endpoint_for(name, false))
                .send()?;

            if let Some(value) = value {
                map.insert(key.to_string(), value);
            }

            Ok(())
        };

        add_value(&mut out, "AWS_INSTANCE_ID", "meta-data/instance-id")?;
        add_value(&mut out, "AWS_INSTANCE_TYPE", "meta-data/instance-type")?;
        add_value(&mut out, "AWS_IPV4_LOCAL", "meta-data/local-ipv4")?;
        add_value(&mut out, "AWS_IPV4_PUBLIC", "meta-data/public-ipv4")?;
        add_value(
            &mut out,
            "AWS_AVAILABILITY_ZONE",
            "meta-data/placement/availability-zone",
        )?;
        add_value(&mut out, "AWS_HOSTNAME", "meta-data/hostname")?;
        add_value(&mut out, "AWS_PUBLIC_HOSTNAME", "meta-data/public-hostname")?;

        let region = self
            .client
            .get(
                retry::Json,
                AwsProvider::endpoint_for("dynamic/instance-identity/document", false),
            )
            .send()?
            .map(|instance_id_doc: InstanceIdDoc| instance_id_doc.region);
        if let Some(region) = region {
            out.insert("AWS_REGION".to_string(), region);
        }

        Ok(out)
    }

    fn hostname(&self) -> Result<Option<String>> {
        self.client
            .get(
                retry::Raw,
                AwsProvider::endpoint_for("meta-data/hostname", false),
            )
            .send()
    }

    fn ssh_keys(&self) -> Result<Vec<PublicKey>> {
        self.fetch_ssh_keys().map(|keys| {
            keys.into_iter()
                .map(|key| {
                    let key = PublicKey::parse(&key)?;
                    Ok(key)
                })
                .collect::<Result<Vec<_>>>()
        })?
    }
}
