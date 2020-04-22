//! VMware provider on x86_64.
//!
//! This uses the guest->host backdoor protocol for introspection.

use error_chain::bail;

use super::VmwareProvider;
use crate::errors::*;

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
