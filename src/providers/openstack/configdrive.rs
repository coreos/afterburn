//! configdrive metadata fetcher for OpenStack
//! reference: https://docs.openstack.org/nova/latest/user/metadata.html

use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use openssh_keys::PublicKey;
use slog_scope::{error, warn};
use tempfile::TempDir;

use crate::errors::*;
use crate::network;
use crate::providers::MetadataProvider;

const CONFIG_DRIVE_LABEL: &str = "config-2";

/// Partial object for ec2 `meta_data.json`
#[derive(Debug, Deserialize)]
pub struct MetadataEc2JSON {
    /// Instance ID.
    #[serde(rename = "instance-id")]
    pub instance_id: Option<String>,
    /// Instance type.
    #[serde(rename = "instance-type")]
    pub instance_type: Option<String>,
    /// Local IPV4.
    #[serde(rename = "local-ipv4")]
    pub local_ipv4: Option<String>,
    /// Public IPV4.
    #[serde(rename = "public-ipv4")]
    pub public_ipv4: Option<String>,
}

/// Partial object for openstack `meta_data.json`
#[derive(Debug, Deserialize)]
pub struct MetadataOpenstackJSON {
    /// Availability zone.
    pub availability_zone: Option<String>,
    /// Local hostname.
    pub hostname: Option<String>,
    /// SSH public keys.
    pub public_keys: Option<HashMap<String, String>>,
}

/// OpenStack config-drive.
#[derive(Debug)]
pub struct OpenstackConfigDrive {
    /// Path to the top directory of the mounted config-drive.
    drive_path: PathBuf,
    /// Temporary directory for own mountpoint (if any).
    temp_dir: Option<TempDir>,
}

impl OpenstackConfigDrive {
    /// Try to build a new provider client.
    ///
    /// This internally tries to mount (and own) the config-drive.
    pub fn try_new() -> Result<Self> {
        const TARGET_FS: &str = "iso9660";
        let target = tempfile::Builder::new()
            .prefix("afterburn-")
            .tempdir()
            .chain_err(|| "failed to create temporary directory")?;
        crate::util::mount_ro(
            &Path::new("/dev/disk/by-label/").join(CONFIG_DRIVE_LABEL),
            target.path(),
            TARGET_FS,
            3,
        )?;

        let cd = OpenstackConfigDrive {
            drive_path: target.path().to_owned(),
            temp_dir: Some(target),
        };
        Ok(cd)
    }

    /// Return the path to the metadata directory.
    fn metadata_dir(&self, platform: &str) -> PathBuf {
        self.drive_path.clone().join(platform).join("latest")
    }

    /// Parse metadata attributes
    ///
    /// Metadata file contains a JSON object, corresponding to `MetadataEc2JSON`.
    fn parse_metadata_ec2<T: Read>(input: BufReader<T>) -> Result<MetadataEc2JSON> {
        serde_json::from_reader(input).chain_err(|| "failed parse JSON metadata")
    }

    /// Parse metadata attributes
    ///
    /// Metadata file contains a JSON object, corresponding to `MetadataOpenstackJSON`.
    fn parse_metadata_openstack<T: Read>(input: BufReader<T>) -> Result<MetadataOpenstackJSON> {
        serde_json::from_reader(input).chain_err(|| "failed parse JSON metadata")
    }

    /// The metadata is stored as key:value pair in ec2/latest/meta-data.json file
    fn read_metadata_ec2(&self) -> Result<MetadataEc2JSON> {
        let filename = self.metadata_dir("ec2").join("meta-data.json");
        let file =
            File::open(&filename).chain_err(|| format!("failed to open file '{:?}'", filename))?;
        let bufrd = BufReader::new(file);
        Self::parse_metadata_ec2(bufrd)
            .chain_err(|| format!("failed to parse file '{:?}'", filename))
    }

    /// The metadata is stored as key:value pair in openstack/latest/meta_data.json file
    fn read_metadata_openstack(&self) -> Result<MetadataOpenstackJSON> {
        let filename = self.metadata_dir("openstack").join("meta_data.json");
        let file =
            File::open(&filename).chain_err(|| format!("failed to open file '{:?}'", filename))?;
        let bufrd = BufReader::new(file);
        Self::parse_metadata_openstack(bufrd)
            .chain_err(|| format!("failed to parse file '{:?}'", filename))
    }

