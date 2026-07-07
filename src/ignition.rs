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

//! Ignition config fragment types.
//!
//! Provider-agnostic representation of a per-feature Ignition config fragment,
//! with helpers to build the hostname/platform-user fragments and write them
//! out as `.ign` files. Ignition merges these natively from
//! `base.platform.d/<platform>/` under a system config directory such as
//! `/etc/ignition`. Sourcing the underlying data from a provider lives in the
//! `render-ignition` CLI sub-command.

use anyhow::{Context, Result};
use serde::Serialize;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

const IGNITION_VERSION: &str = "3.0.0";

#[derive(Debug, Serialize)]
pub(crate) struct IgnitionConfig {
    pub ignition: IgnitionMeta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage: Option<Storage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passwd: Option<Passwd>,
}

impl IgnitionConfig {
    /// Build a fragment that sets the system hostname via an `/etc/hostname`
    /// storage file.
    pub(crate) fn hostname_fragment(hostname: &str) -> Self {
        IgnitionConfig {
            ignition: IgnitionMeta {
                version: IGNITION_VERSION.to_string(),
            },
            storage: Some(Storage {
                files: vec![StorageFile {
                    path: "/etc/hostname".into(),
                    mode: 420,
                    overwrite: true,
                    contents: FileContents {
                        source: hostname_data_uri(hostname),
                    },
                }],
            }),
            passwd: None,
        }
    }

    /// Build a fragment that configures a single platform user with the given
    /// SSH keys and optional password hash.
    pub(crate) fn user_fragment(
        name: String,
        ssh_keys: Vec<String>,
        password_hash: Option<String>,
    ) -> Self {
        IgnitionConfig {
            ignition: IgnitionMeta {
                version: IGNITION_VERSION.to_string(),
            },
            storage: None,
            passwd: Some(Passwd {
                users: vec![PasswdUser {
                    name,
                    ssh_authorized_keys: if ssh_keys.is_empty() {
                        None
                    } else {
                        Some(ssh_keys)
                    },
                    password_hash,
                }],
            }),
        }
    }

