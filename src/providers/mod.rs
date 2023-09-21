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

//! Providers
//!
//! These are the providers which Afterburn knows how to retrieve metadata
//! from. Internally, they handle the ins and outs of each providers metadata
//! services, and externally, they provide a function to fetch that metadata in
//! a regular format.
//!
//! To add a provider, put a `pub mod provider;` line in this file, export a
//! function to fetch the metadata, and then add a match line in the top-level
//! `fetch_metadata()` function in metadata.rs.

pub mod aliyun;
pub mod aws;
pub mod cloudstack;
pub mod digitalocean;
pub mod exoscale;
pub mod gcp;
pub mod hetzner;
pub mod ibmcloud;
pub mod ibmcloud_classic;
pub mod kubevirt;
pub mod microsoft;
pub mod openstack;
pub mod packet;
pub mod powervs;
pub mod vmware;
pub mod vultr;

use crate::network;
use anyhow::{anyhow, Context, Result};
use libsystemd::logging;
use nix::unistd;
use openssh_keys::PublicKey;
use slog_scope::warn;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::Path;
use uzers::{self, User};

/// Message ID markers for authorized-keys entries in journal.
const AFTERBURN_SSH_AUTHORIZED_KEYS_ADDED_MESSAGEID: &str = "0f7d7a502f2d433caa1323440a6b4190";
const AFTERBURN_SSH_AUTHORIZED_KEYS_REMOVED_MESSAGEID: &str = "f8b91c53f5544868a3a10d0dcf68e9ea";

fn create_file(filename: &str) -> Result<File> {
    let file_path = Path::new(&filename);
    // create the directories if they don't exist
    let folder = file_path
        .parent()
        .ok_or_else(|| anyhow!("could not get parent directory of {:?}", file_path))?;
    fs::create_dir_all(folder).with_context(|| format!("failed to create directory {folder:?}"))?;
    // create (or truncate) the file we want to write to
    File::create(file_path).with_context(|| format!("failed to create file {file_path:?}"))
}

/// Add a message to the journal logging SSH key additions; this
/// will be used by at least Fedora CoreOS to display in the console
/// if no ssh keys are present.
fn write_ssh_key_journal_entry(log: logging::Priority, name: &str, path: &str, added: bool) {
    let message = format!(
        "{} ssh authorized keys file for user: {}",
        if added { "wrote" } else { "removed" },
        name
    );
    let map = maplit::hashmap! {
        "AFTERBURN_USER_NAME" => name.as_ref(),
        "AFTERBURN_PATH" => path.as_ref(),
        "MESSAGE_ID" => match added {
            true => AFTERBURN_SSH_AUTHORIZED_KEYS_ADDED_MESSAGEID,
            false => AFTERBURN_SSH_AUTHORIZED_KEYS_REMOVED_MESSAGEID,
        },
    };
    if let Err(e) = logging::journal_send(log, &message, map.iter()) {
        warn!("failed to send information to journald: {}", e);
    }
}

