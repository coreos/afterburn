//! VMware provider.

use std::collections::HashMap;

use openssh_keys::PublicKey;
use slog_scope::warn;

use crate::errors::*;
use crate::network;
use crate::providers::MetadataProvider;

/// VMware provider.
#[derive(Clone, Debug)]
pub struct VmwareProvider {
    /// External network kargs for initrd.
    guestinfo_net_kargs: Option<String>,
}

// Architecture-specific implementation.
cfg_if::cfg_if! {
    if #[cfg(all(target_os = "linux", target_arch = "x86_64"))] {
        mod amd64;
    } else {
        mod unsupported;
    }
}

impl MetadataProvider for VmwareProvider {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        Ok(HashMap::new())
    }

    fn hostname(&self) -> Result<Option<String>> {
        Ok(None)
    }

    fn ssh_keys(&self) -> Result<Vec<PublicKey>> {
        Ok(vec![])
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