    /// Serialize this config and write it as an Ignition fragment file at
    /// `path`, creating parent directories and setting mode 0644.
    pub(crate) fn write_to(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create directory {}", parent.display()))?;
        }
        let json =
            serde_json::to_string_pretty(self).context("failed to serialize ignition config")?;
        fs::write(path, json.as_bytes())
            .with_context(|| format!("failed to write {}", path.display()))?;
        fs::set_permissions(path, fs::Permissions::from_mode(0o644))
            .with_context(|| format!("failed to set permissions on {}", path.display()))?;
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct IgnitionMeta {
    pub version: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct Storage {
    pub files: Vec<StorageFile>,
}

#[derive(Debug, Serialize)]
pub(crate) struct StorageFile {
    pub path: String,
    pub mode: u32,
    pub overwrite: bool,
    pub contents: FileContents,
}

#[derive(Debug, Serialize)]
pub(crate) struct FileContents {
    pub source: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct Passwd {
    pub users: Vec<PasswdUser>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PasswdUser {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_authorized_keys: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_hash: Option<String>,
}

fn hostname_data_uri(hostname: &str) -> String {
    let encoded =
        percent_encoding::utf8_percent_encode(hostname, percent_encoding::NON_ALPHANUMERIC)
            .to_string();
    format!("data:,{encoded}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hostname_fragment_builds_storage_file() {
        let cfg = IgnitionConfig::hostname_fragment("myvm");
        let v: serde_json::Value =
            serde_json::from_str(&serde_json::to_string_pretty(&cfg).unwrap()).unwrap();
        assert_eq!(v["storage"]["files"][0]["path"], "/etc/hostname");
        assert_eq!(v["storage"]["files"][0]["contents"]["source"], "data:,myvm");
        assert!(v.get("passwd").is_none());
    }

    #[test]
    fn test_user_fragment_omits_empty_ssh_keys() {
        let cfg = IgnitionConfig::user_fragment("core".into(), vec![], None);
        let v: serde_json::Value =
            serde_json::from_str(&serde_json::to_string_pretty(&cfg).unwrap()).unwrap();
        assert_eq!(v["passwd"]["users"][0]["name"], "core");
        assert!(v["passwd"]["users"][0].get("sshAuthorizedKeys").is_none());
        assert!(v["passwd"]["users"][0].get("passwordHash").is_none());
    }

    #[test]
    fn test_user_fragment_includes_keys_and_hash() {
        let cfg = IgnitionConfig::user_fragment(
            "core".into(),
            vec!["ssh-ed25519 AAAA... test".into()],
            Some("$6$rounds=10000$salt$hash".into()),
        );
        let v: serde_json::Value =
            serde_json::from_str(&serde_json::to_string_pretty(&cfg).unwrap()).unwrap();
        assert_eq!(
            v["passwd"]["users"][0]["sshAuthorizedKeys"][0],
            "ssh-ed25519 AAAA... test"
        );
        assert_eq!(
            v["passwd"]["users"][0]["passwordHash"],
            "$6$rounds=10000$salt$hash"
        );
    }

    #[test]
    fn test_ignition_json_with_keys() {
        let cfg = IgnitionConfig {
            ignition: IgnitionMeta {
                version: "3.0.0".into(),
            },
            storage: None,
            passwd: Some(Passwd {
                users: vec![PasswdUser {
                    name: "testuser".into(),
                    ssh_authorized_keys: Some(vec!["ssh-ed25519 AAAA...".into()]),
                    password_hash: None,
                }],
            }),
        };
        let v: serde_json::Value =
            serde_json::from_str(&serde_json::to_string_pretty(&cfg).unwrap()).unwrap();
        assert_eq!(v["ignition"]["version"], "3.0.0");
        assert_eq!(v["passwd"]["users"][0]["name"], "testuser");
        assert_eq!(
            v["passwd"]["users"][0]["sshAuthorizedKeys"][0],
            "ssh-ed25519 AAAA..."
        );
        assert!(v["passwd"]["users"][0].get("passwordHash").is_none());
    }

    #[test]
    fn test_ignition_json_with_password_hash() {
        let cfg = IgnitionConfig {
            ignition: IgnitionMeta {
                version: "3.0.0".into(),
            },
            storage: None,
            passwd: Some(Passwd {
                users: vec![PasswdUser {
                    name: "azureuser".into(),
                    ssh_authorized_keys: None,
                    password_hash: Some("$6$rounds=10000$salt$hash".into()),
                }],
            }),
        };
        let v: serde_json::Value =
            serde_json::from_str(&serde_json::to_string_pretty(&cfg).unwrap()).unwrap();
        assert_eq!(v["passwd"]["users"][0]["name"], "azureuser");
        assert!(v["passwd"]["users"][0].get("sshAuthorizedKeys").is_none());
        assert_eq!(
            v["passwd"]["users"][0]["passwordHash"],
            "$6$rounds=10000$salt$hash"
        );
    }

    #[test]
    fn test_write_to_emits_valid_json_and_permissions() {
        let tmp = tempfile::tempdir().unwrap();
        let out_file = tmp
            .path()
            .join("etc/ignition/base.platform.d/azure/extensions.ign");

        let cfg = IgnitionConfig {
            ignition: IgnitionMeta {
                version: "3.0.0".into(),
            },
            storage: None,
            passwd: Some(Passwd {
                users: vec![PasswdUser {
                    name: "core".into(),
                    ssh_authorized_keys: Some(vec![
                        "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAAAgQDYVEprvtYJXVOBN0XNKVVRNCRX6BlnNbI+USLGais1sUWPwtSg7z9K9vhbYAPUZcq8c/s5S9dg5vTHbsiyPCIDOKyeHba4MUJq8Oh5b2i71/3BISpyxTBH/uZDHdslW2a+SrPDCeuMMoss9NFhBdKtDkdG9zyi0ibmCP6yMdEX8Q== Generated by Nova".into(),
                    ]),
                    password_hash: None,
                }],
            }),
        };

        cfg.write_to(&out_file).unwrap();

        assert!(out_file.exists());

        let raw = fs::read_to_string(&out_file).unwrap();
        let json: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(json["ignition"]["version"], "3.0.0");
        assert_eq!(json["passwd"]["users"][0]["name"], "core");

        let mode = fs::metadata(&out_file).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o644);
    }

    #[test]
    fn test_hostname_data_uri() {
        assert_eq!(hostname_data_uri("core1"), "data:,core1");
        assert_eq!(
            hostname_data_uri("my-vm.internal"),
            "data:,my%2Dvm%2Einternal"
        );
    }

    #[test]
    fn test_hostname_storage_fragment_serialization() {
        let cfg = IgnitionConfig {
            ignition: IgnitionMeta {
                version: "3.0.0".into(),
            },
            storage: Some(Storage {
                files: vec![StorageFile {
                    path: "/etc/hostname".into(),
                    mode: 420,
                    overwrite: true,
                    contents: FileContents {
                        source: hostname_data_uri("myvm"),
                    },
                }],
            }),
            passwd: None,
        };

        let v: serde_json::Value =
            serde_json::from_str(&serde_json::to_string_pretty(&cfg).unwrap()).unwrap();
        assert_eq!(v["ignition"]["version"], "3.0.0");
        assert_eq!(v["storage"]["files"][0]["path"], "/etc/hostname");
        assert_eq!(v["storage"]["files"][0]["mode"], 420);
        assert_eq!(v["storage"]["files"][0]["overwrite"], true);
        assert_eq!(v["storage"]["files"][0]["contents"]["source"], "data:,myvm");
        assert!(v.get("passwd").is_none());
    }

    #[test]
    fn test_storage_none_omitted_from_json() {
        let cfg = IgnitionConfig {
            ignition: IgnitionMeta {
                version: "3.0.0".into(),
            },
            storage: None,
            passwd: None,
        };

        let v: serde_json::Value =
            serde_json::from_str(&serde_json::to_string_pretty(&cfg).unwrap()).unwrap();
        assert!(v.get("storage").is_none());
        assert!(v.get("passwd").is_none());
    }
}
