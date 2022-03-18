//! Metadata fetcher for KubeVirt instances.
//!
//! This provider supports platforms based on KubeVirt.
//! It provides a config-drive as the only metadata source, whose layout
//! follows the `cloud-init ConfigDrive v2` [datasource][configdrive], with
//! the following details:
//!  - disk filesystem label is `config-2` (lowercase)
//!  - filesystem is `iso9660`
//!  - drive contains a single directory at `/openstack/latest/`
//!  - content is exposed as JSON files called `meta_data.json`.
//!
//! configdrive: https://cloudinit.readthedocs.io/en/latest/topics/datasources/configdrive.html

use anyhow::{bail, Context, Result};
use openssh_keys::PublicKey;
use serde::Deserialize;
use slog_scope::warn;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

use crate::network;
use crate::providers::MetadataProvider;

// Filesystem label for the Config Drive.
static CONFIG_DRIVE_FS_LABEL: &str = "config-2";

// Filesystem type for the Config Drive.
static CONFIG_DRIVE_FS_TYPE: &str = "iso9660";

///KubeVirt provider.
#[derive(Debug)]
pub struct KubeVirtProvider {
    /// Path to the top directory of the mounted config-drive.
    drive_path: PathBuf,
    /// Temporary directory for own mountpoint.
    temp_dir: TempDir,
}

/// Partial object for `meta_data.json`
#[derive(Debug, Deserialize)]
pub struct MetaDataJSON {
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

impl KubeVirtProvider {
    /// Try to build a new provider client.
    ///
    /// This internally tries to mount (and own) the config-drive.
    pub fn try_new() -> Result<Self> {
        let target = tempfile::Builder::new()
            .prefix("afterburn-")
            .tempdir()
            .context("failed to create temporary directory")?;
        crate::util::mount_ro(
            &Path::new("/dev/disk/by-label/").join(CONFIG_DRIVE_FS_LABEL),
            target.path(),
            CONFIG_DRIVE_FS_TYPE,
            3, // maximum retries
        )?;

        let provider = Self {
            drive_path: target.path().to_owned(),
            temp_dir: target,
        };
        Ok(provider)
    }

    /// Return the path to the metadata directory.
    fn metadata_dir(&self) -> PathBuf {
        let drive = self.drive_path.clone();
        drive.join("openstack").join("latest")
    }

    /// Read and parse metadata file.
    fn read_metadata(&self) -> Result<MetaDataJSON> {
        let filename = self.metadata_dir().join("meta_data.json");
        let file = File::open(&filename)
            .with_context(|| format!("failed to open file '{:?}'", filename))?;
        let bufrd = BufReader::new(file);
        Self::parse_metadata(bufrd)
    }

    /// Parse metadata attributes.
    ///
    /// Metadata file contains a JSON object, corresponding to `MetaDataJSON`.
    fn parse_metadata<T: Read>(input: BufReader<T>) -> Result<MetaDataJSON> {
        serde_json::from_reader(input).context("failed to parse JSON metadata")
    }

    /// Extract supported metadata values and convert to Afterburn attributes.
    ///
    /// The `AFTERBURN_` prefix is added later on, so it is not part of the
    /// key-labels here.
    fn known_attributes(metadata: MetaDataJSON) -> Result<HashMap<String, String>> {
        if metadata.instance_id.is_empty() {
            bail!("empty instance ID");
        }

        if metadata.hostname.is_empty() {
            bail!("empty local hostname");
        }

        let mut attrs = maplit::hashmap! {
            "KUBEVIRT_INSTANCE_ID".to_string() => metadata.instance_id,
            "KUBEVIRT_HOSTNAME".to_string() => metadata.hostname,
        };
        if let Some(instance_type) = metadata.instance_type {
            attrs.insert("KUBEVIRT_INSTANCE_TYPE".to_string(), instance_type);
        }
        Ok(attrs)
    }

    /// The public key is stored as key:value pair in openstack/latest/meta_data.json file
    fn public_keys(metadata: MetaDataJSON) -> Result<Vec<PublicKey>> {
        let public_keys_map = metadata.public_keys.unwrap_or_default();
        let public_keys_vec: Vec<&std::string::String> = public_keys_map.values().collect();
        let mut out = vec![];
        for key in public_keys_vec {
            let key = PublicKey::parse(key)?;
            out.push(key);
        }
        Ok(out)
    }
}

impl MetadataProvider for KubeVirtProvider {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        let metadata = self.read_metadata()?;
        Self::known_attributes(metadata)
    }

