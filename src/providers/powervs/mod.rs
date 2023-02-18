//! Metadata fetcher for PowerVS instances.
//!
//! This provider supports the Power Virtual Server infrastructure type on IBMCloud.
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

///PowerVS provider.
#[derive(Debug)]
pub struct PowerVSProvider {
    /// Path to the top directory of the mounted config-drive.
    drive_path: PathBuf,
    /// Temporary directory for own mountpoint.
    temp_dir: TempDir,
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
    pub public_keys: Option<HashMap<String, String>>,
}

impl PowerVSProvider {
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
        let file =
            File::open(&filename).with_context(|| format!("failed to open file '{filename:?}'"))?;
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

        if metadata.local_hostname.is_empty() {
            bail!("empty local hostname");
        }

        let attrs = maplit::hashmap! {
            "POWERVS_INSTANCE_ID".to_string() => metadata.instance_id,
            "POWERVS_LOCAL_HOSTNAME".to_string() => metadata.local_hostname,

        };
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

impl MetadataProvider for PowerVSProvider {
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

impl Drop for PowerVSProvider {
    fn drop(&mut self) {
        if let Err(e) = crate::util::unmount(
            self.temp_dir.path(),
            3, // maximum retries
        ) {
            slog_scope::error!("failed to unmount powervs config-drive: {}", e);
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_powervs_basic_attributes() {
        let metadata = r#"
{
  "hostname": "test_instance-powervs.foo.cloud",
  "name": "test_instance-powervs",
  "uuid": "41b4fb82-ca29-11eb-b8bc-0242ac130003"
}
"#;

        let bufrd = BufReader::new(Cursor::new(metadata));
        let parsed = PowerVSProvider::parse_metadata(bufrd).unwrap();
        assert_eq!(parsed.instance_id, "41b4fb82-ca29-11eb-b8bc-0242ac130003",);
        assert_eq!(parsed.local_hostname, "test_instance-powervs",);

        let attrs = PowerVSProvider::known_attributes(parsed).unwrap();
        assert_eq!(attrs.len(), 2);
        assert_eq!(
            attrs.get("POWERVS_INSTANCE_ID"),
            Some(&"41b4fb82-ca29-11eb-b8bc-0242ac130003".to_string())
        );
        assert_eq!(
            attrs.get("POWERVS_LOCAL_HOSTNAME"),
            Some(&"test_instance-powervs".to_string())
        );
    }

    #[test]
    fn test_powervs_parse_metadata_json() {
        let fixture = File::open("./tests/fixtures/powervs/meta_data.json").unwrap();
        let bufrd = BufReader::new(fixture);
        let parsed = PowerVSProvider::parse_metadata(bufrd).unwrap();

        assert!(!parsed.instance_id.is_empty());
        assert!(!parsed.local_hostname.is_empty());
        assert!(parsed.public_keys.is_some());
    }

    #[test]
    fn test_powervs_ssh_keys() {
        let fixture = File::open("./tests/fixtures/powervs/meta_data.json").unwrap();
        let bufrd = BufReader::new(fixture);
        let parsed = PowerVSProvider::parse_metadata(bufrd).unwrap();
        let keys = PowerVSProvider::public_keys(parsed).unwrap();
        let expect = PublicKey::parse("ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQDmMuiypdqqftqhrQeBTjOhcgyARvylZMLiH+6nCvi5Lv5M7evAnvvG3Hz4rbjbbqoVgSCIdAEb4PuttiCdwE6UyAl0TYAydOVPx7l87BlaucTEqDFbXkQB+yyUmzodllCpWAMUmxwvJB/ntFrC6rP0K0kKxx4SESvozutwM2X5oH3LNHcYI1xgKIMF9VMJLkkM0rLo8Fmj6mWF5KtbU7vS7JJPvLTCRhW5TYrqvhHKuIS6KBtj3GJqvRt+it8AsIb6/RUaji68Mt7W41UrmFSPt8bxJMdE/xKGMcFQjamURPSCHx7z8/pr2/pv2QbQF76FO7lRdPH3f542uAkOOpO1 user1@user1-MacBook-Pro-2.local").unwrap();

        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0], expect);
    }
}
