//! `exp` CLI sub-command.

use crate::{initrd, util};
use anyhow::Result;
use clap::{ArgGroup, Parser};

/// Experimental subcommands
#[derive(Debug, Parser)]
pub enum CliExp {
    RdNetworkKargs(CliRdNetworkKargs),
}

impl CliExp {
    // Run sub-command.
    pub(crate) fn run(&self) -> Result<()> {
        match self {
            CliExp::RdNetworkKargs(cmd) => cmd.run()?,
        };
        Ok(())
    }
}

/// Supplement initrd with network configuration kargs
#[derive(Debug, Parser)]
#[command(group(ArgGroup::new("provider-group").args(["cmdline", "provider"]).required(true)))]
pub struct CliRdNetworkKargs {
    /// Read the cloud provider from the kernel cmdline
    #[arg(long)]
    cmdline: bool,
    /// The name of the cloud provider
    #[arg(long, value_name = "name")]
    provider: Option<String>,
    /// Default value for network kargs fallback
    #[arg(long = "default-value", value_name = "args")]
    default_kargs: String,
}

impl CliRdNetworkKargs {
    /// Run the sub-command.
    pub(crate) fn run(&self) -> Result<()> {
        let provider = super::get_provider(self.provider.as_deref())?;

        if util::has_network_kargs(super::CMDLINE_PATH)? {
            slog_scope::warn!("kernel cmdline already specifies network arguments, skipping");
            return Ok(());
        };

        let provider_kargs = initrd::fetch_network_kargs(&provider)?;
        let kargs = provider_kargs.as_ref().unwrap_or(&self.default_kargs);
        initrd::write_network_kargs(kargs)
    }
}
