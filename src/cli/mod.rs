//! Command-line arguments parsing.

use anyhow::{bail, Result};
use clap::{self, crate_version, Arg, ArgMatches, Command};
use slog_scope::trace;

mod exp;
mod multi;

/// Path to kernel command-line (requires procfs mount).
const CMDLINE_PATH: &str = "/proc/cmdline";

/// CLI sub-commands configuration.
#[derive(Debug)]
pub(crate) enum CliConfig {
    Multi(multi::CliMulti),
    Exp(exp::CliExp),
}

impl CliConfig {
    /// Parse CLI sub-commands into configuration.
    pub fn parse_subcommands(app_matches: ArgMatches) -> Result<Self> {
        let cfg = match app_matches.subcommand().expect("no subcommand") {
            ("multi", matches) => multi::CliMulti::parse(matches)?,
            ("exp", matches) => exp::CliExp::parse(matches)?,
            (x, _) => unreachable!("unrecognized subcommand '{}'", x),
        };

        Ok(cfg)
    }

    /// Run the relevant CLI sub-command.
    pub fn run(self) -> Result<()> {
        match self {
            CliConfig::Multi(cmd) => cmd.run(),
            CliConfig::Exp(cmd) => cmd.run(),
        }
    }
}

/// Parse command-line arguments into CLI configuration.
pub(crate) fn parse_args(argv: impl IntoIterator<Item = String>) -> Result<CliConfig> {
    let args = translate_legacy_args(argv);
    let matches = match cli_setup().try_get_matches_from(args) {
        Err(e) if e.kind() == clap::ErrorKind::DisplayHelp => e.exit(),
        Err(e) if e.kind() == clap::ErrorKind::DisplayVersion => e.exit(),
        v => v,
    }?;

    let cfg = CliConfig::parse_subcommands(matches)?;
    trace!("cli configuration - {:?}", cfg);
    Ok(cfg)
}

/// Parse provider ID from flag or kargs.
fn parse_provider(matches: &clap::ArgMatches) -> Result<String> {
    let provider = match (matches.value_of("provider"), matches.is_present("cmdline")) {
        (Some(provider), false) => String::from(provider),
        (None, true) => crate::util::get_platform(CMDLINE_PATH)?,
        (None, false) => bail!("must set either --provider or --cmdline"),
        (Some(_), true) => bail!("cannot process both --provider and --cmdline"),
    };

    Ok(provider)
}

