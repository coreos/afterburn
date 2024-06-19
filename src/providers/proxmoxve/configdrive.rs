use super::ProxmoxVECloudConfig;
use crate::{network, providers::MetadataProvider};
use anyhow::{Context, Result};
use openssh_keys::PublicKey;
use slog_scope::error;
use std::{collections::HashMap, path::Path};
use tempfile::TempDir;

const CONFIG_DRIVE_LABEL: &str = "cidata";
const TARGET_FS: &str = "iso9660";

#[derive(Debug)]
pub struct ProxmoxVEConfigDrive {
    mount_dir: TempDir,
    config: ProxmoxVECloudConfig,
}

impl ProxmoxVEConfigDrive {
    pub fn try_new() -> Result<Self> {
        let mount_dir = tempfile::Builder::new()
            .prefix("afterburn-")
            .tempdir()
            .context("failed to create temporary directory")?;

        crate::util::mount_ro(
            &Path::new("/dev/disk/by-label/").join(CONFIG_DRIVE_LABEL),
            mount_dir.path(),
            TARGET_FS,
            3,
        )?;

        Ok(Self {
            config: ProxmoxVECloudConfig::try_new(mount_dir.path())?,
            mount_dir,
        })
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
}

impl Drop for ProxmoxVEConfigDrive {
    fn drop(&mut self) {
        if let Err(e) = crate::util::unmount(self.mount_dir.path(), 3) {
            error!("failed to cleanup Proxmox VE config-drive: {:?}", e);
        };
    }
}
