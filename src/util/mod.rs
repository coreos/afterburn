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

use anyhow::{anyhow, Context, Result};
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

mod cmdline;
pub use self::cmdline::{get_platform, has_network_kargs};

mod dhcp;
pub use self::dhcp::DhcpOption;

mod mount;
pub(crate) use mount::{mount_ro, unmount};

/// Atomically write `contents` to `path` with the given Unix mode.
///
/// The data is written to a temporary file in the same directory, its
/// permissions are set, and it is then renamed into place. This ensures readers
/// never observe a partially written file, and no file with the temporary's
/// default permissions is ever visible at `path`. Parent directories are
/// created as needed.
pub(crate) fn write_file_atomic(path: &Path, contents: &[u8], mode: u32) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("path {} has no parent directory", path.display()))?;
    fs::create_dir_all(parent)
        .with_context(|| format!("failed to create directory {}", parent.display()))?;

    let mut tmp = tempfile::NamedTempFile::new_in(parent)
        .with_context(|| format!("failed to create temporary file in {}", parent.display()))?;
    tmp.write_all(contents)
        .with_context(|| format!("failed to write temporary file for {}", path.display()))?;
    tmp.as_file()
        .set_permissions(fs::Permissions::from_mode(mode))
        .with_context(|| {
            format!(
                "failed to set permissions on temporary file for {}",
                path.display()
            )
        })?;
    tmp.persist(path)
        .with_context(|| format!("failed to persist {}", path.display()))?;
    Ok(())
}

fn key_lookup_line(delim: char, key: &str, line: &str) -> Option<String> {
    match line.find(delim) {
        Some(index) => {
            let (k, val) = line.split_at(index + 1);
            if k != format!("{key}{delim}") {
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