    fn hostname(&self) -> Result<Option<String>> {
        let metadata = self.read_metadata()?;
        let hostname = if metadata.hostname.is_empty() {
            None
        } else {
            Some(metadata.hostname)
        };
        Ok(hostname)
    }

    fn ssh_keys(&self) -> Result<Vec<PublicKey>> {
        let metadata = self.read_metadata()?;
        Self::public_keys(metadata)
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

impl Drop for KubeVirtProvider {
    fn drop(&mut self) {
        if let Err(e) = crate::util::unmount(
            self.temp_dir.path(),
            3, // maximum retries
        ) {
            slog_scope::error!("failed to unmount kubevirt config-drive: {}", e);
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_kubevirt_basic_attributes() {
        let metadata = r#"
{
  "hostname": "test_instance-kubevirt.foo.cloud",
  "uuid": "41b4fb82-ca29-11eb-b8bc-0242ac130003"
}
"#;

        let bufrd = BufReader::new(Cursor::new(metadata));
        let parsed = KubeVirtProvider::parse_metadata(bufrd).unwrap();
        assert_eq!(parsed.instance_id, "41b4fb82-ca29-11eb-b8bc-0242ac130003");
        assert_eq!(parsed.hostname, "test_instance-kubevirt.foo.cloud");

        let attrs = KubeVirtProvider::known_attributes(parsed).unwrap();
        assert_eq!(attrs.len(), 2);
        assert_eq!(
            attrs.get("KUBEVIRT_INSTANCE_ID"),
            Some(&"41b4fb82-ca29-11eb-b8bc-0242ac130003".to_string())
        );
        assert_eq!(
            attrs.get("KUBEVIRT_HOSTNAME"),
            Some(&"test_instance-kubevirt.foo.cloud".to_string())
        );
    }

    #[test]
    fn test_kubevirt_extended_attributes() {
        let metadata = r#"
{
  "hostname": "test_instance-kubevirt.foo.cloud",
  "uuid": "41b4fb82-ca29-11eb-b8bc-0242ac130003",
  "instance_type": "some_type"
}
"#;

        let bufrd = BufReader::new(Cursor::new(metadata));
        let parsed = KubeVirtProvider::parse_metadata(bufrd).unwrap();
        assert_eq!(parsed.instance_id, "41b4fb82-ca29-11eb-b8bc-0242ac130003");
        assert_eq!(parsed.hostname, "test_instance-kubevirt.foo.cloud");
        assert_eq!(parsed.instance_type.as_deref().unwrap(), "some_type");

        let attrs = KubeVirtProvider::known_attributes(parsed).unwrap();
        assert_eq!(attrs.len(), 3);
        assert_eq!(
            attrs.get("KUBEVIRT_INSTANCE_ID"),
            Some(&"41b4fb82-ca29-11eb-b8bc-0242ac130003".to_string())
        );
        assert_eq!(
            attrs.get("KUBEVIRT_HOSTNAME"),
            Some(&"test_instance-kubevirt.foo.cloud".to_string())
        );
        assert_eq!(
            attrs.get("KUBEVIRT_INSTANCE_TYPE"),
            Some(&"some_type".to_string())
        );
    }

    #[test]
    fn test_kubevirt_parse_metadata_json() {
        let fixture = File::open("./tests/fixtures/kubevirt/meta_data.json").unwrap();
        let bufrd = BufReader::new(fixture);
        let parsed = KubeVirtProvider::parse_metadata(bufrd).unwrap();

        assert!(!parsed.instance_id.is_empty());
        assert!(!parsed.hostname.is_empty());
        assert!(!parsed.public_keys.is_none());
    }

    #[test]
    fn test_kubevirt_ssh_keys() {
        let fixture = File::open("./tests/fixtures/kubevirt/meta_data.json").unwrap();

        let bufrd = BufReader::new(fixture);
        let parsed = KubeVirtProvider::parse_metadata(bufrd).unwrap();
        let keys = KubeVirtProvider::public_keys(parsed).unwrap();
        let expect = PublicKey::parse("ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAAAgQDYVEprvtYJXVOBN0XNKVVRNCRX6BlnNbI+USLGais1sUWPwtSg7z9K9vhbYAPUZcq8c/s5S9dg5vTHbsiyPCIDOKyeHba4MUJq8Oh5b2i71/3BISpyxTBH/uZDHdslW2a+SrPDCeuMMoss9NFhBdKtDkdG9zyi0ibmCP6yMdEX8Q== Generated by Nova").unwrap();

        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0], expect);
    }
}
