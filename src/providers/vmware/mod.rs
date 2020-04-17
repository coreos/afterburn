//! VMware provider.
//!
//! This uses the guest->host backdoor protocol for introspection.

use std::collections::HashMap;

use error_chain::bail;
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

impl VmwareProvider {
    pub fn try_new() -> Result<Self> {
        if !vmw_backdoor::is_vmware_cpu() {
            bail!("not running on VMWare CPU");
        }

        let mut backdoor = vmw_backdoor::probe_backdoor()?;
        let mut erpc = backdoor.open_enhanced_chan()?;
        let guestinfo_net_kargs = Self::get_net_kargs(&mut erpc)?;

        let provider = Self {
            guestinfo_net_kargs,
        };
        Ok(provider)
    }

    fn get_net_kargs(_erpc: &mut vmw_backdoor::EnhancedChan) -> Result<Option<String>> {
        // TODO(lucab): pick a stable key name and implement this logic.
        Ok(None)
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
