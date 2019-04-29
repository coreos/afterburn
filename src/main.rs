// Copyright 2017 CoreOS, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate base64;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate hostname;
extern crate ipnetwork;
extern crate nix;
extern crate openssh_keys;
extern crate openssl;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate serde_xml_rs;
#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;
#[macro_use]
extern crate slog_scope;
extern crate tempdir;
extern crate tempfile;
#[cfg(feature = "cl-legacy")]
extern crate update_ssh_keys;
extern crate users;

#[cfg(test)]
extern crate mockito;

mod errors;
mod metadata;
mod network;
mod providers;
mod retry;
mod util;

use clap::{App, Arg};
use slog::Drain;
use std::env;

use crate::errors::*;
use crate::metadata::fetch_metadata;

/// Path to kernel command-line (requires procfs mount).
const CMDLINE_PATH: &str = "/proc/cmdline";

#[derive(Debug)]
struct Config {
    provider: String,
    attributes_file: Option<String>,
    check_in: bool,
    ssh_keys_user: Option<String>,
    hostname_file: Option<String>,
    network_units_dir: Option<String>,
}

quick_main!(run);

fn run() -> Result<()> {
    // setup logging
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let log = slog::Logger::root(drain, slog_o!());
    let _guard = slog_scope::set_global_logger(log);

    debug!("Logging initialized");

    // initialize program
    let config = init().chain_err(|| "initialization")?;

    trace!("cli configuration - {:?}", config);

    // fetch the metadata from the configured provider
    let metadata =
        fetch_metadata(&config.provider).chain_err(|| "fetching metadata from provider")?;

    // write attributes if configured to do so
    config
        .attributes_file
        .map_or(Ok(()), |x| metadata.write_attributes(x))
        .chain_err(|| "writing metadata attributes")?;

    // write ssh keys if configured to do so
    config
        .ssh_keys_user
        .map_or(Ok(()), |x| metadata.write_ssh_keys(x))
        .chain_err(|| "writing ssh keys")?;

    // write hostname if configured to do so
    config
        .hostname_file
        .map_or(Ok(()), |x| metadata.write_hostname(x))
        .chain_err(|| "writing hostname")?;

    // write network units if configured to do so
    config
        .network_units_dir
        .map_or(Ok(()), |x| metadata.write_network_units(x))
        .chain_err(|| "writing network units")?;

    // perform boot check-in.
    if config.check_in {
        metadata
            .boot_checkin()
            .chain_err(|| "checking-in instance boot to cloud provider")?;
    }

    debug!("Done!");

    Ok(())
}

fn init() -> Result<Config> {
    // do some pre-processing on the command line arguments so that we support
    // golang-style arguments for backwards compatibility. since we have a
    // rather restricted set of flags, all without short options, we can make
    // a lot of assumptions about what we are seeing.
    let args = env::args().map(|arg| {
        if arg.starts_with('-') && !arg.starts_with("--") && arg.len() > 2 {
            format!("-{}", arg)
        } else {
            arg
        }
    });

    // setup cli
    // WARNING: if additional arguments are added, one of two things needs to
    // happen:
    //   1. don't add a shortflag
    //   2. modify the preprocessing logic above to be smarter about where it
    //      prepends the hyphens
    // the preprocessing will probably convert any short flags it finds into
    // long ones
    let matches = App::new("Afterburn")
        .version(crate_version!())
        .arg(
            Arg::with_name("attributes")
                .long("attributes")
                .help("The file into which the metadata attributes are written")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("check-in")
                .long("check-in")
                .help("Check-in this instance boot with the cloud provider"),
        )
        .arg(
            Arg::with_name("cmdline")
                .long("cmdline")
                .help("Read the cloud provider from the kernel cmdline"),
        )
        .arg(
            Arg::with_name("hostname")
                .long("hostname")
                .help("The file into which the hostname should be written")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("network-units")
                .long("network-units")
                .help("The directory into which network units are written")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("provider")
                .long("provider")
                .help("The name of the cloud provider")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("ssh-keys")
                .long("ssh-keys")
                .help("Update SSH keys for the given user")
                .takes_value(true),
        )
        .get_matches_from(args);

    // return configuration
    Ok(Config {
        provider: match matches.value_of("provider") {
            Some(provider) => String::from(provider),
            None => {
                if matches.is_present("cmdline") {
                    util::get_platform(CMDLINE_PATH)?
                } else {
                    return Err("Must set either --provider or --cmdline".into());
                }
            }
        },
        attributes_file: matches.value_of("attributes").map(String::from),
        check_in: matches.is_present("check-in"),
        ssh_keys_user: matches.value_of("ssh-keys").map(String::from),
        hostname_file: matches.value_of("hostname").map(String::from),
        network_units_dir: matches.value_of("network-units").map(String::from),
    })
}