fn write_ssh_keys(user: User, ssh_keys: Vec<PublicKey>) -> Result<()> {
    use std::io::ErrorKind::NotFound;
    use uzers::os::unix::UserExt;

    // switch users
    let _guard = uzers::switch::switch_user_group(user.uid(), user.primary_group_id())
        .context("failed to switch user/group")?;

    // get paths
    let dir_path = user.home_dir().join(".ssh").join("authorized_keys.d");
    let file_name = "afterburn";
    let file_path = &dir_path.join(file_name);

    // stringify for logging
    let username = user.name().to_string_lossy();
    let file_path_str = file_path.to_string_lossy();

    if !ssh_keys.is_empty() {
        // ensure directory exists
        fs::create_dir_all(&dir_path)
            .with_context(|| format!("failed to create directory {:?}", &dir_path))?;

        // create temporary file
        let mut temp_file = tempfile::Builder::new()
            .prefix(&format!(".{file_name}-"))
            .tempfile_in(&dir_path)
            .context("failed to create temporary file")?;

        // write out keys
        for key in ssh_keys {
            writeln!(temp_file, "{key}").with_context(|| {
                format!("failed to write to file {:?}", temp_file.path().display())
            })?;
        }

        // sync to disk
        temp_file
            .as_file()
            .sync_all()
            .with_context(|| format!("failed to sync file {:?}", temp_file.path().display()))?;

        // atomically rename to destination
        // don't leak temporary file on error
        temp_file
            .persist(file_path)
            .map_err(|e| {
                e.file.close().ok();
                e.error
            })
            .with_context(|| format!("failed to persist file {:?}", file_path.display()))?;

        // emit journal entry
        write_ssh_key_journal_entry(logging::Priority::Info, &username, &file_path_str, true);
    } else {
        // delete the file
        let deleted = match fs::remove_file(file_path) {
            Err(ref e) if e.kind() == NotFound => Ok(false),
            Err(e) => Err(e),
            Ok(()) => Ok(true),
        }
        .with_context(|| format!("failed to remove file {:?}", file_path.display()))?;

        // emit journal entry
        if deleted {
            write_ssh_key_journal_entry(logging::Priority::Info, &username, &file_path_str, false);
        }
    }

    // sync parent dir to persist updates
    match File::open(&dir_path) {
        Ok(dir_file) => dir_file.sync_all(),
        Err(ref e) if e.kind() == NotFound => Ok(()),
        Err(e) => Err(e),
    }
    .with_context(|| format!("failed to sync '{}'", dir_path.display()))?;

    // make clippy happy while fulfilling our interface
    drop(user);

    Ok(())
}

fn max_hostname_len() -> Result<Option<usize>> {
    unistd::sysconf(unistd::SysconfVar::HOST_NAME_MAX)
        .context("querying maximum hostname length")?
        .map(|l| {
            l.try_into()
                .context("overflow querying maximum hostname length")
        })
        .transpose()
}

