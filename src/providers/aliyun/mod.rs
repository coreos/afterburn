//! Alibaba Cloud
//!
//! This provider is selected via the platform ID `aliyun`.
//! The metadata endpoint is documented at https://www.alibabacloud.com/help/doc-detail/49122.htm.

use anyhow::{anyhow, Result};
#[cfg(test)]
use mockito;
use openssh_keys::PublicKey;
use slog_scope::error;
use std::collections::{BTreeSet, HashMap};

use crate::providers::MetadataProvider;
use crate::retry;

#[cfg(test)]
mod mock_tests;

/// Provider prefix for Alibaba Cloud.
static PROVIDER_PREFIX: &str = "ALIYUN";

#[derive(Clone, Debug)]
pub struct AliyunProvider {
    client: retry::Client,
}

impl AliyunProvider {
    pub fn try_new() -> Result<AliyunProvider> {
        let client = retry::Client::try_new()?.return_on_404(true);

        Ok(AliyunProvider { client })
    }

    #[cfg(test)]
    fn endpoint_for(name: &str) -> String {
        let url = mockito::server_url();
        format!("{url}/{name}")
    }

    #[cfg(not(test))]
    fn endpoint_for(name: &str) -> String {
        format!("http://100.100.100.200/latest/meta-data/{}", name)
    }

    /// Fetch a metadata attribute from its specific endpoint.
    ///
    /// Content (if any) is stored into the provided `map`,
    /// overwriting any previous existing value under the same `key`.
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

    /// Retrieve SSH public keys.
    ///
    /// Note: this uses a `BTreeSet` to de-duplicate redundant
    /// entries returned by the metadata API.
    fn fetch_ssh_keys(&self) -> Result<BTreeSet<String>> {
        let entries: Option<String> = self
            .client
            .get(retry::Raw, AliyunProvider::endpoint_for("public-keys/"))
            .send()?;
        let keys_list = entries.unwrap_or_default();

        let mut out = BTreeSet::new();
        for entry in keys_list.lines() {
            let key_id = entry.trim_end_matches('/');
            let ep = format!("public-keys/{}/openssh-key", key_id);
            let key: String = self
                .client
                .get(retry::Raw, AliyunProvider::endpoint_for(&ep))
                .send()?
                .ok_or_else(|| anyhow!("missing ssh key"))?;
            out.insert(key);
        }

        Ok(out)
    }

    /// Retrieve hostname.
    fn fetch_hostname(&self) -> Result<Option<String>> {
        let value: Option<String> = self
            .client
            .get(retry::Raw, AliyunProvider::endpoint_for("hostname"))
            .send()?;

        let hostname = value.unwrap_or_default();
        if hostname.is_empty() {
            return Ok(None);
        }

        Ok(Some(hostname))
    }
}

impl MetadataProvider for AliyunProvider {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        // See https://www.alibabacloud.com/help/doc-detail/49122.htm.
        let mut out = HashMap::with_capacity(10);

        self.fetch_attribute(&mut out, &format!("{}_EIPV4", PROVIDER_PREFIX), "eipv4")?;
        self.fetch_attribute(
            &mut out,
            &format!("{}_HOSTNAME", PROVIDER_PREFIX),
            "hostname",
        )?;
        self.fetch_attribute(
            &mut out,
            &format!("{}_IMAGE_ID", PROVIDER_PREFIX),
            "image-id",
        )?;
        self.fetch_attribute(
            &mut out,
            &format!("{}_INSTANCE_ID", PROVIDER_PREFIX),
            "instance-id",
        )?;
        self.fetch_attribute(
            &mut out,
            &format!("{}_INSTANCE_TYPE", PROVIDER_PREFIX),
            "instance/instance-type",
        )?;
        self.fetch_attribute(
            &mut out,
            &format!("{}_IPV4_PRIVATE", PROVIDER_PREFIX),
            "private-ipv4",
        )?;
        self.fetch_attribute(
            &mut out,
            &format!("{}_IPV4_PUBLIC", PROVIDER_PREFIX),
            "public-ipv4",
        )?;
        self.fetch_attribute(
            &mut out,
            &format!("{}_REGION_ID", PROVIDER_PREFIX),
            "region-id",
        )?;
        self.fetch_attribute(&mut out, &format!("{}_VPC_ID", PROVIDER_PREFIX), "vpc-id")?;
        self.fetch_attribute(&mut out, &format!("{}_ZONE_ID", PROVIDER_PREFIX), "zone-id")?;

        Ok(out)
    }

    fn hostname(&self) -> Result<Option<String>> {
        self.fetch_hostname()
    }

    fn ssh_keys(&self) -> Result<Vec<PublicKey>> {
        let entries = self.fetch_ssh_keys()?;
        let mut out = Vec::with_capacity(entries.len());
        for key in entries {
            match PublicKey::parse(&key) {
                Ok(pk) => out.push(pk),
                Err(e) => error!("failed to parse SSH public-key entry: {}", e),
            };
        }

        Ok(out)
    }
}
