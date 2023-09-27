use super::ProxmoxCloudConfig;
use crate::{network, providers::MetadataProvider};
use anyhow::{Context, Result};
use openssh_keys::PublicKey;
use slog_scope::error;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct ProxmoxConfigDrive {
    mount_path: PathBuf,
    config: ProxmoxCloudConfig,
}

impl ProxmoxConfigDrive {
    pub fn try_new() -> Result<Self> {
        const CONFIG_DRIVE_LABEL: &str = "cidata";
        const TARGET_FS: &str = "iso9660";

        let target = tempfile::Builder::new()
            .prefix("afterburn-")
            .tempdir()
            .context("failed to create temporary directory")?;

        crate::util::mount_ro(
            &Path::new("/dev/disk/by-label/").join(CONFIG_DRIVE_LABEL),
            target.path(),
            TARGET_FS,
            3,
        )?;

        let mount_path = target.path().to_owned();
        Ok(Self {
            config: ProxmoxCloudConfig::try_new(&mount_path)?,
            mount_path,
        })
    }
}

impl MetadataProvider for ProxmoxConfigDrive {
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

impl Drop for ProxmoxConfigDrive {
    fn drop(&mut self) {
        if let Err(e) = crate::util::unmount(&self.mount_path, 3) {
            error!("failed to cleanup Proxmox config-drive: {:?}", e);
        };
    }
}