pub trait MetadataProvider {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        Ok(HashMap::new())
    }

    fn hostname(&self) -> Result<Option<String>> {
        Ok(None)
    }

    fn ssh_keys(&self) -> Result<Vec<PublicKey>> {
        warn!("ssh-keys requested, but not supported on this platform");
        Ok(vec![])
    }

    fn networks(&self) -> Result<Vec<network::Interface>> {
        Ok(vec![])
    }

    fn boot_checkin(&self) -> Result<()> {
        warn!("boot check-in requested, but not supported on this platform");
        Ok(())
    }

    /// Return a list of virtual network devices for this machine.
    ///
    /// This is used to setup virtual interfaces, e.g. via [systemd.netdev][netdev]
    /// configuration fragments.
    ///
    /// netdev: https://www.freedesktop.org/software/systemd/man/systemd.netdev.html
    fn virtual_network_devices(&self) -> Result<Vec<network::VirtualNetDev>> {
        Ok(vec![])
    }

    /// Return custom initrd network kernel arguments, if any.
    fn rd_network_kargs(&self) -> Result<Option<String>> {
        Ok(None)
    }

    fn write_attributes(&self, attributes_file_path: String) -> Result<()> {
        let mut attributes_file = create_file(&attributes_file_path)?;
        for (k, v) in self.attributes()? {
            writeln!(&mut attributes_file, "AFTERBURN_{k}={v}").with_context(|| {
                format!("failed to write attributes to file {attributes_file:?}")
            })?;
        }
        Ok(())
    }

    fn write_ssh_keys(&self, ssh_keys_user: String) -> Result<()> {
        let ssh_keys = self.ssh_keys()?;
        let user = uzers::get_user_by_name(&ssh_keys_user)
            .ok_or_else(|| anyhow!("could not find user with username {:?}", ssh_keys_user))?;

        write_ssh_keys(user, ssh_keys)?;

        Ok(())
    }

    fn write_hostname(&self, hostname_file_path: String) -> Result<()> {
        if let Some(mut hostname) = self.hostname()? {
            if let Some(maxlen) = max_hostname_len()? {
                if hostname.len() > maxlen {
                    // Value exceeds the system's maximum hostname length.
                    // Truncate hostname to the first dot, or to the maximum
                    // length if necessary.
                    // https://github.com/coreos/afterburn/issues/509
                    slog_scope::info!(
                        "received hostname {:?} longer than {} characters; truncating",
                        hostname,
                        maxlen
                    );
                    hostname.truncate(maxlen);
                    if let Some(idx) = hostname.find('.') {
                        hostname.truncate(idx);
                    }
                }
            }

            let mut hostname_file = create_file(&hostname_file_path)?;
            writeln!(&mut hostname_file, "{hostname}").with_context(|| {
                format!("failed to write hostname {hostname:?} to file {hostname_file:?}")
            })?;
            slog_scope::info!("wrote hostname {} to {}", hostname, hostname_file_path);
        }
        Ok(())
    }

    fn write_network_units(&self, network_units_dir: String) -> Result<()> {
        let dir_path = Path::new(&network_units_dir);
        fs::create_dir_all(dir_path)
            .with_context(|| format!("failed to create directory {dir_path:?}"))?;

        // Write `.network` fragments for network interfaces/links.
        for interface in &self.networks()? {
            let unit_name = interface.sd_network_unit_name()?;
            let file_path = dir_path.join(unit_name);
            let mut unit_file = File::create(&file_path)
                .with_context(|| format!("failed to create file {file_path:?}"))?;
            write!(&mut unit_file, "{}", interface.config()).with_context(|| {
                format!("failed to write network interface unit file {unit_file:?}")
            })?;
        }

        // Write `.netdev` fragments for virtual network devices.
        for device in &self.virtual_network_devices()? {
            let file_path = dir_path.join(device.netdev_unit_name());
            let mut unit_file = File::create(&file_path)
                .with_context(|| format!("failed to create netdev unit file {file_path:?}"))?;
            write!(&mut unit_file, "{}", device.sd_netdev_config())
                .with_context(|| format!("failed to write netdev unit file {unit_file:?}"))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    struct HostnameMock(String);

    impl MetadataProvider for HostnameMock {
        fn hostname(&self) -> Result<Option<String>> {
            Ok(Some(self.0.clone()))
        }
    }

    // write specified hostname to a file, then read it back
    fn try_write_hostname(hostname: &str) -> String {
        let mut temp = NamedTempFile::new().unwrap();
        let provider = HostnameMock(hostname.into());
        provider
            .write_hostname(temp.path().to_str().unwrap().into())
            .unwrap();
        let mut ret = String::new();
        temp.read_to_string(&mut ret).unwrap();
        ret.trim_end().into()
    }

    #[test]
    fn test_hostname_truncation() {
        // assume some maximum exists
        let maxlen = max_hostname_len().unwrap().unwrap();
        let long_string = "helloworld"
            .chars()
            .cycle()
            .take(maxlen * 2)
            .collect::<String>();
        // simple hostname
        assert_eq!(try_write_hostname("hostname7"), "hostname7");
        // simple FQDN
        assert_eq!(
            try_write_hostname("hostname7.example.com"),
            "hostname7.example.com"
        );
        // truncated simple hostname
        assert_eq!(
            try_write_hostname(&long_string[0..maxlen + 10]),
            long_string[0..maxlen]
        );
        // truncated FQDN
        assert_eq!(
            try_write_hostname(&format!("{}.example.com", &long_string[0..maxlen + 5])),
            long_string[0..maxlen]
        );
        // truncate to first dot
        assert_eq!(
            try_write_hostname(&format!("{}.example.com", &long_string[0..maxlen - 5])),
            long_string[0..maxlen - 5]
        );
        // truncate to first dot even if we could truncate to second dot
        assert_eq!(
            try_write_hostname(&format!("{}.example.com", &long_string[0..maxlen - 10])),
            long_string[0..maxlen - 10]
        );
    }
}
