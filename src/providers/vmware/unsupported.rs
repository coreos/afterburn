//! VMware provider on unsupported architectures.

use super::VmwareProvider;
use anyhow::{bail, Result};

impl VmwareProvider {
    pub fn try_new() -> Result<Self> {
        bail!("unsupported architecture");
    }

    pub fn parse_netplan_config(&self) -> Result<Option<String>> {
        bail!("unsupported architecture");
    }
}
