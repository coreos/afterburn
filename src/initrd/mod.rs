//! Agent logic running at early boot.
//!
//! This is run early-on in initrd, possibly before networking and other
//! services are configured, so it may not be able to use all usual metadata
//! fetcher.

use crate::providers::vmware::VmwareProvider;
use crate::providers::MetadataProvider;
use anyhow::{Context, Result};
use std::fs::File;
use std::io::Write;

/// Path to cmdline.d fragment for network kernel arguments.
static KARGS_PATH: &str = "/etc/cmdline.d/50-afterburn-network-kargs.conf";

/// Fetch network kargs for the given provider.
pub(crate) fn fetch_network_kargs(provider: &str) -> Result<Option<String>> {
    match provider {
        "vmware" => VmwareProvider::try_new()?.rd_network_kargs(),
        _ => Ok(None),
    }
}

/// Write network kargs into a cmdline.d fragment.
pub(crate) fn write_network_kargs(kargs: &str) -> Result<()> {
    let mut fragment_file = File::create(KARGS_PATH)
        .with_context(|| format!("failed to create file {KARGS_PATH:?}"))?;

    fragment_file
        .write_all(kargs.as_bytes())
        .context("failed to write network arguments fragment")?;
    fragment_file
        .write_all(&[b'\n'])
        .context("failed to write trailing newline")?;

    Ok(())
}
