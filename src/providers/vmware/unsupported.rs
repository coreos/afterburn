//! VMware provider on unsupported architectures.

use super::VmwareProvider;
use crate::errors::*;
use error_chain::bail;

impl VmwareProvider {
    pub fn try_new() -> Result<Self> {
        bail!("unsupported architecture");
    }
}
