//! `render-ignition` CLI sub-command.
//!
//! Fetches metadata from a cloud provider and writes Ignition config fragment
//! files for the enabled features. The fragment types and their serialization
//! live in [`crate::ignition`]; this module turns provider metadata into those
//! fragments and handles argument parsing and dispatch.

use anyhow::{bail, Context, Result};
use clap::{ArgGroup, Parser};
use slog_scope::{info, warn};
use std::path::Path;

use crate::ignition::IgnitionConfig;
use crate::providers::MetadataProvider;

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

fn generate_hostname_fragment(provider: &dyn MetadataProvider, output_dir: &str) -> Result<()> {
    let hostname = match provider.hostname()? {
        Some(h) => h,
        None => {
            warn!("hostname requested, but not available from this provider");
            return Ok(());
        }
    };

    let path = Path::new(output_dir).join("hostname.ign");
    IgnitionConfig::hostname_fragment(&hostname).write_to(&path)?;
    info!("wrote hostname ignition fragment"; "path" => path.display().to_string());
    Ok(())
}

fn generate_user_fragment(provider: &dyn MetadataProvider, output_dir: &str) -> Result<()> {
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

    let path = Path::new(output_dir).join("user.ign");
    IgnitionConfig::user_fragment(username, ssh_keys, password_hash).write_to(&path)?;
    info!("wrote platform-user ignition fragment"; "path" => path.display().to_string());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use openssh_keys::PublicKey;
    use std::fs;

    /// Minimal generic provider used to exercise the fragment generators
    /// without any platform-specific plumbing.
    struct FakeProvider {
        hostname: Option<String>,
        admin_username: Option<String>,
        ssh_keys: Vec<&'static str>,
        admin_password_hash: Option<String>,
    }

    impl MetadataProvider for FakeProvider {
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
}
