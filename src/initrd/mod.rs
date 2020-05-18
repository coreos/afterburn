/// Agent logic running at early boot.
///
/// This is run early-on in initrd, possibly before networking and other
/// services are configured, so it may not be able to use all usual metadata
/// fetcher.
use crate::errors::*;

use std::fs::File;
use std::io::Write;

/// Path to cmdline.d fragment for network kernel arguments.
static KARGS_PATH: &str = "/etc/cmdline.d/50-afterburn-network-kargs.conf";

/// Fetch network kargs for the given provider.
pub(crate) fn fetch_network_kargs(provider: &str) -> Result<Option<String>> {
    match provider {
        // TODO(lucab): wire-in vmware guestinfo logic.
        "vmware" => Ok(None),
        _ => Ok(None),
    }
}

/// Write network kargs into a cmdline.d fragment.
pub(crate) fn write_network_kargs(kargs: &str) -> Result<()> {
    let mut fragment_file =
        File::create(KARGS_PATH).chain_err(|| format!("failed to create file {:?}", KARGS_PATH))?;

    fragment_file
        .write_all(kargs.as_bytes())
        .chain_err(|| "failed to write network arguments fragment")?;

    Ok(())
}
