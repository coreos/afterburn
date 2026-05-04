//! `render-ignition` CLI sub-command.
//!
//! Generates per-feature Ignition config fragment files in an output directory.
//! Each enabled feature writes its own `.ign` file; Ignition merges them
//! natively from `base.platform.d/<platform>/` under a system config directory
//! such as `/etc/ignition`.

use anyhow::{bail, Context, Result};
use clap::{ArgGroup, Parser};

/// Render Ignition config fragments from cloud provider metadata
#[derive(Debug, Parser)]
#[command(group(ArgGroup::new("provider-group").args(["cmdline", "provider"]).required(true)))]
pub struct CliRenderIgnition {
    /// The name of the cloud provider
    #[arg(long, value_name = "name")]
    provider: Option<String>,
    /// Read the cloud provider from the kernel cmdline
    #[arg(long)]
    cmdline: bool,
    /// Directory to write Ignition config fragment files into
    #[arg(long = "render-ignition-dir", value_name = "path")]
    render_ignition_dir: String,
    /// Include hostname in a fragment file
    #[arg(long)]
    hostname: bool,
    /// Include platform user and SSH keys in a fragment file
    #[arg(long)]
    platform_user: bool,
    /// Enable all platform extensions (implies --hostname --platform-user)
    #[arg(long)]
    platform_extensions: bool,
}

impl CliRenderIgnition {
    const SUPPORTED_PROVIDERS: &[&str] = &["azure"];

    pub(crate) fn run(self) -> Result<()> {
        let provider_id = super::get_provider(self.provider.as_deref())?;

        if !Self::SUPPORTED_PROVIDERS.contains(&provider_id.as_str()) {
            bail!(
                "render-ignition is only supported for providers {:?}, got '{}'",
                Self::SUPPORTED_PROVIDERS,
                provider_id,
            );
        }

        let hostname = self.hostname || self.platform_extensions;
        let platform_user = self.platform_user || self.platform_extensions;

        if !hostname && !platform_user {
            slog_scope::warn!("render-ignition: no features specified");
            return Ok(());
        }

        let metadata = crate::metadata::fetch_metadata(&provider_id)
            .context("fetching metadata from provider")?;

        if hostname {
            crate::providers::microsoft::azure::config::generate_hostname_fragment(
                metadata.as_ref(),
                &self.render_ignition_dir,
            )
            .context("generating hostname ignition fragment")?;
        }

        if platform_user {
            crate::providers::microsoft::azure::config::generate_user_fragment(
                metadata.as_ref(),
                &self.render_ignition_dir,
            )
            .context("generating platform-user ignition fragment")?;
        }

        Ok(())
    }
}
