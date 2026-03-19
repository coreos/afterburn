// Copyright 2026 CoreOS, Inc.
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

//! Ignition config fragment generation for Azure.
//!
//! Reads username and SSH keys from IMDS and writes a JSON Ignition fragment to
//! `/usr/lib/ignition/base.platform.d/azure/extensions.ign`.
//! OVF data is only consulted for `adminPassword` policy checks.

use anyhow::{anyhow, Context, Result};
use reqwest::header::{HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use slog_scope::{info, warn};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::retry;
use crate::util;

const OUTPUT_DIR: &str = "/usr/lib/ignition/base.platform.d/azure";
const OUTPUT_FILE: &str = "/usr/lib/ignition/base.platform.d/azure/extensions.ign";
const IGNITION_VERSION: &str = "3.4.0";
const IMDS_ENDPOINT: &str = "http://169.254.169.254";

const MOUNT_DEVICE: &str = "/dev/sr0";
const MOUNT_POINT: &str = "/run/afterburn/media/";
const CDROM_FS_TYPES: &[&str] = &["udf", "iso9660"];
const MOUNT_RETRIES: u8 = 3;

#[derive(Debug, Serialize)]
struct IgnitionConfig {
    ignition: IgnitionMeta,
    #[serde(skip_serializing_if = "Option::is_none")]
    passwd: Option<Passwd>,
}

#[derive(Debug, Serialize)]
struct IgnitionMeta {
    version: String,
}

#[derive(Debug, Serialize)]
struct Passwd {
    users: Vec<PasswdUser>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PasswdUser {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    ssh_authorized_keys: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "Environment")]
struct OvfEnvironment {
    #[serde(rename = "ProvisioningSection")]
    provisioning_section: ProvisioningSection,
}

#[derive(Debug, Deserialize)]
struct ProvisioningSection {
    #[serde(rename = "LinuxProvisioningConfigurationSet")]
    linux_prov_conf_set: LinuxProvisioningConfigurationSet,
}

#[derive(Debug, Deserialize)]
struct LinuxProvisioningConfigurationSet {
    #[serde(rename = "AdminPassword", alias = "adminPassword", default)]
    admin_password: String,
}

fn imds_client() -> Result<retry::Client> {
    retry::Client::try_new().map(|client| {
        client.header(
            HeaderName::from_static("metadata"),
            HeaderValue::from_static("true"),
        )
    })
}

fn fetch_os_profile_username(client: &retry::Client) -> Result<String> {
    const URL: &str =
        "metadata/instance/compute/osProfile/adminUsername?api-version=2021-02-01&format=text";
    let url = format!("{IMDS_ENDPOINT}/{URL}");

    let username = client
        .get(retry::Raw, url)
        .send::<String>()
        .context("failed to query IMDS for adminUsername")?
        .ok_or_else(|| anyhow!("IMDS did not return adminUsername"))?;

    let username = username.trim();
    if username.is_empty() {
        anyhow::bail!("IMDS returned an empty adminUsername");
    }
    Ok(username.to_string())
}

fn fetch_imds_ssh_keys(client: &retry::Client) -> Result<Vec<String>> {
    const URL: &str = "metadata/instance/compute/publicKeys?api-version=2021-02-01";
    let url = format!("{IMDS_ENDPOINT}/{URL}");

    let body = client
        .get(retry::Raw, url)
        .send::<String>()
        .context("failed to query IMDS for publicKeys")?
        .ok_or_else(|| anyhow!("IMDS did not return a publicKeys payload"))?;

    let keys = super::parse_imds_public_keys(&body)?;
    Ok(keys.into_iter().map(|k| k.to_key_format()).collect())
}

fn write_fragment_file(config: &IgnitionConfig, output_dir: &str, output_file: &str) -> Result<()> {
    fs::create_dir_all(output_dir).with_context(|| format!("failed to create {output_dir}"))?;
    let json =
        serde_json::to_string_pretty(config).context("failed to serialize ignition config")?;
    fs::write(output_file, json.as_bytes())
        .with_context(|| format!("failed to write {output_file}"))?;
    fs::set_permissions(Path::new(output_file), fs::Permissions::from_mode(0o644))
        .with_context(|| format!("failed to set permissions on {output_file}"))?;
    Ok(())
}

/// Generate and write an Azure Ignition config fragment.
pub(crate) fn generate() -> Result<()> {
    let imds = imds_client().context("failed to initialize IMDS client")?;
    let username = fetch_os_profile_username(&imds)?;
    let ssh_keys = fetch_imds_ssh_keys(&imds)?;

    validate_ovf_admin_password_policy()?;

    let config = IgnitionConfig {
        ignition: IgnitionMeta {
            version: IGNITION_VERSION.to_string(),
        },
        passwd: Some(Passwd {
            users: vec![PasswdUser {
                name: username,
                ssh_authorized_keys: if ssh_keys.is_empty() {
                    None
                } else {
                    Some(ssh_keys)
                },
            }],
        }),
    };

    write_fragment_file(&config, OUTPUT_DIR, OUTPUT_FILE)?;
    info!("wrote ignition fragment"; "path" => OUTPUT_FILE);
    Ok(())
}

/// OVF is optional; if present, only `adminPassword` is consulted.
fn validate_ovf_admin_password_policy() -> Result<()> {
    let xml = match mount_and_read_ovf() {
        Ok(s) => s,
        Err(e) => {
            warn!("could not read OVF media: {}", e);
            return Ok(());
        }
    };

    let env = parse_ovf_env(&xml).context("failed to parse OVF provisioning data")?;
    if !env
        .provisioning_section
        .linux_prov_conf_set
        .admin_password
        .trim()
        .is_empty()
    {
        anyhow::bail!("OVF contains a non-empty adminPassword, which is not supported");
    }
    Ok(())
}

fn mount_and_read_ovf() -> Result<String> {
    let device = Path::new(MOUNT_DEVICE);
    let mount_point = Path::new(MOUNT_POINT);

    fs::create_dir_all(mount_point)?;
    fs::set_permissions(mount_point, fs::Permissions::from_mode(0o700))?;

    let mut mounted = false;
    for fstype in CDROM_FS_TYPES {
        if util::mount_ro(device, mount_point, fstype, MOUNT_RETRIES).is_ok() {
            mounted = true;
            break;
        }
    }
    if !mounted {
        anyhow::bail!(
            "failed to mount {MOUNT_DEVICE} (tried {:?})",
            CDROM_FS_TYPES
        );
    }

    let result = fs::read_to_string(mount_point.join("ovf-env.xml"));
    let _ = util::unmount(mount_point, MOUNT_RETRIES);
    result.context("failed to read ovf-env.xml")
}

fn parse_ovf_env(xml: &str) -> Result<OvfEnvironment> {
    // Strip Azure `wa:` namespace prefixes so serde-xml-rs can match element names.
    let clean = xml.replace("<wa:", "<").replace("</wa:", "</");
    let env: OvfEnvironment = serde_xml_rs::from_str(&clean).context("failed to parse OVF XML")?;
    Ok(env)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ignition_json_with_keys() {
        let cfg = IgnitionConfig {
            ignition: IgnitionMeta {
                version: "3.4.0".into(),
            },
            passwd: Some(Passwd {
                users: vec![PasswdUser {
                    name: "testuser".into(),
                    ssh_authorized_keys: Some(vec!["ssh-ed25519 AAAA...".into()]),
                }],
            }),
        };
        let v: serde_json::Value =
            serde_json::from_str(&serde_json::to_string_pretty(&cfg).unwrap()).unwrap();
        assert_eq!(v["ignition"]["version"], "3.4.0");
        assert_eq!(v["passwd"]["users"][0]["name"], "testuser");
        assert_eq!(
            v["passwd"]["users"][0]["sshAuthorizedKeys"][0],
            "ssh-ed25519 AAAA..."
        );
    }

    #[test]
    fn test_ignition_json_no_keys() {
        let cfg = IgnitionConfig {
            ignition: IgnitionMeta {
                version: "3.4.0".into(),
            },
            passwd: Some(Passwd {
                users: vec![PasswdUser {
                    name: "azureuser".into(),
                    ssh_authorized_keys: None,
                }],
            }),
        };
        let v: serde_json::Value =
            serde_json::from_str(&serde_json::to_string_pretty(&cfg).unwrap()).unwrap();
        assert_eq!(v["passwd"]["users"][0]["name"], "azureuser");
        assert!(v["passwd"]["users"][0].get("sshAuthorizedKeys").is_none());
    }

    #[test]
    fn test_ovf_parse_admin_password() {
        let xml = r#"
<Environment xmlns="http://schemas.dmtf.org/ovf/environment/1"
    xmlns:wa="http://schemas.microsoft.com/windowsazure">
    <wa:ProvisioningSection>
        <wa:Version>1.0</wa:Version>
        <LinuxProvisioningConfigurationSet>
            <AdminPassword></AdminPassword>
        </LinuxProvisioningConfigurationSet>
    </wa:ProvisioningSection>
</Environment>"#;
        let env = parse_ovf_env(xml).unwrap();
        assert_eq!(
            env.provisioning_section
                .linux_prov_conf_set
                .admin_password
                .as_str(),
            ""
        );
    }

    #[test]
    fn test_ovf_parse_supports_lowercase_admin_password_tag() {
        let xml = r#"
<Environment xmlns="http://schemas.dmtf.org/ovf/environment/1"
    xmlns:wa="http://schemas.microsoft.com/windowsazure">
    <wa:ProvisioningSection>
        <wa:Version>1.0</wa:Version>
        <LinuxProvisioningConfigurationSet>
            <adminPassword></adminPassword>
        </LinuxProvisioningConfigurationSet>
    </wa:ProvisioningSection>
</Environment>"#;
        let env = parse_ovf_env(xml).unwrap();
        assert_eq!(
            env.provisioning_section
                .linux_prov_conf_set
                .admin_password
                .as_str(),
            ""
        );
    }

    #[test]
    fn test_parse_imds_public_keys_rejects_malformed_key() {
        let body = r#"
[
    {
        "keyData": "not-an-ssh-key",
        "path": "/home/core/.ssh/authorized_keys"
    }
]
"#;

        let err = super::super::parse_imds_public_keys(body).unwrap_err();
        let message = format!("{err:#}");
        assert!(message.contains("failed to parse IMDS public key"));
        assert!(message.contains("/home/core/.ssh/authorized_keys"));
    }

    #[test]
    fn test_parse_imds_public_keys_accepts_valid_key() {
        let body = r#"
[
    {
        "keyData": "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAAAgQDYVEprvtYJXVOBN0XNKVVRNCRX6BlnNbI+USLGais1sUWPwtSg7z9K9vhbYAPUZcq8c/s5S9dg5vTHbsiyPCIDOKyeHba4MUJq8Oh5b2i71/3BISpyxTBH/uZDHdslW2a+SrPDCeuMMoss9NFhBdKtDkdG9zyi0ibmCP6yMdEX8Q== Generated by Nova",
        "path": "/home/core/.ssh/authorized_keys"
    }
]
"#;

        let keys = super::super::parse_imds_public_keys(body).unwrap();
        assert_eq!(keys.len(), 1);
        assert!(keys[0]
            .to_key_format()
            .starts_with("ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAAAgQDYVEprvtYJ"));
    }

    #[test]
    fn test_write_fragment_file_emits_valid_json_and_permissions() {
        let tmp = tempfile::tempdir().unwrap();
        let out_dir = tmp.path().join("base.platform.d/azure");
        let out_file = out_dir.join("extensions.ign");

        let cfg = IgnitionConfig {
            ignition: IgnitionMeta {
                version: "3.4.0".into(),
            },
            passwd: Some(Passwd {
                users: vec![PasswdUser {
                    name: "core".into(),
                    ssh_authorized_keys: Some(vec![
                        "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAAAgQDYVEprvtYJXVOBN0XNKVVRNCRX6BlnNbI+USLGais1sUWPwtSg7z9K9vhbYAPUZcq8c/s5S9dg5vTHbsiyPCIDOKyeHba4MUJq8Oh5b2i71/3BISpyxTBH/uZDHdslW2a+SrPDCeuMMoss9NFhBdKtDkdG9zyi0ibmCP6yMdEX8Q== Generated by Nova".into(),
                    ]),
                }],
            }),
        };

        write_fragment_file(&cfg, out_dir.to_str().unwrap(), out_file.to_str().unwrap()).unwrap();

        assert!(out_file.exists());

        let raw = fs::read_to_string(&out_file).unwrap();
        let json: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(json["ignition"]["version"], "3.4.0");
        assert_eq!(json["passwd"]["users"][0]["name"], "core");

        let mode = fs::metadata(&out_file).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o644);
    }
}
