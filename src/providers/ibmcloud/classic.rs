//! Metadata fetcher for IBMCloud (Classic) instances.
//!
//! This provider supports the "Classic" infrastructure type on IBMCloud.
//! It provides a config-drive as the only metadata source, whose layout
//! follows the `cloud-init ConfigDrive v2` [datasource][configdrive], with
//! the following details:
//!  - disk filesystem label is `config-2` (lowercase)
//!  - filesystem is `vfat`
//!  - drive contains a single directory at `/openstack/latest/`
//!  - content is exposed as JSON files called `meta_data.json`, `network_data.json`, and `vendor_data.json`.
//!
//! configdrive: https://cloudinit.readthedocs.io/en/latest/topics/datasources/configdrive.html

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use error_chain::bail;
use openssh_keys::PublicKey;
use serde::Deserialize;
use slog_scope::warn;
use tempdir::TempDir;

use crate::errors::*;
use crate::network;
use crate::providers::MetadataProvider;

// Filesystem label for the Config Drive.
static CONFIG_DRIVE_FS_LABEL: &str = "config-2";

// Filesystem type for the Config Drive.
static CONFIG_DRIVE_FS_TYPE: &str = "vfat";

/// IBMCloud provider (Classic).
#[derive(Debug)]
pub struct ClassicProvider {
    /// Path to the top directory of the mounted config-drive.
    drive_path: PathBuf,
    /// Temporary directory for own mountpoint (if any).
    temp_dir: Option<TempDir>,
}

/// Partial object for `meta_data.json`
#[derive(Debug, Deserialize)]
pub struct MetaDataJSON {
    /// Fully-Qualified Domain Name (FQDN).
    #[serde(rename = "hostname")]
    pub fqdn: String,
    /// Local hostname.
    #[serde(rename = "name")]
    pub local_hostname: String,
    /// Instance ID (UUID).
    #[serde(rename = "uuid")]
    pub instance_id: String,
    /// SSH public keys.
    pub public_keys: HashMap<String, String>,
}

impl ClassicProvider {
    /// Try to build a new provider client.
    ///
    /// This internally tries to mount (and own) the config-drive.
    pub fn try_new() -> Result<Self> {
        let target =
            TempDir::new("afterburn").chain_err(|| "failed to create temporary directory")?;
        crate::util::mount_ro(
            &Path::new("/dev/disk/by-label/").join(CONFIG_DRIVE_FS_LABEL),
            target.path(),
            CONFIG_DRIVE_FS_TYPE,
            3, // maximum retries
        )?;

        let provider = Self {
            drive_path: target.path().to_owned(),
            temp_dir: Some(target),
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
        let file =
            File::open(&filename).chain_err(|| format!("failed to open file '{:?}'", filename))?;
        let bufrd = BufReader::new(file);
        Self::parse_metadata(bufrd)
    }

    /// Parse metadata attributes.
    ///
    /// Metadata file contains a JSON object, corresponding to `MetaDataJSON`.
    fn parse_metadata<T: Read>(input: BufReader<T>) -> Result<MetaDataJSON> {
        serde_json::from_reader(input).chain_err(|| "failed parse JSON metadata")
    }

    /// Extract supported metadata values and convert to Afterburn attributes.
    ///
    /// The `AFTERBURN_` prefix is added later on, so it is not part of the
    /// key-labels here.
    fn known_attributes(metadata: MetaDataJSON) -> Result<HashMap<String, String>> {
        if metadata.instance_id.is_empty() {
            bail!("empty instance ID");
        }

        if metadata.local_hostname.is_empty() {
            bail!("empty local hostname");
        }

        let attrs = maplit::hashmap! {
            "IBMCLOUD_INSTANCE_ID".to_string() => metadata.instance_id,
            "IBMCLOUD_LOCAL_HOSTNAME".to_string() => metadata.local_hostname,

        };
        Ok(attrs)
    }
}

impl MetadataProvider for ClassicProvider {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        let metadata = self.read_metadata()?;
        Self::known_attributes(metadata)
    }

    fn hostname(&self) -> Result<Option<String>> {
        let metadata = self.read_metadata()?;
        let hostname = if metadata.local_hostname.is_empty() {
            None
        } else {
            Some(metadata.local_hostname)
        };
        Ok(hostname)
    }

    fn ssh_keys(&self) -> Result<Vec<PublicKey>> {
        warn!("cloud SSH keys requested, but not supported on this platform");
        Ok(vec![])
    }

    fn networks(&self) -> Result<Vec<network::Interface>> {
        warn!("network metadata requested, but not supported on this platform");
        Ok(vec![])
    }

    fn network_devices(&self) -> Result<Vec<network::Device>> {
        warn!("network devices metadata requested, but not supported on this platform");
        Ok(vec![])
    }

    fn boot_checkin(&self) -> Result<()> {
        warn!("boot check-in requested, but not supported on this platform");
        Ok(())
    }
}

impl Drop for ClassicProvider {
    fn drop(&mut self) {
        if let Some(ref mountpoint) = self.temp_dir {
            if let Err(e) = crate::util::unmount(
                mountpoint.path(),
                3, // maximum retries
            ) {
                slog_scope::error!("failed to unmount ibmcloud (Classic) config-drive: {}", e);
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_basic_attributes() {
        let metadata = r#"
{
  "hostname": "test_instance-classic.foo.cloud",
  "name": "test_instance-classic",
  "uuid": "3c9085db-3eba-4ef2-9d97-d3ffcff6fffe",
  "public_keys": {
    "1602320": "ssh-rsa AAAA foo@example.com"
  }
}
"#;

        let bufrd = BufReader::new(Cursor::new(metadata));
        let parsed = ClassicProvider::parse_metadata(bufrd).unwrap();
        assert_eq!(parsed.instance_id, "3c9085db-3eba-4ef2-9d97-d3ffcff6fffe",);
        assert_eq!(parsed.local_hostname, "test_instance-classic",);

        let attrs = ClassicProvider::known_attributes(parsed).unwrap();
        assert_eq!(attrs.len(), 2);
        assert_eq!(
            attrs.get("IBMCLOUD_INSTANCE_ID"),
            Some(&"3c9085db-3eba-4ef2-9d97-d3ffcff6fffe".to_string())
        );
        assert_eq!(
            attrs.get("IBMCLOUD_LOCAL_HOSTNAME"),
            Some(&"test_instance-classic".to_string())
        );
    }

    #[test]
    fn test_parse_metadata_json() {
        let fixture = File::open("./tests/fixtures/ibmcloud/classic/meta_data.json").unwrap();
        let bufrd = BufReader::new(fixture);
        let parsed = ClassicProvider::parse_metadata(bufrd).unwrap();

        assert!(!parsed.instance_id.is_empty());
        assert!(!parsed.local_hostname.is_empty());
        assert!(!parsed.public_keys.is_empty());
    }
}
