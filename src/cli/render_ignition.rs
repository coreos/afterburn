//! `render-ignition` CLI sub-command.
//!
//! Generates per-feature Ignition config fragment files in an output directory.
//! Each enabled feature writes its own `.ign` file; Ignition merges them
//! natively from `base.platform.d/<platform>/` under a system config directory
//! such as `/etc/ignition`.
//!
//! The Ignition config fragment types and `write_fragment`/`hostname_data_uri`
//! helpers are provider-agnostic and live here so other platforms can reuse
//! them; any provider-specific data (e.g. the admin password) is sourced
//! through the `MetadataProvider` trait rather than referenced directly.

use anyhow::{bail, Context, Result};
use clap::{ArgGroup, Parser};
use serde::Serialize;
use slog_scope::{info, warn};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

const IGNITION_VERSION: &str = "3.0.0";

/// Render Ignition config fragments from cloud provider metadata
#[derive(Debug, Parser)]
#[command(group(ArgGroup::new("provider-group").args(["cmdline", "provider"]).required(true)))]
pub struct CliRenderIgnition {
    /// The name of the cloud provider
    #[arg(long, value_name = "name")]
    provider: Option<String>,
    /// Read the cloud provider from the kernel cmdline
    #[arg(long)]
    cmdline: bool,
    /// Directory to write Ignition config fragment files into
    #[arg(long = "render-ignition-dir", value_name = "path")]
    render_ignition_dir: String,
    /// Do not write the hostname fragment file
    #[arg(long)]
    disable_hostname_fragment: bool,
    /// Do not write the platform user fragment file
    #[arg(long)]
    disable_user_fragment: bool,
}

impl CliRenderIgnition {
    const SUPPORTED_PROVIDERS: &[&str] = &["azure"];

    pub(crate) fn run(self) -> Result<()> {
        let provider_id = super::get_provider(self.provider.as_deref())?;

        if !Self::SUPPORTED_PROVIDERS.contains(&provider_id.as_str()) {
            bail!(
                "render-ignition is only supported for providers {:?}, got '{}'",
                Self::SUPPORTED_PROVIDERS,
                provider_id,
            );
        }

        let hostname = !self.disable_hostname_fragment;
        let platform_user = !self.disable_user_fragment;

        if !hostname && !platform_user {
            slog_scope::warn!("render-ignition: all fragments disabled, nothing to do");
            return Ok(());
        }

        let metadata = crate::metadata::fetch_metadata(&provider_id)
            .context("fetching metadata from provider")?;

        if hostname {
            generate_hostname_fragment(metadata.as_ref(), &self.render_ignition_dir)
                .context("generating hostname ignition fragment")?;
        }

        if platform_user {
            generate_user_fragment(metadata.as_ref(), &self.render_ignition_dir)
                .context("generating platform-user ignition fragment")?;
        }

        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct IgnitionConfig {
    pub ignition: IgnitionMeta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage: Option<Storage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passwd: Option<Passwd>,
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

pub(crate) fn hostname_data_uri(hostname: &str) -> String {
    let encoded =
        percent_encoding::utf8_percent_encode(hostname, percent_encoding::NON_ALPHANUMERIC)
            .to_string();
    format!("data:,{encoded}")
}

pub(crate) fn write_fragment(config: &IgnitionConfig, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create directory {}", parent.display()))?;
    }
    let json =
        serde_json::to_string_pretty(config).context("failed to serialize ignition config")?;
    fs::write(path, json.as_bytes())
        .with_context(|| format!("failed to write {}", path.display()))?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o644))
        .with_context(|| format!("failed to set permissions on {}", path.display()))?;
    Ok(())
}

fn generate_hostname_fragment(
    provider: &dyn crate::providers::MetadataProvider,
    output_dir: &str,
) -> Result<()> {
    let hostname = match provider.hostname()? {
        Some(h) => h,
        None => {
            warn!("hostname requested, but not available from this provider");
            return Ok(());
        }
    };

    let config = IgnitionConfig {
        ignition: IgnitionMeta {
            version: IGNITION_VERSION.to_string(),
        },
        storage: Some(Storage {
            files: vec![StorageFile {
                path: "/etc/hostname".into(),
                mode: 420,
                overwrite: true,
                contents: FileContents {
                    source: hostname_data_uri(&hostname),
                },
            }],
        }),
        passwd: None,
    };

    let path = Path::new(output_dir).join("hostname.ign");
    write_fragment(&config, &path)?;
    info!("wrote hostname ignition fragment"; "path" => path.display().to_string());
    Ok(())
}

