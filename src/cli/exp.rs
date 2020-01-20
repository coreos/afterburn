//! `exp` CLI sub-command.

use crate::errors::*;
use crate::{initrd, util};
use clap::ArgMatches;
use error_chain::bail;

/// Experimental subcommands.
#[derive(Debug)]
pub enum CliExp {
    RdNetworkKargs(CliRdNetworkKargs),
}

impl CliExp {
    /// Parse sub-command into configuration.
    pub(crate) fn parse(app_matches: &ArgMatches) -> Result<super::CliConfig> {
        if app_matches.subcommand_name().is_none() {
            bail!("missing subcommand for 'exp'");
        }

        let cfg = match app_matches.subcommand() {
            ("rd-network-kargs", Some(matches)) => CliRdNetworkKargs::parse(matches)?,
            (x, _) => unreachable!("unrecognized subcommand for 'exp': '{}'", x),
        };

        Ok(super::CliConfig::Exp(cfg))
    }

    // Run sub-command.
    pub(crate) fn run(&self) -> Result<()> {
        match self {
            CliExp::RdNetworkKargs(cmd) => cmd.run()?,
        };
        Ok(())
    }
}

/// Sub-command for network kernel arguments.
#[derive(Debug)]
pub struct CliRdNetworkKargs {
    platform: String,
    default_kargs: String,
}

impl CliRdNetworkKargs {
    /// Parse sub-command into configuration.
    pub(crate) fn parse(matches: &ArgMatches) -> Result<CliExp> {
        let platform = super::parse_provider(matches)?;
        let default_kargs = matches
            .value_of("default-value")
            .ok_or_else(|| "missing network kargs default value")?
            .to_string();

        let cfg = Self {
            platform,
            default_kargs,
        };
        Ok(CliExp::RdNetworkKargs(cfg))
    }

    /// Run the sub-command.
    pub(crate) fn run(&self) -> Result<()> {
        if util::has_network_kargs(super::CMDLINE_PATH)? {
            slog_scope::warn!("kernel cmdline already specifies network arguments, skipping");
            return Ok(());
        };

        let provider_kargs = initrd::fetch_network_kargs(&self.platform)?;
        let kargs = provider_kargs
            .as_ref()
            .unwrap_or_else(|| &self.default_kargs);
        initrd::write_network_kargs(kargs)
    }
}
