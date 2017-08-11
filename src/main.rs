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

#[macro_use]
extern crate clap;
#[macro_use]
extern crate slog;
extern crate slog_term;
extern crate slog_async;
#[macro_use]
extern crate slog_scope;
extern crate users;

#[macro_use]
mod macros;
mod metadata;
mod ssh;
mod network;

use std::fs::File;
use std::io::prelude::*;
use clap::{Arg, App};
use slog::Drain;

const CMDLINE_PATH: &'static str = "/proc/cmdline";
const CMDLINE_OEM_FLAG:&'static str = "coreos.oem.id";

#[derive(Debug)]
struct Config {
    provider: String,
    attributes_file: Option<String>,
    ssh_keys_user: Option<String>,
    hostname_file: Option<String>,
    network_units_dir: Option<String>,
}

fn main() {
    // setup logging
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let log = slog::Logger::root(drain, slog_o!());
    let _guard = slog_scope::set_global_logger(log);

    debug!("Logging initialized");

    // initialize program
    let config = init()
        .unwrap_or_else(log_and_die!("initialization"));

    trace!("cli configuration - {:?}", config);

    // fetch the metadata from the configured provider
    let metadata = metadata::fetch_metadata(&config.provider)
        .unwrap_or_else(log_and_die!("fetching metadata from provider"));

    // write attributes if configured to do so
    config.attributes_file
        .map_or(Ok(()), |x| metadata.write_attributes(x))
        .unwrap_or_else(log_and_die!("writing metadata attributes"));

    // write ssh keys if configured to do so
    config.ssh_keys_user
        .map_or(Ok(()), |x| metadata.write_ssh_keys(x))
        .unwrap_or_else(log_and_die!("writing metadata attributes"));

    // write hostname if configured to do so
    config.hostname_file
        .map_or(Ok(()), |x| metadata.write_hostname(x))
        .unwrap_or_else(log_and_die!("writing metadata attributes"));

    // write network units if configured to do so
    config.network_units_dir
        .map_or(Ok(()), |x| metadata.write_network_units(x))
        .unwrap_or_else(log_and_die!("writing metadata attributes"));

    debug!("Done!")
}

fn init() -> Result<Config, String> {
    // setup cli
    let matches = App::new("coreos-metadata")
        .version(crate_version!())
        .arg(Arg::with_name("attributes")
             .long("attributes")
             .help("The file into which the metadata attributes are written")
             .takes_value(true))
        .arg(Arg::with_name("cmdline")
             .long("cmdline")
             .help("Read the cloud provider from the kernel cmdline"))
        .arg(Arg::with_name("hostname")
             .long("hostname")
             .help("The file into which the hostname should be written")
             .takes_value(true))
        .arg(Arg::with_name("network-units")
             .long("network-units")
             .help("The directory into which network units are written")
             .takes_value(true))
        .arg(Arg::with_name("provider")
             .long("provider")
             .help("The name of the cloud provider")
             .takes_value(true))
        .arg(Arg::with_name("ssh-keys")
             .long("ssh-keys")
             .help("Update SSH keys for the given user")
             .takes_value(true))
        .get_matches();

    // return configuration
    Ok(Config {
        provider: match matches.value_of("provider") {
            Some(provider) => String::from(provider),
            None => if matches.is_present("cmdline") {
                get_oem()?
            } else {
                return Err("Must set either --provider or --cmdline".to_string());
            }
        },
        attributes_file: matches.value_of("attributes").map(String::from),
        ssh_keys_user: matches.value_of("ssh-keys").map(String::from),
        hostname_file: matches.value_of("hostname").map(String::from),
        network_units_dir: matches.value_of("network-units").map(String::from),
    })
}

fn get_oem() -> Result<String, String> {
    // open the cmdline file
    let mut file = File::open(CMDLINE_PATH)
        .map_err(wrap_error!("Failed to open cmdline file ({})", CMDLINE_PATH))?;

    // read the contents
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(wrap_error!("Failed to read cmdline file ({})", CMDLINE_PATH))?;

    // split the contents into elements
    let params: Vec<Vec<&str>> = contents.split(' ')
        .map(|s| s.split('=').collect())
        .collect();

    // find the oem flag
    for p in params {
        if p.len() > 1 && p[0] == CMDLINE_OEM_FLAG {
            return Ok(String::from(p[1]));
        }
    }

    Err(format!("Couldn't find '{}' flag in cmdline file ({})", CMDLINE_OEM_FLAG, CMDLINE_PATH))
}