fn generate_user_fragment(
    provider: &dyn crate::providers::MetadataProvider,
    output_dir: &str,
) -> Result<()> {
    let username = provider
        .admin_username()
        .context("failed to query admin username from provider")?;
    let username = match username {
        Some(u) => u,
        None => {
            warn!("platform-user requested, but admin username not available from this provider");
            return Ok(());
        }
    };

    let ssh_keys: Vec<String> = provider
        .ssh_keys()
        .context("failed to query SSH keys from provider")?
        .into_iter()
        .map(|k| k.to_key_format())
        .collect();

    let password_hash = provider
        .admin_password_hash()
        .context("failed to query admin password hash from provider")?;

    let config = IgnitionConfig {
        ignition: IgnitionMeta {
            version: IGNITION_VERSION.to_string(),
        },
        storage: None,
        passwd: Some(Passwd {
            users: vec![PasswdUser {
                name: username,
                ssh_authorized_keys: if ssh_keys.is_empty() {
                    None
                } else {
                    Some(ssh_keys)
                },
                password_hash,
            }],
        }),
    };

    let path = Path::new(output_dir).join("user.ign");
    write_fragment(&config, &path)?;
    info!("wrote platform-user ignition fragment"; "path" => path.display().to_string());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use openssh_keys::PublicKey;

    /// Minimal generic provider used to exercise the fragment generators
    /// without any platform-specific plumbing.
    struct FakeProvider {
        hostname: Option<String>,
        admin_username: Option<String>,
        ssh_keys: Vec<&'static str>,
        admin_password_hash: Option<String>,
    }

    impl crate::providers::MetadataProvider for FakeProvider {
        fn hostname(&self) -> Result<Option<String>> {
            Ok(self.hostname.clone())
        }
        fn admin_username(&self) -> Result<Option<String>> {
            Ok(self.admin_username.clone())
        }
        fn ssh_keys(&self) -> Result<Vec<PublicKey>> {
            self.ssh_keys
                .iter()
                .map(|s| {
                    s.parse::<PublicKey>()
                        .map_err(|e| anyhow::anyhow!("failed to parse test ssh key: {e}"))
                })
                .collect()
        }
        fn admin_password_hash(&self) -> Result<Option<String>> {
            Ok(self.admin_password_hash.clone())
        }
    }

    #[test]
    fn test_generate_hostname_fragment_writes_file() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_str().unwrap();

        let provider = FakeProvider {
            hostname: Some("myhost".into()),
            admin_username: None,
            ssh_keys: vec![],
            admin_password_hash: None,
        };

        generate_hostname_fragment(&provider, dir).unwrap();

        let raw = fs::read_to_string(tmp.path().join("hostname.ign")).unwrap();
        let json: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(json["storage"]["files"][0]["path"], "/etc/hostname");
        assert_eq!(
            json["storage"]["files"][0]["contents"]["source"],
            "data:,myhost"
        );
    }

    #[test]
    fn test_generate_hostname_fragment_skipped_when_absent() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_str().unwrap();

        let provider = FakeProvider {
            hostname: None,
            admin_username: None,
            ssh_keys: vec![],
            admin_password_hash: None,
        };

        generate_hostname_fragment(&provider, dir).unwrap();
        assert!(!tmp.path().join("hostname.ign").exists());
    }

    #[test]
    fn test_generate_user_fragment_includes_password_hash() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_str().unwrap();

        let provider = FakeProvider {
            hostname: None,
            admin_username: Some("core".into()),
            ssh_keys: vec![
                "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAAAgQDYVEprvtYJXVOBN0XNKVVRNCRX6BlnNbI+USLGais1sUWPwtSg7z9K9vhbYAPUZcq8c/s5S9dg5vTHbsiyPCIDOKyeHba4MUJq8Oh5b2i71/3BISpyxTBH/uZDHdslW2a+SrPDCeuMMoss9NFhBdKtDkdG9zyi0ibmCP6yMdEX8Q== test",
            ],
            admin_password_hash: Some("$6$rounds=10000$salt$hash".into()),
        };

        generate_user_fragment(&provider, dir).unwrap();

        let raw = fs::read_to_string(tmp.path().join("user.ign")).unwrap();
        let json: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(json["passwd"]["users"][0]["name"], "core");
        assert_eq!(
            json["passwd"]["users"][0]["passwordHash"],
            "$6$rounds=10000$salt$hash"
        );
        assert!(json["passwd"]["users"][0]["sshAuthorizedKeys"][0]
            .as_str()
            .unwrap()
            .starts_with("ssh-rsa "));
    }

    #[test]
    fn test_generate_user_fragment_skipped_without_username() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_str().unwrap();

        let provider = FakeProvider {
            hostname: None,
            admin_username: None,
            ssh_keys: vec![],
            admin_password_hash: Some("$6$rounds=10000$salt$hash".into()),
        };

        generate_user_fragment(&provider, dir).unwrap();
        assert!(!tmp.path().join("user.ign").exists());
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
    fn test_write_fragment_emits_valid_json_and_permissions() {
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

        write_fragment(&cfg, &out_file).unwrap();

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
