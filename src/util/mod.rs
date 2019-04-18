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

//! utility functions

use crate::errors::*;
use crate::retry;
use pnet;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::time::Duration;

mod cmdline;
pub use self::cmdline::get_platform;

fn key_lookup_line(delim: char, key: &str, line: &str) -> Option<String> {
    match line.find(delim) {
        Some(index) => {
            let (k, val) = line.split_at(index + 1);
            if k != format!("{}{}", key, delim) {
                None
            } else {
                Some(val.to_owned())
            }
        }
        None => None,
    }
}

pub fn key_lookup<R: Read>(delim: char, key: &str, reader: R) -> Result<Option<String>> {
    let contents = BufReader::new(reader);

    for l in contents.lines() {
        let l = l?;
        if let Some(v) = key_lookup_line(delim, key, &l) {
            return Ok(Some(v));
        }
    }
    Ok(None)
}

pub fn dns_lease_key_lookup(key: &str) -> Result<String> {
    let interfaces = pnet::datalink::interfaces();
    trace!("interfaces - {:?}", interfaces);

    retry::Retry::new()
        .initial_backoff(Duration::from_millis(50))
        .max_backoff(Duration::from_millis(500))
        .max_attempts(60)
        .retry(|_| {
            for interface in interfaces.clone() {
                trace!("looking at interface {:?}", interface);
                let lease_path = format!("/run/systemd/netif/leases/{}", interface.index);
                let lease_path = Path::new(&lease_path);
                if lease_path.exists() {
                    debug!("found lease file - {:?}", lease_path);
                    let lease = File::open(&lease_path)
                        .chain_err(|| format!("failed to open lease file ({:?})", lease_path))?;

                    if let Some(v) = key_lookup('=', key, lease)? {
                        return Ok(v);
                    }

                    debug!(
                        "failed to get value from existing lease file '{:?}'",
                        lease_path
                    );
                }
            }
            Err("failed to retrieve fabric address".into())
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    #[test]
    fn key_lookup_test() {
        let tests = vec![
            (
                '=',
                "DNS",
                "foo=bar\nbaz=bax\nDNS=8.8.8.8\n",
                Some("8.8.8.8".to_owned()),
            ),
            (':', "foo", "foo:bar", Some("bar".to_owned())),
            (' ', "foo", "", None),
            (':', "bar", "foo:bar\nbaz:bar", None),
            (' ', "baz", "foo foo\nbaz bar", Some("bar".to_owned())),
            (' ', "foo", "\n\n\n\n\n\n\n \n", None),
        ];
        for (delim, key, contents, expected_val) in tests {
            let val = key_lookup(delim, key, Cursor::new(contents));
            assert_eq!(val.unwrap(), expected_val);
        }
    }
}
