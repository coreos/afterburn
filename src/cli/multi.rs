//! `multi` CLI sub-command.

use crate::metadata;
use anyhow::{Context, Result};
use clap::{ArgGroup, Parser};

/// Perform multiple tasks in a single call
#[derive(Debug, Parser)]
#[command(group(ArgGroup::new("provider-group").args(["cmdline", "provider"]).required(true)))]
pub struct CliMulti {
    /// The name of the cloud provider
    #[arg(long, value_name = "name")]
    provider: Option<String>,
    /// Read the cloud provider from the kernel cmdline
    #[arg(long)]
    cmdline: bool,
    /// The file into which the metadata attributes are written
    #[arg(long = "attributes", value_name = "path")]
    attributes_file: Option<String>,
    /// Check-in this instance boot with the cloud provider
    #[arg(long)]
    check_in: bool,
    /// The file into which the hostname should be written
    #[arg(long = "hostname", value_name = "path")]
    hostname_file: Option<String>,
    /// The directory into which network units are written
    #[arg(long = "network-units", value_name = "path")]
    network_units_dir: Option<String>,
    /// Update SSH keys for the given user
    #[arg(long = "ssh-keys", value_name = "username")]
    ssh_keys_user: Option<String>,
    /// Whether this command was translated from legacy CLI args
    #[arg(long, hide = true)]
    legacy_cli: bool,
}

impl CliMulti {
    /// Run the `multi` sub-command.
    pub(crate) fn run(self) -> Result<()> {
        let provider = super::get_provider(self.provider.as_deref())?;

        if self.attributes_file.is_none()
            && self.network_units_dir.is_none()
            && !self.check_in
            && self.ssh_keys_user.is_none()
            && self.hostname_file.is_none()
        {
            slog_scope::warn!("multi: no action specified");
        }

        // fetch the metadata from the configured provider
        let metadata =
            metadata::fetch_metadata(&provider).context("fetching metadata from provider")?;

        // write attributes if configured to do so
        self.attributes_file
            .map_or(Ok(()), |x| metadata.write_attributes(x))
            .context("writing metadata attributes")?;

        // write ssh keys if configured to do so
        self.ssh_keys_user
            .map_or(Ok(()), |x| metadata.write_ssh_keys(x))
            .context("writing ssh keys")?;

        // write hostname if configured to do so
        self.hostname_file
            .map_or(Ok(()), |x| metadata.write_hostname(x))
            .context("writing hostname")?;

        // write network units if configured to do so
        self.network_units_dir
            .map_or(Ok(()), |x| metadata.write_network_units(x))
            .context("writing network units")?;

        // perform boot check-in.
        if self.check_in {
            metadata
                .boot_checkin()
                .context("checking-in instance boot to cloud provider")?;
        }

        Ok(())
    }
}
