//! configdrive metadata fetcher for cloudstack

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use nix::mount;
use openssh_keys::PublicKey;
use tempdir::TempDir;

use errors::*;
use network;
use providers::MetadataProvider;

const CONFIG_DRIVE_LABEL_1: &str = "config-2";
const CONFIG_DRIVE_LABEL_2: &str = "CONFIG-2";

#[derive(Debug)]
pub struct ConfigDrive {
    temp_dir: Option<TempDir>,
    target: PathBuf,
    path: PathBuf,
}

impl ConfigDrive {
    pub fn try_new() -> Result<Self> {
        // maybe its already mounted
        let path = Path::new("/media/ConfigDrive/cloudstack/metadata/");
        if path.exists() {
            return Ok(ConfigDrive {
                temp_dir: None,
                path: path.to_owned(),
                target: path.to_owned(),
            });
        }

        // if not try and mount with each of the labels
        let target =
            TempDir::new("coreos-metadata").chain_err(|| "failed to create temporary directory")?;
        ConfigDrive::mount_ro(
            &Path::new("/dev/disk/by-label/").join(CONFIG_DRIVE_LABEL_1),
            target.path(),
            "iso9660",
        )
        .or_else(|_| {
            ConfigDrive::mount_ro(
                &Path::new("/dev/disk/by-label/").join(CONFIG_DRIVE_LABEL_2),
                target.path(),
                "iso9660",
            )
        })?;

        Ok(ConfigDrive {
            path: target.path().join("cloudstack").join("metadata"),
            target: target.path().to_owned(),
            temp_dir: Some(target),
        })
    }

    fn fetch_value(&self, key: &str) -> Result<Option<String>> {
        let filename = self.path.join(format!("{}.txt", key));

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
        let filename = self.path.join("public_keys.txt");
        let file =
            File::open(&filename).chain_err(|| format!("failed to open file '{:?}'", filename))?;

        PublicKey::read_keys(file).chain_err(|| "failed to read public keys from config drive file")
    }

    fn mount_ro(source: &Path, target: &Path, fstype: &str) -> Result<()> {
        mount::mount(
            Some(source),
            target,
            Some(fstype),
            mount::MsFlags::MS_RDONLY,
            None::<&str>,
        )
        .chain_err(|| {
            format!(
                "failed to read-only mount source '{:?}' to target '{:?}' with filetype '{}'",
                source, target, fstype
            )
        })
    }

    fn unmount(target: &Path) -> Result<()> {
        mount::umount(target).chain_err(|| format!("failed to unmount target '{:?}'", target))
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

    fn network_devices(&self) -> Result<Vec<network::Device>> {
        Ok(vec![])
    }

    fn boot_checkin(&self) -> Result<()> {
        warn!("boot check-in requested, but not supported on this platform");
        Ok(())
    }
}

impl ::std::ops::Drop for ConfigDrive {
    fn drop(&mut self) {
        if self.temp_dir.is_some() {
            ConfigDrive::unmount(&self.path).unwrap();
        }
    }
}