    /// The public key is stored as key:value pair in openstack/latest/meta_data.json file
    fn fetch_publickeys(&self) -> Result<Vec<PublicKey>> {
        let filename = self.metadata_dir("openstack").join("meta_data.json");
        let file =
            File::open(&filename).chain_err(|| format!("failed to open file '{:?}'", filename))?;

        let bufrd = BufReader::new(file);
        let metadata: MetadataOpenstackJSON = Self::parse_metadata_openstack(bufrd)
            .chain_err(|| format!("failed to parse file '{:?}'", filename))?;

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

impl MetadataProvider for OpenstackConfigDrive {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        let mut out = HashMap::with_capacity(5);
        let metadata_ec2: MetadataEc2JSON = self.read_metadata_ec2()?;
        let metadata_openstack: MetadataOpenstackJSON = self.read_metadata_openstack()?;
        if let Some(hostname) = metadata_openstack.hostname {
            out.insert("OPENSTACK_HOSTNAME".to_string(), hostname);
        }
        if let Some(instance_id) = metadata_ec2.instance_id {
            out.insert("OPENSTACK_INSTANCE_ID".to_string(), instance_id);
        }
        if let Some(instance_type) = metadata_ec2.instance_type {
            out.insert("OPENSTACK_INSTANCE_TYPE".to_string(), instance_type);
        }
        if let Some(local_ipv4) = metadata_ec2.local_ipv4 {
            out.insert("OPENSTACK_IPV4_LOCAL".to_string(), local_ipv4);
        }
        if let Some(public_ipv4) = metadata_ec2.public_ipv4 {
            out.insert("OPENSTACK_IPV4_PUBLIC".to_string(), public_ipv4);
        }
        Ok(out)
    }

    fn hostname(&self) -> Result<Option<String>> {
        let metadata: MetadataOpenstackJSON = self.read_metadata_openstack()?;
        Ok(metadata.hostname)
    }

    fn ssh_keys(&self) -> Result<Vec<PublicKey>> {
        self.fetch_publickeys()
    }

    fn networks(&self) -> Result<Vec<network::Interface>> {
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

impl Drop for OpenstackConfigDrive {
    fn drop(&mut self) {
        if self.temp_dir.is_some() {
            if let Err(e) = crate::util::unmount(&self.drive_path, 3) {
                error!("failed to cleanup OpenStack config-drive: {:?}", e);
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attributes_ec2() {
        let fixture =
            File::open("./tests/fixtures/openstack-config-drive/ec2/meta-data.json").unwrap();
        let bufrd = BufReader::new(fixture);
        let parsed = OpenstackConfigDrive::parse_metadata_ec2(bufrd).unwrap();

        assert_eq!(parsed.instance_id.unwrap_or_default(), "i-022da7a2");
        assert_eq!(parsed.instance_type.unwrap_or_default(), "m1.small");
        assert_eq!(parsed.local_ipv4.unwrap_or_default(), "10.0.151.35");
        assert_eq!(parsed.public_ipv4.unwrap_or_default(), "");
    }

    #[test]
    fn test_attributes_openstack() {
        let fixture =
            File::open("./tests/fixtures/openstack-config-drive/openstack/meta_data.json").unwrap();
        let bufrd = BufReader::new(fixture);
        let parsed = OpenstackConfigDrive::parse_metadata_openstack(bufrd).unwrap();

        let expect = maplit::hashmap! {
            "mykey".to_string() => "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAAAgQDYVEprvtYJXVOBN0XNKVVRNCRX6BlnNbI+USLGais1sUWPwtSg7z9K9vhbYAPUZcq8c/s5S9dg5vTHbsiyPCIDOKyeHba4MUJq8Oh5b2i71/3BISpyxTBH/uZDHdslW2a+SrPDCeuMMoss9NFhBdKtDkdG9zyi0ibmCP6yMdEX8Q== Generated by Nova\n".to_string(),
        };

        assert_eq!(
            parsed.hostname.unwrap_or_default(),
            "abai-fcos-afterburn-test"
        );
        assert_eq!(parsed.availability_zone.unwrap_or_default(), "nova");
        assert_eq!(parsed.public_keys.unwrap_or_default(), expect);
    }

    #[test]
    fn test_ssh_keys() {
        let fixture =
            File::open("./tests/fixtures/openstack-config-drive/openstack/meta_data.json").unwrap();
        let bufrd = BufReader::new(fixture);
        let parsed = OpenstackConfigDrive::parse_metadata_openstack(bufrd).unwrap();

        let expect = maplit::hashmap! {
            "mykey".to_string() => "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAAAgQDYVEprvtYJXVOBN0XNKVVRNCRX6BlnNbI+USLGais1sUWPwtSg7z9K9vhbYAPUZcq8c/s5S9dg5vTHbsiyPCIDOKyeHba4MUJq8Oh5b2i71/3BISpyxTBH/uZDHdslW2a+SrPDCeuMMoss9NFhBdKtDkdG9zyi0ibmCP6yMdEX8Q== Generated by Nova\n".to_string(),
        };

        assert_eq!(parsed.public_keys.unwrap_or_default(), expect);
    }
}
