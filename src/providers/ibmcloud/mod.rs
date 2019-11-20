//! Metadata fetcher for the IBMCloud provider.
//!
//! IBMCloud supports multiple kind of compute nodes, with different
//! features and peculiarities.

// TODO(lucab): this allows adding 'vpc-gen1' instances too via auto-detection,
//  if there is a need for that in the future.

use crate::errors::Result;
use crate::providers;

mod classic;
mod gen2;

/// Build a new client for IBMCloud.
///
/// This internally tries to autodetect the infrastructure type
/// and mount the relevant config-drive.
/// Auto-detection order: Gen2, classic (possibly later: Gen1).
pub fn try_autodetect() -> Result<Box<dyn providers::MetadataProvider>> {
    slog_scope::trace!("trying to autodetect ibmcloud infrastructure type");

    // TODO(lucab): make auto-detection parallel (and cancellable), otherwise
    //  Classic may suffer some noticeable delay.

    // First try: Gen2.
    match gen2::G2Provider::try_new() {
        Ok(g2) => {
            slog_scope::info!("found metadata for VPC-Gen2 instance");
            return Ok(Box::new(g2));
        }
        Err(e) => {
            slog_scope::debug!("ibmcloud VPC-Gen2 autodetection failed: {}", e);
        }
    };

    // Second try: Classic.
    match classic::ClassicProvider::try_new() {
        Ok(classic) => {
            slog_scope::info!("found metadata for Classic instance");
            return Ok(Box::new(classic));
        }
        Err(e) => {
            slog_scope::debug!("ibmcloud Classic autodetection failed: {}", e);
        }
    };

    error_chain::bail!("unable to find ibmcloud metadata");
}
