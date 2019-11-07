//! Metadata fetcher for the IBMCloud provider.
//!
//! IBMCloud supports multiple kind of compute nodes, with different
//! features and peculiarities.

// TODO(lucab): this allows adding 'classic' and 'vpc-gen1' instances too
//  via auto-detection, if there is a need for that in the future.

use crate::errors::Result;
use crate::providers;

mod gen2;

/// Build a new client for IBMCloud.
///
/// This internally tries to autodetect the infrastructure type
/// and mount the relevant config-drive.
pub fn try_autodetect() -> Result<Box<dyn providers::MetadataProvider>> {
    // Auto-detection order: Gen2 (possibly later: classic, Gen1).
    slog_scope::trace!("trying to autodetect ibmcloud infrastructure type");

    match gen2::G2Provider::try_new() {
        Ok(g2) => {
            slog_scope::info!("found metadata for VPC-Gen2 instance");
            return Ok(Box::new(g2));
        }
        Err(e) => {
            slog_scope::debug!("ibmcloud VPC-Gen2 autodetection failed: {}", e);
        }
    };

    error_chain::bail!("unable to find ibmcloud metadata");
}