/// CLI setup, covering all sub-commands and arguments.
fn cli_setup<'a>() -> Command<'a> {
    // NOTE(lucab): due to legacy translation there can't be global arguments
    //  here, i.e. a sub-command is always expected first.
    Command::new("Afterburn")
        .version(crate_version!())
        .propagate_version(true)
        .subcommand(
            Command::new("multi")
                .about("Perform multiple tasks in a single call")
                .arg(
                    Arg::new("legacy-cli")
                        .long("legacy-cli")
                        .help("Whether this command was translated from legacy CLI args")
                        .hide(true),
                )
                .arg(
                    Arg::new("provider")
                        .long("provider")
                        .help("The name of the cloud provider")
                        .global(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::new("cmdline")
                        .long("cmdline")
                        .global(true)
                        .help("Read the cloud provider from the kernel cmdline"),
                )
                .arg(
                    Arg::new("attributes")
                        .long("attributes")
                        .help("The file into which the metadata attributes are written")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("check-in")
                        .long("check-in")
                        .help("Check-in this instance boot with the cloud provider"),
                )
                .arg(
                    Arg::new("hostname")
                        .long("hostname")
                        .help("The file into which the hostname should be written")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("network-units")
                        .long("network-units")
                        .help("The directory into which network units are written")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("ssh-keys")
                        .long("ssh-keys")
                        .help("Update SSH keys for the given user")
                        .takes_value(true),
                ),
        )
        .subcommand(
            Command::new("exp")
                .about("experimental subcommands")
                .subcommand_required(true)
                .subcommand(
                    Command::new("rd-network-kargs")
                        .about("Supplement initrd with network configuration kargs")
                        .arg(
                            Arg::new("cmdline")
                                .long("cmdline")
                                .global(true)
                                .help("Read the cloud provider from the kernel cmdline"),
                        )
                        .arg(
                            Arg::new("provider")
                                .long("provider")
                                .help("The name of the cloud provider")
                                .global(true)
                                .takes_value(true),
                        )
                        .arg(
                            Arg::new("default-value")
                                .long("default-value")
                                .help("Default value for network kargs fallback")
                                .required(true)
                                .takes_value(true),
                        ),
                ),
        )
}

/// Translate command-line arguments from legacy mode.
///
/// In legacy mode there are no sub-commands, and single-dash (Golang-style)
/// arguments are allowed too.
fn translate_legacy_args(cli: impl IntoIterator<Item = String>) -> impl Iterator<Item = String> {
    // Process the first two arguments and check whether there is a sub-command (normal mode)
    // or not (legacy mode).
    let mut argv = cli.into_iter();
    let argv_0 = argv.next().unwrap_or_else(|| "afterburn".to_string());
    let argv_1 = argv.next();
    let legacy_mode = match argv_1 {
        Some(ref arg) => arg.starts_with('-'),
        None => true,
    };

    // Inject back the first two arguments, plus the `multi` sub-command with a legacy marker.
    let mut new_argv = vec![argv_0];
    if let Some(arg) = argv_1 {
        new_argv.push(arg);
    }
    if legacy_mode {
        new_argv.insert(1, "multi".to_string());
        new_argv.insert(2, "--legacy-cli".to_string());
    }
    let argv = new_argv.into_iter().chain(argv);

    // Do some pre-processing on the command line arguments so that legacy
    // Go-style arguments are supported for backwards compatibility.
    argv.map(move |arg| {
        if legacy_mode && arg.starts_with('-') && !arg.starts_with("--") && arg.len() > 2 {
            format!("-{arg}")
        } else {
            arg
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clap_tests() {
        cli_setup().debug_assert();
    }

    #[test]
    fn test_translate_legacy_args() {
        let legacy: Vec<_> = ["afterburn", "-ssh-keys"]
            .iter()
            .map(ToString::to_string)
            .collect();

        let translated: Vec<_> = translate_legacy_args(legacy).collect();
        assert_eq!(translated[0], "afterburn".to_string());
        assert_eq!(translated[1], "multi".to_string());
        assert_eq!(translated[2], "--legacy-cli".to_string());
        assert_eq!(translated[3], "--ssh-keys".to_string());
        assert_eq!(translated.len(), 4);
    }

    #[test]
    fn test_legacy_no_action() {
        let legacy: Vec<_> = ["afterburn", "--provider", "azure"]
            .iter()
            .map(ToString::to_string)
            .collect();

        let cmd = parse_args(legacy).unwrap();
        match cmd {
            CliConfig::Multi(_) => {}
            x => panic!("unexpected cmd: {x:?}"),
        };
    }

    #[test]
    fn test_no_args() {
        let args = vec!["afterburn".to_string()];
        parse_args(args).unwrap_err();
    }

    #[test]
    fn test_basic_cli_args() {
        let args: Vec<_> = ["afterburn", "--provider", "azure", "--check-in"]
            .iter()
            .map(ToString::to_string)
            .collect();

        let cmd = parse_args(args).unwrap();
        match cmd {
            CliConfig::Multi(_) => {}
            x => panic!("unexpected cmd: {x:?}"),
        };
    }

    #[test]
    fn test_multi_cmd() {
        let args: Vec<_> = ["afterburn", "multi", "--provider", "azure", "--check-in"]
            .iter()
            .map(ToString::to_string)
            .collect();

        let cmd = parse_args(args).unwrap();
        match cmd {
            CliConfig::Multi(_) => {}
            x => panic!("unexpected cmd: {x:?}"),
        };
    }

    #[test]
    fn test_exp_cmd() {
        let args: Vec<_> = [
            "afterburn",
            "exp",
            "rd-network-kargs",
            "--provider",
            "gcp",
            "--default-value",
            "ip=dhcp",
        ]
        .iter()
        .map(ToString::to_string)
        .collect();

        let cmd = parse_args(args).unwrap();
        let subcmd = match cmd {
            CliConfig::Exp(v) => v,
            x => panic!("unexpected cmd: {x:?}"),
        };

        match subcmd {
            exp::CliExp::RdNetworkKargs(_) => {}
            #[allow(unreachable_patterns)]
            x => panic!("unexpected 'exp' sub-command: {x:?}"),
        };
    }

    #[test]
    fn test_default_net_kargs() {
        // Missing flag.
        let t1: Vec<_> = ["afterburn", "exp", "rd-network-kargs", "--provider", "gcp"]
            .iter()
            .map(ToString::to_string)
            .collect();

        // Missing flag value.
        let t2: Vec<_> = [
            "afterburn",
            "exp",
            "rd-network-kargs",
            "--provider",
            "gcp",
            "--default-value",
        ]
        .iter()
        .map(ToString::to_string)
        .collect();

        for args in vec![t1, t2] {
            let input = format!("{args:?}");
            parse_args(args).expect_err(&input);
        }

        // Empty flag value.
        let t3: Vec<_> = [
            "afterburn",
            "exp",
            "rd-network-kargs",
            "--provider",
            "gcp",
            "--default-value",
            "",
        ]
        .iter()
        .map(ToString::to_string)
        .collect();

        parse_args(t3).unwrap();
    }
}
