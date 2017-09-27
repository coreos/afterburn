//! configdrive metadata fetcher for cloudstack

use errors::*;
use metadata::Metadata;
use nix::mount;
use openssh_keys::PublicKey;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;
use tempdir::TempDir;

const CONFIG_DRIVE_LABEL_1: &'static str = "config-2";
const CONFIG_DRIVE_LABEL_2: &'static str = "CONFIG-2";

#[derive(Debug)]
struct ConfigDrive {
    temp_dir: Option<TempDir>,
    target: PathBuf,
    path: PathBuf,
}

impl ConfigDrive {
    fn new() -> Result<Self> {
        // maybe its already mounted
        let path = Path::new("/media/ConfigDrive/cloudstack/metadata/");
        if path.exists() {
            return Ok(ConfigDrive {
                temp_dir: None,
                path: path.to_owned(),
                target: path.to_owned(),
            })
        }

        // if not try and mount with each of the labels
        let target = TempDir::new("coreos-metadata")
            .chain_err(|| "failed to create temporary directory")?;
        mount_ro(&Path::new("/dev/disk/by-label/").join(CONFIG_DRIVE_LABEL_1), target.path(), "iso9660")
            .or_else(|_| mount_ro(&Path::new("/dev/disk/by-label/").join(CONFIG_DRIVE_LABEL_2), target.path(), "iso9660"))?;

        Ok(ConfigDrive {
            path: target.path().join("cloudstack").join("metadata"),
            target: target.path().to_owned(),
            temp_dir: Some(target),
        })
    }

    fn fetch_value(&self, key: &str) -> Result<Option<String>> {
        let filename = self.path.join(format!("{}.txt", key));

        if !filename.exists() {
            return Ok(None)
        }

        let mut file = File::open(&filename)
            .chain_err(|| format!("failed to open file '{:?}'", filename))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .chain_err(|| format!("failed to read from file '{:?}'", filename))?;

        Ok(Some(contents))
    }

    fn fetch_publickeys(&self) -> Result<Vec<PublicKey>> {
        let filename = self.path.join("public_keys.txt");
        let file = File::open(&filename)
            .chain_err(|| format!("failed to open file '{:?}'", filename))?;

        PublicKey::read_keys(file)
            .chain_err(|| "failed to read public keys from config drive file")
    }
}

impl ::std::ops::Drop for ConfigDrive {
    fn drop(&mut self) {
        if self.temp_dir.is_some() {
            unmount(&self.path).unwrap();
        }
    }
}

fn mount_ro(source: &Path, target: &Path, fstype: &str) -> Result<()> {
    mount::mount(Some(source), target, Some(fstype), mount::MS_RDONLY, None::<&str>)
        .chain_err(|| format!("failed to read-only mount source '{:?}' to target '{:?}' with filetype '{}'", source, target, fstype))
}

fn unmount(target: &Path) -> Result<()> {
    mount::umount(target)
        .chain_err(|| format!("failed to unmount target '{:?}'", target))
}

pub fn fetch_metadata() -> Result<Metadata> {
    let drive = ConfigDrive::new()?;

    Ok(Metadata::builder()
       .add_publickeys(drive.fetch_publickeys()?)
       .add_attribute_if_exists("CLOUDSTACK_AVAILABILITY_ZONE".into(), drive.fetch_value("availability_zone")?)
       .add_attribute_if_exists("CLOUDSTACK_INSTANCE_ID".into(), drive.fetch_value("instance_id")?)
       .add_attribute_if_exists("CLOUDSTACK_SERVICE_OFFERING".into(), drive.fetch_value("service_offering")?)
       .add_attribute_if_exists("CLOUDSTACK_CLOUD_IDENTIFIER".into(), drive.fetch_value("cloud_identifier")?)
       .add_attribute_if_exists("CLOUDSTACK_LOCAL_HOSTNAME".into(), drive.fetch_value("local_hostname")?)
       .add_attribute_if_exists("CLOUDSTACK_VM_ID".into(), drive.fetch_value("vm_id")?)
       .build())
}
