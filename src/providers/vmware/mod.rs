//! VMWare provider.
//!
//! This uses the guest->host backdoor protocol for introspection.

use std::collections::HashMap;

use error_chain::bail;
use openssh_keys::PublicKey;
use slog_scope::warn;

use crate::errors::*;
use crate::network;
use crate::providers::MetadataProvider;

/// Guestinfo key for network kargs.
static KARGS_NM_INITRD: &str = "guestinfo.exp.org.freedesktop.NetworkManager.initrd";

/// Path to fragment file with network kargs.
static NET_KARGS_PATH: &str = "/etc/cmdline.d/60-afterburn-net-kargs.conf";

/// VMWare provider.
#[derive(Clone, Debug)]
pub struct VmwareProvider {
    /// Extra kargs for early network setup.
    net_kargs: Option<String>,
}

impl VmwareProvider {
    pub fn try_new() -> Result<Self> {
        if !vmw_backdoor::is_vmware_cpu() {
            bail!("not running on VMWare CPU");
        }

        let mut backdoor = vmw_backdoor::probe_backdoor()?;
        let mut erpc = backdoor.open_enhanced_chan()?;
        let net_kargs = Self::get_net_kargs(&mut erpc)?;

        let provider = Self { net_kargs };
        Ok(provider)
    }

    fn get_net_kargs(erpc: &mut vmw_backdoor::EnhancedChan) -> Result<Option<String>> {
        let guestinfo = erpc
            .get_guestinfo(KARGS_NM_INITRD.as_bytes())
            .chain_err(|| "failed to retrieve network kargs")?
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned());
        Ok(guestinfo)
    }
}

impl MetadataProvider for VmwareProvider {
    fn rd_net_kargs(&self) -> Result<()> {
        use std::io::Write;

        let kargs = match &self.net_kargs {
            Some(val) => val,
            None => return Ok(()),
        };
        let mut fp = std::fs::File::create(NET_KARGS_PATH)
            .chain_err(|| format!("failed to create {}", NET_KARGS_PATH))?;
        fp.write_all(kargs.as_bytes())?;
        fp.write(&[b'\n'])?;
        Ok(())
    }

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
