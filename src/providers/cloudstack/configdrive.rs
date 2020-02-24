//! configdrive metadata fetcher for cloudstack

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use openssh_keys::PublicKey;
use slog_scope::{error, warn};
use tempfile::TempDir;

use crate::errors::*;
use crate::network;
use crate::providers::MetadataProvider;

const CONFIG_DRIVE_LABEL_1: &str = "config-2";
const CONFIG_DRIVE_LABEL_2: &str = "CONFIG-2";

/// CloudStack config-drive.
#[derive(Debug)]
pub struct ConfigDrive {
    /// Path to the top directory of the mounted config-drive.
    drive_path: PathBuf,
    /// Temporary directory for own mountpoint (if any).
    temp_dir: Option<TempDir>,
}

impl ConfigDrive {
    /// Try to build a new provider client.
    ///
    /// This internally tries to mount (and own) the config-drive.
    pub fn try_new() -> Result<Self> {
        // Short-circuit if the config-drive is already mounted.
        let path = Path::new("/media/ConfigDrive/cloudstack/metadata/");
        if path.exists() {
            return Ok(ConfigDrive {
                temp_dir: None,
                drive_path: PathBuf::from("/media/ConfigDrive/"),
            });
        }

        // Otherwise, try and mount with each of the labels.
        let target = tempfile::Builder::new()
            .prefix("afterburn-")
            .tempdir()
            .chain_err(|| "failed to create temporary directory")?;
        crate::util::mount_ro(
            &Path::new("/dev/disk/by-label/").join(CONFIG_DRIVE_LABEL_1),
            target.path(),
            "iso9660",
            3,
        )
        .or_else(|_| {
            crate::util::mount_ro(
                &Path::new("/dev/disk/by-label/").join(CONFIG_DRIVE_LABEL_2),
                target.path(),
                "iso9660",
                3,
            )
        })?;

        let cd = ConfigDrive {
            drive_path: target.path().to_owned(),
            temp_dir: Some(target),
        };
        Ok(cd)
    }

    /// Return the path to the metadata directory.
    fn metadata_dir(&self) -> PathBuf {
        self.drive_path.clone().join("cloudstack").join("metadata")
    }

    fn fetch_value(&self, key: &str) -> Result<Option<String>> {
        let filename = self.metadata_dir().join(format!("{}.txt", key));

        if !filename.exists() {
            return Ok(None);
        }

        let mut file =
            File::open(&filename).chain_err(|| format!("failed to open file '{:?}'", filename))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .chain_err(|| format!("failed to read from file '{:?}'", filename))?;

        Ok(Some(contents))
    }

    fn fetch_publickeys(&self) -> Result<Vec<PublicKey>> {
        let filename = self.metadata_dir().join("public_keys.txt");
        let file =
            File::open(&filename).chain_err(|| format!("failed to open file '{:?}'", filename))?;

        PublicKey::read_keys(file).chain_err(|| "failed to read public keys from config drive file")
    }
}

impl MetadataProvider for ConfigDrive {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        let mut out = HashMap::with_capacity(6);
        let add_value = |map: &mut HashMap<_, _>, key: &str, name| -> Result<()> {
            let value = self.fetch_value(name)?;

            if let Some(value) = value {
                map.insert(key.to_string(), value);
            }

            Ok(())
        };

        add_value(
            &mut out,
            "CLOUDSTACK_AVAILABILITY_ZONE",
            "availability_zone",
        )?;
        add_value(&mut out, "CLOUDSTACK_CLOUD_IDENTIFIER", "cloud_identifier")?;
        add_value(&mut out, "CLOUDSTACK_INSTANCE_ID", "instance_id")?;
        add_value(&mut out, "CLOUDSTACK_LOCAL_HOSTNAME", "local_hostname")?;
        add_value(&mut out, "CLOUDSTACK_SERVICE_OFFERING", "service_offering")?;
        add_value(&mut out, "CLOUDSTACK_VM_ID", "vm_id")?;

        Ok(out)
    }

    fn hostname(&self) -> Result<Option<String>> {
        Ok(None)
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

impl Drop for ConfigDrive {
    fn drop(&mut self) {
        if self.temp_dir.is_some() {
            if let Err(e) = crate::util::unmount(&self.metadata_dir(), 3) {
                error!("failed to cleanup CloudStack config-drive: {}", e);
            };
        }
    }
}
