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

use std::fs::File;
use std::io::prelude::*;
use clap::{Arg, App};
use slog::Drain;
use std::ops::Deref;

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

type Provider = fn() -> Result<Metadata, String>;

struct Metadata {
}

fn main() {
    // setup logging
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let log = slog::Logger::root(drain, slog_o!());
    let _guard = slog_scope::set_global_logger(log);

    debug!("Logging initialized");

    let config = match init() {
        Ok(config) => config,
        Err(err) => {
            error!("initialization"; "error" => err);
            panic!()
        }
    };

    trace!("cli configuration - {:?}", config);

    // get the concrete provider from the configured value
    let fetch = match get_metadata_fetch(config.provider) {
        Ok(provider) => provider,
        Err(err) => {
            error!("getting provider"; "error" => err);
            panic!()
        }
    };

    // fetch the metadata from that provider
    let _metadata = match fetch() {
        Ok(metadata) => metadata,
        Err(err) => {
            error!("fetching metadata from provider"; "error" => err);
            panic!()
        }
    };
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
        .map_err(|err| format!("Failed to open cmdline file ({}) - {}", CMDLINE_PATH, err))?;

    // read the contents
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|err| format!("Failed to read cmdline file ({}) - {}", CMDLINE_PATH, err))?;

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

fn get_metadata_fetch(provider: String) -> Result<Provider, String> {
    match provider.deref() {
        _ => Err(format!("unknown provider '{}'", provider)),
    }
}
