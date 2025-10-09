use super::KubeVirtCloudConfig;
use crate::{network, providers::MetadataProvider};
use anyhow::{Context, Result};
use openssh_keys::PublicKey;
use slog_scope::error;
use std::{collections::HashMap, path::Path, process::Command};
use tempfile::TempDir;

const TARGET_FS: &str = "iso9660";

#[derive(Debug, Clone, Copy)]
pub enum NetworkConfigurationFormat {
    ConfigDrive, // config-2 label with OpenStack network data format
    NoCloud,     // cidata label with cloud-init NoCloud network-config format
}

#[derive(Debug)]
pub struct KubeVirtProvider {
    mount_dir: TempDir,
    config: KubeVirtCloudConfig,
}

impl KubeVirtProvider {
    fn find_config_device() -> Option<(String, NetworkConfigurationFormat)> {
        // Try config-2 first (OpenStack ConfigDrive)
        let output = Command::new("blkid")
            .args(["--cache-file", "/dev/null", "-L", "config-2"])
            .output()
            .ok()?;

        if output.status.success() {
            return Some((
                String::from_utf8_lossy(&output.stdout).trim().to_string(),
                NetworkConfigurationFormat::ConfigDrive,
            ));
        }

        // Try cidata (NoCloud)
        let output = Command::new("blkid")
            .args(["--cache-file", "/dev/null", "-L", "cidata"])
            .output()
            .ok()?;

        if output.status.success() {
            return Some((
                String::from_utf8_lossy(&output.stdout).trim().to_string(),
                NetworkConfigurationFormat::NoCloud,
            ));
        }

        None
    }

    pub fn try_new() -> Result<Option<Self>> {
        let mount_dir = tempfile::Builder::new()
            .prefix("afterburn-")
            .tempdir()
            .context("failed to create temporary directory")?;

        let (device_path, format) = match Self::find_config_device() {
            Some(result) => result,
            None => return Ok(None),
        };

        crate::util::mount_ro(Path::new(&device_path), mount_dir.path(), TARGET_FS, 3)?;

        let config = KubeVirtCloudConfig::try_new(mount_dir.path(), format)
            .context("failed to read KubeVirt cloud config")?;

        Ok(Some(Self { config, mount_dir }))
    }
}

impl MetadataProvider for KubeVirtProvider {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        self.config.attributes()
    }

    fn hostname(&self) -> Result<Option<String>> {
        self.config.hostname()
    }

    fn ssh_keys(&self) -> Result<Vec<PublicKey>> {
        self.config.ssh_keys()
    }

    fn networks(&self) -> Result<Vec<network::Interface>> {
        self.config.networks()
    }

    fn rd_network_kargs(&self) -> Result<Option<String>> {
        self.config.rd_network_kargs()
    }

    fn virtual_network_devices(&self) -> Result<Vec<network::VirtualNetDev>> {
        self.config.virtual_network_devices()
    }

    fn boot_checkin(&self) -> Result<()> {
        self.config.boot_checkin()
    }
}

impl Drop for KubeVirtProvider {
    fn drop(&mut self) {
        if let Err(e) = crate::util::unmount(self.mount_dir.path(), 3) {
            error!("failed to cleanup KubeVirt config device: {:?}", e);
        };
    }
}
