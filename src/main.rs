/*
 * Copyright 2015 CoreOS, Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#[macro_use]
extern crate clap;
extern crate hyper;

use clap::{App, Arg};
use hyper::Client;
use hyper::client::Response;
use std::error::Error;
use std::{fmt, fs, io};
use std::fs::File;
use std::io::{Read, Write, stderr};
use std::path::PathBuf;
use std::process::exit;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::thread::sleep_ms;

arg_enum!{
    enum Provider {
        EC2
    }
}

struct Metadata {
    public_ipv4: Ipv4Addr,
    local_ipv4: Ipv4Addr,
    hostname: String
}

struct MetadataError {
    description: String,
    cause: Option<Box<Error>>
}

impl From<hyper::Error> for MetadataError {
    fn from(err: hyper::Error) -> MetadataError {
        MetadataError{
            description: "HTTP failure".to_string(),
            cause: Some(Box::new(err))
        }
    }
}

impl From<io::Error> for MetadataError {
    fn from(err: io::Error) -> MetadataError {
        MetadataError{
            description: "IO failure".to_string(),
            cause: Some(Box::new(err))
        }
    }
}

impl fmt::Display for MetadataError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.cause {
            Some(ref cause) => write!(f, "{}: {:?}", self.description, cause),
            None => write!(f, "{}", self.description)
        }
    }
}

fn parse_flags() -> (Provider, PathBuf) {
    let providers = ["ec2"];
    let matches = App::new("")
        .version(&crate_version!()[..])
        .about("A simple agent for fetching and saving cloud-provider metadata")
        .arg(Arg::with_name("PROVIDER")
             .short("p")
             .long("provider")
             .help("The name of the cloud provider")
             .possible_values(&providers)
             .takes_value(true)
             .required(true))
        .arg(Arg::with_name("OUTPUT")
             .short("o")
             .long("output")
             .help("The file into which the metadata is written")
             .takes_value(true)
             .required(true))
        .get_matches();
    (
        value_t_or_exit!(matches.value_of("PROVIDER"), Provider),
        PathBuf::from(matches.value_of("OUTPUT").unwrap()),
    )
}

fn fetch_metadata(provider: Provider) -> Result<Metadata, MetadataError> {
    fn get_with_retry(client: &Client, url: &str) -> Result<Response, MetadataError> {
        for attempt in 0..10 {
            writeln!(stderr(), "GET '{}': Attempt {}", url, attempt).unwrap();
            match client.get(url).send() {
                Ok(response) => return Ok(response),
                Err(e) => writeln!(stderr(), "error: {:?}", e).unwrap()
            };
            let delay = {
                let delay = (2 as u32).pow(attempt) * 100;
                if delay > 1000 { 1000 } else { delay }
            };
            writeln!(stderr(), "sleeping {}ms", delay).unwrap();
            sleep_ms(delay);
        }

        Err(MetadataError{
            description: format!("timed out while fetching '{}'", url),
            cause: None
        })
    }

    fn fetch_string(client: &Client, url: &'static str) -> Result<String, MetadataError> {
        let mut response = try!(get_with_retry(client, url));

        if response.status.is_success() {
            let mut value = String::new();
            try!(response.read_to_string(&mut value));
            Ok(value)
        } else {
            Err(MetadataError{
                description: match response.status.canonical_reason() {
                    Some(reason) => reason.to_string(),
                    None => format!("unknown HTTP failure ({})", response.status)
                },
                cause: None
            })
        }
    }

    fn fetch_ipv4(client: &Client, url: &'static str) -> Result<Ipv4Addr, MetadataError> {
        let response = try!(fetch_string(client, url));
        match Ipv4Addr::from_str(&response[..]) {
            Ok(ip) => Ok(ip),
            Err(_) => Err(MetadataError{
                description: format!("could not parse '{}' as IPv4 address", response),
                cause: None
            })
        }
    }

    match provider {
        Provider::EC2 => {
            let client = Client::new();
            Ok(Metadata {
                public_ipv4: try!(fetch_ipv4(&client, "http://169.254.169.254/2009-04-04/meta-data/public-ipv4")),
                local_ipv4: try!(fetch_ipv4(&client, "http://169.254.169.254/2009-04-04/meta-data/local-ipv4")),
                hostname: try!(fetch_string(&client, "http://169.254.169.254/2009-04-04/meta-data/hostname"))
            })
        }
    }
}

fn write_metadata(output: File, metadata: Metadata) -> Result<(), MetadataError> {
    try!(writeln!(&output, "COREOS_IPV4_PUBLIC={}", metadata.public_ipv4));
    try!(writeln!(&output, "COREOS_IPV4_LOCAL={}", metadata.local_ipv4));
    try!(writeln!(&output, "COREOS_HOSTNAME={}", metadata.hostname));
    Ok(())
}

fn process(provider: Provider, filename: PathBuf) -> Result<(), MetadataError> {
    if let Some(dir) = filename.parent() {
        if let Err(e) = fs::create_dir_all(dir) {
            return Err(MetadataError {
                description: format!("failed to create directory '{}'", dir.display()),
                cause: Some(Box::new(e))
            })
        }
    };
    let file = try!(File::create(&filename));
    let metadata = try!(fetch_metadata(provider));
    write_metadata(file, metadata)
}

fn main() {
    let (provider, filename) = parse_flags();
    if let Err(e) = process(provider, filename) {
        writeln!(&mut stderr(), "failed to process metadata: {}", e).unwrap();
        exit(1);
    }
}
