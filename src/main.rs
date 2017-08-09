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

use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::prelude::*;
use clap::{Arg, App};
use slog::Drain;
use std::ops::Deref;
use std::collections::HashMap;
use users::os::unix::UserExt;

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
    attributes: HashMap<String, String>,
    hostname: Option<String>,
    ssh_keys: Vec<String>,
    network: Vec<NetworkInterface>,
    net_dev: Vec<NetworkDevice>,
}

struct NetworkInterface {
}

struct NetworkDevice{
}

impl NetworkInterface {
    fn unit_name(&self) -> String {
        String::new()
    }
    fn config(&self) -> String {
        String::new()
    }
}

impl NetworkDevice {
    fn unit_name(&self) -> String {
        String::new()
    }
    fn config(&self) -> String {
        String::new()
    }
}

fn create_file(filename: String) -> Result<File, String> {
    let file_path = Path::new(&filename);
    // create the directories if they don't exist
    let folder = file_path.parent()
        .ok_or(format!("could not get parent directory of {:?}", file_path))?;
    fs::create_dir_all(&folder)
        .map_err(|err| format!("failed to create directory {:?}: {:?}", folder, err))?;
    // create (or truncate) the file we want to write to
    File::create(file_path)
        .map_err(|err| format!("failed to create file {:?}: {:?}", file_path, err))
}

// this actually has to be a lot more complicate than this. We need to properly
// interact with the existing go tooling, which uses lock files on disk to
// ensure that only one program is manipulating the authorized keys
// this whole part of the os is really weird.
// for now, with this poc, just leave it like this.
fn create_authorized_keys_dir(user: users::User) -> Result<PathBuf, String> {
    // construct the path to the authorized keys directory
    let ssh_dir = user.home_dir().join(".ssh");
    let authorized_keys_dir = ssh_dir.join("authorized_keys.d");
    // check if the authorized keys directory exists
    if authorized_keys_dir.is_dir() {
        // if it does, just return
        return Ok(authorized_keys_dir);
    }
    // if it doesn't, create it
    fs::create_dir_all(&authorized_keys_dir)
        .map_err(|err| format!("failed to create directory {:?}: {:?}", authorized_keys_dir, err))?;
    // check if there is an authorized keys file
    let authorized_keys_file = ssh_dir.join("authorized_keys");
    if authorized_keys_file.is_file() {
        // if there is, copy it into the authorized keys directory
        let preserved_keys_file = authorized_keys_dir.join("orig_authorzied_keys");
        fs::copy(&authorized_keys_file, preserved_keys_file)
            .map_err(|err| format!("failed to copy old authorzied keys file: {:?}", err))?;
    }
    // then we are done
    Ok(authorized_keys_dir)
}

fn sync_authorized_keys(authorized_keys_dir: PathBuf) -> Result<(), String> {
    let ssh_dir = authorized_keys_dir.parent()
        .ok_or(format!("could not get parent directory of {:?}", authorized_keys_dir))?;
    let authorized_keys_file = File::create(ssh_dir.join("authorized_keys"))
        .map_err(|err| format!("failed to create file {:?}: {:?}", ssh_dir.join("authorized_keys"), err));
    let dir = fs::read_dir(authorized_keys_dir)
        .map_err(|err| format!("failed to read from directory {:?}: {:?}", authorized_keys_dir, err))?;
    for entry in dir {

    }
}

impl Metadata {
    fn write_attributes(&self, attributes_file_path: String) -> Result<(), String> {
        let mut attributes_file = create_file(attributes_file_path)?;
        for (k,v) in &self.attributes {
            write!(&mut attributes_file, "COREOS_{}={}\n", k, v)
                .map_err(|err| format!("failed to write attributes to file {:?}: {:?}", attributes_file, err))?;
        }
        Ok(())
    }
    fn write_ssh_keys(&self, ssh_keys_user: String) -> Result<(), String> {
        let user = users::get_user_by_name(ssh_keys_user.as_str())
            .ok_or(format!("could not find user with username {:?}", ssh_keys_user))?;
        let authorized_keys_dir = create_authorized_keys_dir(user)?;
        let mut authorized_keys_file = File::create(authorized_keys_dir.join("coreos-metadata"))
            .map_err(|err| format!("failed to create the file {:?} in the ssh authorized users directory: {:?}", "coreos-metadata", err))?;
        for ssh_key in &self.ssh_keys {
            write!(&mut authorized_keys_file, "{}\n", ssh_key)
                .map_err(|err| format!("failed to write ssh key to file {:?}: {:?}", authorized_keys_file, err))?;
        }
        sync_authorized_keys(authorized_keys_dir)
    }
    fn write_hostname(&self, hostname_file_path: String) -> Result<(), String> {
        match self.hostname {
            Some(ref hostname) => {
                let mut hostname_file = create_file(hostname_file_path)?;
                write!(&mut hostname_file, "{}\n", hostname)
                    .map_err(|err| format!("failed to write hostname {:?} to file {:?}: {:?}", self.hostname, hostname_file, err))
            }
            None => Ok(())
        }
    }
    fn write_network_units(&self, network_units_dir: String) -> Result<(), String> {
        let dir_path = Path::new(&network_units_dir);
        fs::create_dir_all(&dir_path)
            .map_err(|err| format!("failed to create directory {:?}: {:?}", dir_path, err))?;
        for interface in &self.network {
            let file_path = dir_path.join(interface.unit_name());
            let mut unit_file = File::create(&file_path)
                .map_err(|err| format!("failed to create file {:?}: {:?}", file_path, err))?;
            write!(&mut unit_file, "{}", interface.config())
                .map_err(|err| format!("failed to write network interface unit file {:?}: {:?}", unit_file, err))?;
        }
        for device in &self.net_dev {
            let file_path = dir_path.join(device.unit_name());
            let mut unit_file = File::create(&file_path)
                .map_err(|err| format!("failed to create file {:?}: {:?}", file_path, err))?;
            write!(&mut unit_file, "{}", device.config())
                .map_err(|err| format!("failed to write network device unit file {:?}: {:?}", unit_file, err))?;
        }
        Ok(())
    }
}

macro_rules! log_and_die {
    ($x:expr) => {
        |err| {
            error!($x; "error" => err);
            panic!()
        }
    };
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

    // get the concrete provider from the configured value
    let fetch = get_metadata_fetch(config.provider)
        .unwrap_or_else(log_and_die!("getting provider"));

    // fetch the metadata from that provider
    let metadata = fetch()
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
