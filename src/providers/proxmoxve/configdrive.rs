use super::ProxmoxVECloudConfig;
use crate::{network, providers::MetadataProvider};
use anyhow::{Context, Result};
use openssh_keys::PublicKey;
use slog_scope::error;
use std::{collections::HashMap, path::Path, process::Command};
use tempfile::TempDir;

const CONFIG_DRIVE_LABEL: &str = "cidata";
const TARGET_FS: &str = "iso9660";

#[derive(Debug)]
pub struct ProxmoxVEConfigDrive {
    mount_dir: TempDir,
    config: ProxmoxVECloudConfig,
}

impl ProxmoxVEConfigDrive {
    fn find_cidata_device() -> Option<String> {
        let output = Command::new("blkid")
            .args(["--cache-file", "/dev/null", "-L", CONFIG_DRIVE_LABEL])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    pub fn try_new() -> Result<Self> {
        let mount_dir = tempfile::Builder::new()
            .prefix("afterburn-")
            .tempdir()
            .context("failed to create temporary directory")?;

        let device_path = Self::find_cidata_device()
            .ok_or_else(|| anyhow::anyhow!("could not find cidata device"))?;

        crate::util::mount_ro(Path::new(&device_path), mount_dir.path(), TARGET_FS, 3)?;

        let config = ProxmoxVECloudConfig::try_new(mount_dir.path())
            .context("failed to read ProxmoxVE cloud config")?;

        Ok(Self { config, mount_dir })
    }
}

impl MetadataProvider for ProxmoxVEConfigDrive {
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

    fn netplan_config(&self) -> Result<Option<String>> {
        self.config.netplan_config()
    }
}

impl Drop for ProxmoxVEConfigDrive {
    fn drop(&mut self) {
        if let Err(e) = crate::util::unmount(self.mount_dir.path(), 3) {
            error!("failed to cleanup Proxmox VE config-drive: {:?}", e);
        };
    }
}
