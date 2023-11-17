//! VMware provider.

use std::collections::HashMap;

use anyhow::Result;

use crate::providers::MetadataProvider;

/// VMware provider.
#[derive(Clone, Debug)]
pub struct VmwareProvider {
    /// External network kargs for initrd.
    guestinfo_net_kargs: Option<String>,
    /// Cloud-Init metadata for netplan YAML
    guestinfo_metadata: Option<String>,
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

    fn rd_network_kargs(&self) -> Result<Option<String>> {
        Ok(self.guestinfo_net_kargs.clone())
    }

    fn netplan_config(&self) -> Result<Option<String>> {
        self.parse_netplan_config()
    }
}
