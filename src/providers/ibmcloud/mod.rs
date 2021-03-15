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

use openssh_keys::PublicKey;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::str;

use tempfile::TempDir;

use crate::errors::*;
use crate::providers::MetadataProvider;

use mailparse::*;
use serde_derive::Deserialize;

const CONFIG_DRIVE_LABEL: &str = "cidata";

/// IBMCloud provider (VPC Gen2).
#[derive(Debug)]
pub struct IBMGen2Provider {
    /// Path to the top directory of the mounted config-drive.
    drive_path: PathBuf,
    /// Temporary directory for own mountpoint.
    temp_dir: TempDir,
}

impl IBMGen2Provider {
    /// Try to build a new provider client.
    ///
    /// This internally tries to mount (and own) the config-drive.
    pub fn try_new() -> Result<Self> {
        let target = tempfile::Builder::new()
            .prefix("afterburn-")
            .tempdir()
            .chain_err(|| "failed to create temporary directory")?;
        crate::util::mount_ro(
            &Path::new("/dev/disk/by-label/").join(CONFIG_DRIVE_LABEL),
            target.path(),
            "iso9660",
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

        for line in input.lines() {
            let line = line.chain_err(|| "reading metadata")?;
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

    /// Read vendordata file into a string
    fn read_vendordata(&self) -> Result<Vec<u8>> {
        let filename = self.metadata_dir().join("vendor-data");
        let mut file = File::open(&filename)
            .chain_err(|| format!("Failed to open vendordata '{:?}'", filename))?;
        let mut contents = String::new();
        let _ = file.read_to_string(&mut contents);
        Ok(contents.into_bytes())
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

    /// Find the SSH keys in the vendordata file
    fn fetch_ssh_keys(vendordata_vec: Vec<u8>) -> Result<Vec<String>> {
        // Parse MIME format from vendor-data file
        let vendor_data_mail =
            parse_mail(&vendordata_vec).chain_err(|| "failed to parse MIME vendor-data")?;
        let mut cloud_config = String::new();
        for section in vendor_data_mail.subparts {
            for header in &section.headers {
                if let "text/cloud-config" = header.get_value().as_str() {
                    if section
                        .get_body()
                        .unwrap_or_default()
                        .contains("ssh_authorized_keys")
                    {
                        cloud_config = section
                            .get_body()
                            .chain_err(|| "failed to get cloud-config content")?;
                        break;
                    }
                }
            }
        }
        // Parse YAML to find SSH keys
        if cloud_config.is_empty() {
            return Err("no cloud-config section found in vendor-data".into());
        }
        let deserialized_cloud_config: VendorDataCloudConfig = serde_yaml::from_str(&cloud_config)
            .chain_err(|| "failed to deserialize cloud-config content")?;
        Ok(deserialized_cloud_config.ssh_authorized_keys)
    }
}

impl MetadataProvider for IBMGen2Provider {
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
        let mut out = Vec::new();

        let vendordata = self.read_vendordata()?;
        for key in IBMGen2Provider::fetch_ssh_keys(vendordata)? {
            let key = PublicKey::parse(&key)?;
            out.push(key);
        }

        Ok(out)
    }
}

impl Drop for IBMGen2Provider {
    fn drop(&mut self) {
        if let Err(e) = crate::util::unmount(
            self.temp_dir.path(),
            3, // maximum retries
        ) {
            slog_scope::error!("failed to unmount IBM Cloud (Gen2) config-drive: {}", e);
        };
    }
}

/// This struct represents the portion of the vendor-data file we will be deserializing.
/// This data is in the "cloud-config" portion of the vendor-data file.
/// The cloud-config can have fields not defined here, they will be ignored.
/// The vendor-data file is in MIME format, the cloud-config data is in YAML format.
#[derive(Debug, Deserialize, Clone)]
struct VendorDataCloudConfig {
    ssh_authorized_keys: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Cursor;

    #[test]
    fn test_basic_attributes() {
        let metadata = r#"
instance-id: 1711_2a588fe2-7da2-4321-1234-1199b77d3911
local-hostname: test_instance-vpc-gen2
foo:      ba:r
"#;

        let bufrd = BufReader::new(Cursor::new(metadata));
        let parsed = IBMGen2Provider::parse_metadata(bufrd).unwrap();
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

        let attrs = IBMGen2Provider::known_attributes(parsed);
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

    #[test]
    fn test_fetch_ssh_keys() {
        let vendordata = fs::read("./tests/fixtures/ibmcloud/vendor-data")
            .expect("Unable to read vendor-data fixture");
        let ssh_keys = IBMGen2Provider::fetch_ssh_keys(vendordata).unwrap();
        assert!(ssh_keys
            .iter()
            .any(|i| i == "ssh-rsa AAAAB3NzaC1yc2 <<snip>> 3TIX+eesnqasq9w== testuser@test.com"));
        assert!(ssh_keys
            .iter()
            .any(|i| i == "ssh-rsa AAAAB4NzaC2yc3 <<snip>> 3TIX+eesnqasq9w== testuser2@test.com"));
    }
}
