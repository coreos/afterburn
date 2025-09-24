//! KubeVirt cloud config parsing.
//!
//! This provider supports platforms based on KubeVirt.
//! It provides a config-drive as the only metadata source, whose layout
//! follows the `cloud-init ConfigDrive v2` [datasource][configdrive], with
//! the following details:
//!  - disk filesystem label is `config-2` (lowercase)
//!  - filesystem is `iso9660`
//!  - drive contains a single directory at `/openstack/latest/`
//!  - content is exposed as JSON or YAML files called `meta_data.json`.
//!
//! configdrive: https://cloudinit.readthedocs.io/en/latest/topics/datasources/configdrive.html

use crate::{network, providers::MetadataProvider};
use anyhow::{bail, Context, Result};
use openssh_keys::PublicKey;
use serde::Deserialize;
use slog_scope::warn;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

/// Partial object for `meta_data.json`
#[derive(Debug, Deserialize)]
pub struct MetaData {
    /// Local hostname
    pub hostname: String,
    /// Instance ID (UUID).
    #[serde(rename = "uuid")]
    pub instance_id: String,
    /// Instance type.
    pub instance_type: Option<String>,
    /// SSH public keys.
    pub public_keys: Option<HashMap<String, String>>,
}

#[derive(Debug)]
pub struct KubeVirtCloudConfig {
    pub meta_data: MetaData,
}

impl KubeVirtCloudConfig {
    pub fn try_new(path: &Path) -> Result<Self> {
        let metadata_dir = path.join("openstack").join("latest");
        let filename = metadata_dir.join("meta_data.json");
        let file =
            File::open(&filename).with_context(|| format!("failed to open file '{filename:?}'"))?;
        let bufrd = BufReader::new(file);
        let meta_data = Self::parse_metadata(bufrd)?;

        Ok(Self { meta_data })
    }

    /// Parse metadata attributes.
    ///
    /// Metadata file contains a JSON or YAML object, corresponding to `MetaData`.
    pub fn parse_metadata<T: Read>(input: BufReader<T>) -> Result<MetaData> {
        serde_json::from_reader(input).context("failed to parse metadata")
    }
}

impl MetadataProvider for KubeVirtCloudConfig {
    /// Extract supported cloud config values and convert to Afterburn attributes.
    ///
    /// The `AFTERBURN_` prefix is added later on, so it is not part of the
    /// key-labels here.
    fn attributes(&self) -> Result<HashMap<String, String>> {
        if self.meta_data.instance_id.is_empty() {
            bail!("empty instance ID");
        }

        if self.meta_data.hostname.is_empty() {
            bail!("empty local hostname");
        }

        let mut attrs = maplit::hashmap! {
            "KUBEVIRT_INSTANCE_ID".to_string() => self.meta_data.instance_id.clone(),
            "KUBEVIRT_HOSTNAME".to_string() => self.meta_data.hostname.clone(),
        };
        if let Some(instance_type) = &self.meta_data.instance_type {
            attrs.insert("KUBEVIRT_INSTANCE_TYPE".to_string(), instance_type.clone());
        }
        Ok(attrs)
    }

    fn hostname(&self) -> Result<Option<String>> {
        let hostname = if self.meta_data.hostname.is_empty() {
            None
        } else {
            Some(self.meta_data.hostname.clone())
        };
        Ok(hostname)
    }

    /// The public key is stored as key:value pair in openstack/latest/meta_data.json file
    fn ssh_keys(&self) -> Result<Vec<PublicKey>> {
        self.meta_data
            .public_keys
            .iter()
            .flat_map(|keys| keys.values())
            .map(|key| PublicKey::parse(key).map_err(anyhow::Error::from))
            .collect()
    }

    fn networks(&self) -> Result<Vec<network::Interface>> {
        warn!("network interfaces metadata requested, but not supported on this platform");
        Ok(vec![])
    }

    fn virtual_network_devices(&self) -> Result<Vec<network::VirtualNetDev>> {
        warn!("virtual network devices metadata requested, but not supported on this platform");
        Ok(vec![])
    }

    fn boot_checkin(&self) -> Result<()> {
        warn!("boot check-in requested, but not supported on this platform");
        Ok(())
    }
}
