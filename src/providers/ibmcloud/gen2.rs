//! Metadata fetcher for IBMCloud (VPC Gen2) instances.
//!
//! This provider supports the "VPC Generation 2" infrastructure type
//! on IBMCloud.
//! It provides a config-drive as the only metadata source, whose layout
//! is very similar to `cloud-init NoCloud` [datasource][nocloud], with
//! a few variations:
//!  - disk label is `cidata` (lowercase)
//!  - filesystem is `iso9660`
//!
//! nocloud: https://cloudinit.readthedocs.io/en/latest/topics/datasources/nocloud.html

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

use openssh_keys::PublicKey;
use slog_scope::warn;
use tempdir::TempDir;

use crate::errors::*;
use crate::network;
use crate::providers::MetadataProvider;

const CONFIG_DRIVE_LABEL: &str = "cidata";

/// IBMCloud provider (VPC Gen2).
#[derive(Debug)]
pub struct G2Provider {
    /// Path to the top directory of the mounted config-drive.
    drive_path: PathBuf,
    /// Temporary directory for own mountpoint (if any).
    temp_dir: Option<TempDir>,
}

impl G2Provider {
    /// Try to build a new provider client.
    ///
    /// This internally tries to mount (and own) the config-drive.
    pub fn try_new() -> Result<Self> {
        let target =
            TempDir::new("afterburn").chain_err(|| "failed to create temporary directory")?;
        crate::util::mount_ro(
            &Path::new("/dev/disk/by-label/").join(CONFIG_DRIVE_LABEL),
            target.path(),
            "iso9660",
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
        self.drive_path.clone()
    }

    /// Read metadata file and parse attributes.
    fn read_metadata(&self) -> Result<HashMap<String, String>> {
        let filename = self.metadata_dir().join("meta-data");
        let file =
            File::open(&filename).chain_err(|| format!("failed to open file '{:?}'", filename))?;
        let bufrd = BufReader::new(file);
        Self::parse_metadata(bufrd)
    }

    /// Parse metadata attributes.
    ///
    /// Metadata file contains one attribute per line, in the form of
    /// `key: value\n`.
    fn parse_metadata<T: Read>(input: BufReader<T>) -> Result<HashMap<String, String>> {
        let mut output = HashMap::new();

        for line in input.lines().filter_map(|l| l.ok()) {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() != 2 {
                continue;
            }
            let key = parts[0].trim().to_string();
            let value = parts[1].trim().to_string();
            if !key.is_empty() && !value.is_empty() {
                output.insert(key, value);
            }
        }

        Ok(output)
    }

    /// Extract supported metadata values and convert to Afterburn attributes.
    ///
    /// The `AFTERBURN_` prefix is added later on, so it is not part of the
    /// key-labels here.
    fn known_attributes(input: HashMap<String, String>) -> HashMap<String, String> {
        let mut output = HashMap::new();
        for (key, value) in input {
            match key.as_str() {
                "instance-id" => {
                    output.insert("IBMCLOUD_INSTANCE_ID".to_string(), value);
                }
                "local-hostname" => {
                    output.insert("IBMCLOUD_LOCAL_HOSTNAME".to_string(), value);
                }
                _ => {}
            };
        }
        output
    }
}

impl MetadataProvider for G2Provider {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        let metadata = self.read_metadata()?;
        let attrs = Self::known_attributes(metadata);
        Ok(attrs)
    }

    fn hostname(&self) -> Result<Option<String>> {
        let metadata = self.read_metadata()?;
        let hostname = metadata.get("local-hostname").map(String::from);
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

impl Drop for G2Provider {
    fn drop(&mut self) {
        if let Some(ref mountpoint) = self.temp_dir {
            if let Err(e) = crate::util::unmount(
                mountpoint.path(),
                3, // maximum retries
            ) {
                slog_scope::error!("failed to unmount IBM Cloud (Gen2) config-drive: {}", e);
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
instance-id: 1711_2a588fe2-7da2-4321-1234-1199b77d3911
local-hostname: test_instance-vpc-gen2
foo:      ba:r
"#;

        let bufrd = BufReader::new(Cursor::new(metadata));
        let parsed = G2Provider::parse_metadata(bufrd).unwrap();
        assert_eq!(parsed.len(), 3);
        assert_eq!(
            parsed.get("instance-id"),
            Some(&"1711_2a588fe2-7da2-4321-1234-1199b77d3911".to_string())
        );
        assert_eq!(
            parsed.get("local-hostname"),
            Some(&"test_instance-vpc-gen2".to_string())
        );
        assert_eq!(parsed.get("foo"), Some(&"ba:r".to_string()));

        let attrs = G2Provider::known_attributes(parsed);
        assert_eq!(attrs.len(), 2);
        assert_eq!(
            attrs.get("IBMCLOUD_INSTANCE_ID"),
            Some(&"1711_2a588fe2-7da2-4321-1234-1199b77d3911".to_string())
        );
        assert_eq!(
            attrs.get("IBMCLOUD_LOCAL_HOSTNAME"),
            Some(&"test_instance-vpc-gen2".to_string())
        );
    }
}
