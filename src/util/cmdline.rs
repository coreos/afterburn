// Copyright 2018 CoreOS, Inc.
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

//! Kernel cmdline parsing - utility functions
//!
//! NOTE(lucab): this is not a complete/correct cmdline parser, as it implements
//!  just enough logic to extract the OEM ID value. In particular, it doesn't
//!  handle separator quoting/escaping, list of values, and merging of repeated
//!  flags.

use errors::*;
use std::io::Read;
use std::{fs, io};

// Get OEM ID value from cmdline file.
pub fn get_oem(fpath: &str, flagname: &str) -> Result<String> {
    // open the cmdline file
    let file =
        fs::File::open(fpath).chain_err(|| format!("Failed to open cmdline file ({})", fpath))?;

    // read the contents
    let mut bufrd = io::BufReader::new(file);
    let mut contents = String::new();
    bufrd
        .read_to_string(&mut contents)
        .chain_err(|| format!("Failed to read cmdline file ({})", fpath))?;

    match find_flag_value(flagname, &contents) {
        Some(s) => Ok(s),
        None => bail!(
            "Couldn't find flag '{}' in cmdline file ({})",
            flagname,
            fpath
        ),
    }
}

// Find OEM ID flag value in cmdline string.
fn find_flag_value(flagname: &str, cmdline: &str) -> Option<String> {
    // split the contents into elements and keep key-value tuples only.
    let params: Vec<(&str, &str)> = cmdline
        .split(' ')
        .filter_map(|s| {
            let kv: Vec<&str> = s.splitn(2, '=').collect();
            match kv.len() {
                2 => Some((kv[0], kv[1])),
                _ => None,
            }
        })
        .collect();

    // find the oem flag
    for (key, val) in params {
        if key != flagname {
            continue;
        }
        let bare_val = val.trim();
        if !bare_val.is_empty() {
            return Some(bare_val.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_find_flag() {
        let flagname = "coreos.oem.id";
        let tests = vec![
            ("", None),
            ("foo=bar", None),
            ("coreos.oem.id", None),
            ("coreos.oem.id=", None),
            ("coreos.oem.id=\t", None),
            ("coreos.oem.id=ec2", Some("ec2".to_string())),
            ("coreos.oem.id=\tec2", Some("ec2".to_string())),
            ("coreos.oem.id=ec2\n", Some("ec2".to_string())),
            ("foo=bar coreos.oem.id=ec2", Some("ec2".to_string())),
            ("coreos.oem.id=ec2 foo=bar", Some("ec2".to_string())),
        ];
        for (tcase, tres) in tests {
            let res = find_flag_value(flagname, tcase);
            assert_eq!(res, tres, "failed testcase: '{}'", tcase);
        }
    }
}
