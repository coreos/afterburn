//! VMware provider on x86_64.
//!
//! This uses the guest->host backdoor protocol for introspection.

use super::VmwareProvider;
use crate::errors::*;
use error_chain::bail;

/// Guestinfo key for network kargs.
static INITRD_NET_KARGS: &str = "guestinfo.afterburn.initrd.network-kargs";

impl VmwareProvider {
    /// Build the VMware provider, fetching and caching guestinfo entries.
    pub fn try_new() -> Result<Self> {
        if !vmw_backdoor::is_vmware_cpu() {
            bail!("not running on VMWare CPU");
        }

        // NOTE(lucab): privileged mode is in theory more reliable but
        //  `kernel_lockdown(7)` may block it due to `iopl()` usage.
        //  Thus, we try that first and fall back if kernel blocks it.
        let mut backdoor = vmw_backdoor::probe_backdoor_privileged().or_else(|e| {
            slog_scope::warn!("failed to probe backdoor in privileged mode: {}", e);
            slog_scope::warn!("falling back to unprivileged backdoor access");
            vmw_backdoor::probe_backdoor()
        })?;

        let mut erpc = backdoor.open_enhanced_chan()?;
        let guestinfo_net_kargs = Self::fetch_guestinfo(&mut erpc, INITRD_NET_KARGS)?;

        let provider = Self {
            guestinfo_net_kargs,
        };

        slog_scope::trace!("cached vmware provider: {:?}", provider);
        Ok(provider)
    }

    /// Retrieve the value of a guestinfo string property, by key.
    fn fetch_guestinfo(erpc: &mut vmw_backdoor::EnhancedChan, key: &str) -> Result<Option<String>> {
        let guestinfo = erpc
            .get_guestinfo(key.as_bytes())
            .chain_err(|| "failed to retrieve network kargs")?
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned());
        Ok(guestinfo)
    }
}
