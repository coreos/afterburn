//! `multi` CLI sub-command.

use crate::metadata;
use anyhow::{Context, Result};

#[derive(Debug)]
pub struct CliMulti {
    attributes_file: Option<String>,
    check_in: bool,
    hostname_file: Option<String>,
    network_units_dir: Option<String>,
    provider: String,
    ssh_keys_user: Option<String>,
}

impl CliMulti {
    /// Parse flags for the `multi` sub-command.
    pub(crate) fn parse(matches: &clap::ArgMatches) -> Result<super::CliConfig> {
        let provider = super::parse_provider(matches)?;

        let multi = Self {
            attributes_file: matches.get_one::<String>("attributes").cloned(),
            check_in: matches.get_flag("check-in"),
            hostname_file: matches.get_one::<String>("hostname").cloned(),
            network_units_dir: matches.get_one::<String>("network-units").cloned(),
            provider,
            ssh_keys_user: matches.get_one::<String>("ssh-keys").cloned(),
        };

        if multi.attributes_file.is_none()
            && multi.network_units_dir.is_none()
            && !multi.check_in
            && multi.ssh_keys_user.is_none()
            && multi.hostname_file.is_none()
            && multi.network_units_dir.is_none()
        {
            slog_scope::warn!("multi: no action specified");
        }

        Ok(super::CliConfig::Multi(multi))
    }

    /// Run the `multi` sub-command.
    pub(crate) fn run(self) -> Result<()> {
        // fetch the metadata from the configured provider
        let metadata =
            metadata::fetch_metadata(&self.provider).context("fetching metadata from provider")?;

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
