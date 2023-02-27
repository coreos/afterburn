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
//!  just enough logic to extract a few interesting values. In particular, it doesn't
//!  handle separator quoting/escaping, list of values, and merging of repeated
//!  flags.

use anyhow::{bail, Context, Result};
use slog_scope::trace;

/// Platform key.
const CMDLINE_PLATFORM_FLAG: &str = "ignition.platform.id";

/// Get platform value from cmdline file.
pub fn get_platform(fpath: &str) -> Result<String> {
    let content = std::fs::read_to_string(fpath)
        .with_context(|| format!("Failed to read cmdline file ({fpath})"))?;

    match find_flag_value(CMDLINE_PLATFORM_FLAG, &content) {
        Some(platform) => {
            trace!("found '{}' flag: {}", CMDLINE_PLATFORM_FLAG, platform);
            Ok(platform)
        }
        None => bail!(
            "Couldn't find flag '{}' in cmdline file ({})",
            CMDLINE_PLATFORM_FLAG,
            fpath
        ),
    }
}

/// Check whether kernel cmdline file contains flags for network configuration.
#[allow(unused)]
pub fn has_network_kargs(fpath: &str) -> Result<bool> {
    const IP_PREFIX: &str = "ip=";

    let content = std::fs::read_to_string(fpath)
        .with_context(|| format!("Failed to read cmdline file ({fpath})"))?;
    let has_ip = contains_flag_prefix(&content, IP_PREFIX);
    Ok(has_ip)
}

/// Check whether cmdline contains any flag starting with the given prefix.
///
/// This splits `cmdline` content into flag elements and match each with `prefix`,
/// short-circuiting to `true` on the first match.
fn contains_flag_prefix(cmdline: &str, prefix: &str) -> bool {
    cmdline.split(' ').any(|s| s.starts_with(prefix))
}

// Find value of flag in cmdline string.
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
            assert_eq!(res, tres, "failed testcase: '{tcase}'");
        }
    }

    #[test]
    fn test_contains_flag_prefix() {
        let prefix = "ip=";
        let tests = vec![
            ("", false),
            ("ip=foo", true),
            ("ip=\n", true),
            ("coreos.oem.id=", false),
            ("coreos.oem.id=ec2", false),
            ("coreos.oem.id=ip ip=bar\n", true),
        ];
        for (tcase, tres) in tests {
            let res = contains_flag_prefix(tcase, prefix);
            assert_eq!(res, tres, "failed testcase: '{tcase}'");
        }
    }
}
