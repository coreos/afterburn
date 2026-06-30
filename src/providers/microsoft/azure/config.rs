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
//! Generates per-feature `.ign` fragment files (hostname, user) into a
//! directory specified by `--render-ignition-dir`.
//! OVF data is consulted for `adminPassword`.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use slog_scope::{info, warn};
use std::ffi::{c_char, CStr, CString};
use std::fs;
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use xml::reader::{EventReader, XmlEvent as ReadEvent};
use xml::writer::{EmitterConfig, XmlEvent as WriteEvent};

use crate::util;

const IGNITION_VERSION: &str = "3.0.0";

const MOUNT_DEVICE: &str = "/dev/sr0";
const MOUNT_POINT: &str = "/run/afterburn/media/";
const CDROM_FS_TYPES: &[&str] = &["udf", "iso9660"];
const MOUNT_RETRIES: u8 = 3;
const PASSWORD_HASH_ROUNDS: usize = 10_000;

const SALT_ALPHABET: &[u8; 64] =
    b"./0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
const SALT_LEN: usize = 16;

// Sizes from libxcrypt's <crypt.h>.
const CRYPT_OUTPUT_SIZE: usize = 384;
const CRYPT_MAX_PASSPHRASE_SIZE: usize = 512;
const CRYPT_DATA_RESERVED_SIZE: usize = 767;
const CRYPT_DATA_INTERNAL_SIZE: usize = 30720;

#[repr(C)]
struct CryptData {
    output: [c_char; CRYPT_OUTPUT_SIZE],
    setting: [c_char; CRYPT_OUTPUT_SIZE],
    input: [c_char; CRYPT_MAX_PASSPHRASE_SIZE],
    reserved: [c_char; CRYPT_DATA_RESERVED_SIZE],
    initialized: c_char,
    internal: [c_char; CRYPT_DATA_INTERNAL_SIZE],
}

type CryptRFn = unsafe extern "C" fn(
    phrase: *const c_char,
    setting: *const c_char,
    data: *mut CryptData,
) -> *mut c_char;

/// Resolve libxcrypt's `crypt_r` at runtime so the binary doesn't bake in a
/// specific libcrypt soname; Fedora/RHEL ship `libcrypt.so.2`, Debian/Ubuntu
/// ship `libcrypt.so.1`.
fn load_crypt_r() -> Result<CryptRFn> {
    use std::sync::OnceLock;
    static CACHED: OnceLock<std::result::Result<CryptRFn, String>> = OnceLock::new();

    match CACHED.get_or_init(|| {
        const CANDIDATES: &[&CStr] = &[c"libcrypt.so.2", c"libcrypt.so.1"];
        let mut last_err = "no libcrypt candidates available".to_string();
        for soname in CANDIDATES {
            let handle =
                unsafe { libc::dlopen(soname.as_ptr(), libc::RTLD_NOW | libc::RTLD_LOCAL) };
            if handle.is_null() {
                last_err = format!("dlopen({}) failed", soname.to_string_lossy());
                continue;
            }
            let sym = unsafe { libc::dlsym(handle, c"crypt_r".as_ptr()) };
            if sym.is_null() {
                last_err = format!("{} did not export crypt_r", soname.to_string_lossy());
                continue;
            }
            let func: CryptRFn = unsafe { std::mem::transmute(sym) };
            return Ok(func);
        }
        Err(last_err)
    }) {
        Ok(func) => Ok(*func),
        Err(msg) => anyhow::bail!("failed to load crypt_r: {msg}"),
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

pub(crate) fn generate_hostname_fragment(
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

pub(crate) fn generate_user_fragment(
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

    let password_hash = read_ovf_admin_password()?
        .map(|password| hash_admin_password(&password))
        .transpose()?;

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

/// OVF is optional; if present, only `adminPassword` is consulted.
fn read_ovf_admin_password() -> Result<Option<String>> {
    let xml = match mount_and_read_ovf() {
        Ok(s) => s,
        Err(e) => {
            warn!("could not read OVF media: {}", e);
            return Ok(None);
        }
    };

    let env = parse_ovf_env(&xml).context("failed to parse OVF provisioning data")?;
    let admin_password = env.provisioning_section.linux_prov_conf_set.admin_password;

    if admin_password.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some(admin_password))
    }
}

fn hash_admin_password(password: &str) -> Result<String> {
    let salt = generate_salt().context("failed to generate password salt")?;
    sha512_crypt(password, &salt, PASSWORD_HASH_ROUNDS)
}

fn generate_salt() -> Result<String> {
    let mut bytes = [0u8; SALT_LEN];
    let mut urandom = fs::File::open("/dev/urandom").context("failed to open /dev/urandom")?;
    urandom
        .read_exact(&mut bytes)
        .context("failed to read random bytes for salt")?;
    Ok(bytes
        .iter()
        .map(|b| SALT_ALPHABET[(*b as usize) % SALT_ALPHABET.len()] as char)
        .collect())
}

fn sha512_crypt(password: &str, salt: &str, rounds: usize) -> Result<String> {
    let setting = CString::new(format!("$6$rounds={rounds}${salt}"))
        .context("crypt setting contains a NUL byte")?;
    let phrase = CString::new(password).context("password contains a NUL byte")?;

    let crypt_r = load_crypt_r()?;

    // CryptData is ~32 KiB; box it to avoid a large stack frame.
    let mut data: Box<CryptData> = unsafe { Box::new(std::mem::zeroed()) };

    let out_ptr = unsafe { crypt_r(phrase.as_ptr(), setting.as_ptr(), &mut *data) };
    if out_ptr.is_null() {
        anyhow::bail!("crypt_r failed: {}", std::io::Error::last_os_error());
    }
    let out = unsafe { CStr::from_ptr(out_ptr) }
        .to_str()
        .context("crypt_r output was not valid UTF-8")?;
    // libxcrypt signals failure with a leading '*'.
    if out.starts_with('*') {
        anyhow::bail!("crypt_r reported a hashing failure: {out}");
    }
    Ok(out.to_owned())
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
    let clean = strip_xml_namespaces(xml).context("failed to strip XML namespaces")?;
    let env: OvfEnvironment = serde_xml_rs::from_str(&clean).context("failed to parse OVF XML")?;
    Ok(env)
}

/// Strip namespace prefixes from XML elements/attributes via structural parsing.
fn strip_xml_namespaces(xml: &str) -> Result<String> {
    let reader = EventReader::from_str(xml);
    let mut output = Vec::new();
    let mut writer = EmitterConfig::new()
        .perform_indent(false)
        .write_document_declaration(false)
        .create_writer(&mut output);

    for event in reader {
        let event = event.context("failed to read XML event")?;
        match event {
            ReadEvent::StartElement {
                name, attributes, ..
            } => {
                let mut elem = WriteEvent::start_element(name.local_name.as_str());
                let filtered_attrs: Vec<_> = attributes
                    .iter()
                    .filter(|a| {
                        a.name.prefix.as_deref() != Some("xmlns") && a.name.local_name != "xmlns"
                    })
                    .collect();
                for attr in &filtered_attrs {
                    elem = elem.attr(attr.name.local_name.as_str(), &attr.value);
                }
                writer
                    .write(elem)
                    .context("failed to write XML start element")?;
            }
            ReadEvent::EndElement { .. } => {
                writer
                    .write(WriteEvent::end_element())
                    .context("failed to write XML end element")?;
            }
            ReadEvent::Characters(text) => {
                writer
                    .write(WriteEvent::characters(&text))
                    .context("failed to write XML characters")?;
            }
            ReadEvent::CData(text) => {
                writer
                    .write(WriteEvent::cdata(&text))
                    .context("failed to write XML CDATA")?;
            }
            _ => {}
        }
    }
    String::from_utf8(output).context("XML output is not valid UTF-8")
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_ovf_parse_non_empty_admin_password() {
        let xml = r#"
<Environment xmlns="http://schemas.dmtf.org/ovf/environment/1"
    xmlns:wa="http://schemas.microsoft.com/windowsazure">
    <wa:ProvisioningSection>
        <wa:Version>1.0</wa:Version>
        <LinuxProvisioningConfigurationSet>
            <AdminPassword>SecretPassword123!</AdminPassword>
        </LinuxProvisioningConfigurationSet>
    </wa:ProvisioningSection>
</Environment>"#;
        let env = parse_ovf_env(xml).unwrap();
        assert_eq!(
            env.provisioning_section
                .linux_prov_conf_set
                .admin_password
                .as_str(),
            "SecretPassword123!"
        );
    }

    #[test]
    fn test_hash_admin_password_emits_sha512_crypt() {
        let hash = hash_admin_password("SecretPassword123!").unwrap();

        assert!(hash.starts_with("$6$rounds=10000$"));
        let parts: Vec<&str> = hash.splitn(5, '$').collect();
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[1], "6");
        assert_eq!(parts[2], "rounds=10000");
        assert_eq!(parts[3].len(), SALT_LEN, "salt should be 16 chars");
        assert!(
            parts[3].bytes().all(|b| SALT_ALPHABET.contains(&b)),
            "salt should only contain crypt(3) alphabet chars"
        );
        assert!(!parts[4].is_empty(), "hash should be non-empty");
    }

    #[test]
    fn test_sha512_crypt_is_deterministic_for_fixed_salt() {
        let a = sha512_crypt("hunter2", "abcdefghijklmnop", 10_000).unwrap();
        let b = sha512_crypt("hunter2", "abcdefghijklmnop", 10_000).unwrap();
        assert_eq!(a, b);
        assert!(a.starts_with("$6$rounds=10000$abcdefghijklmnop$"));
    }

    #[test]
    fn test_sha512_crypt_distinct_salts_produce_distinct_hashes() {
        let a = sha512_crypt("hunter2", "aaaaaaaaaaaaaaaa", 10_000).unwrap();
        let b = sha512_crypt("hunter2", "bbbbbbbbbbbbbbbb", 10_000).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn test_sha512_crypt_distinct_passwords_produce_distinct_hashes() {
        // Holding salt and rounds constant, the password input must
        // affect the digest. Guards against the FFI silently dropping
        // the phrase argument.
        let a = sha512_crypt("hunter2", "saltsaltsaltsalt", 10_000).unwrap();
        let b = sha512_crypt("hunter1", "saltsaltsaltsalt", 10_000).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn test_sha512_crypt_distinct_rounds_produce_distinct_hashes() {
        // The rounds parameter must be wired through to crypt_r and
        // reflected in both the digest and the setting prefix.
        let a = sha512_crypt("hunter2", "saltsaltsaltsalt", 5_000).unwrap();
        let b = sha512_crypt("hunter2", "saltsaltsaltsalt", 10_000).unwrap();
        assert_ne!(a, b);
        assert!(a.starts_with("$6$rounds=5000$saltsaltsaltsalt$"));
        assert!(b.starts_with("$6$rounds=10000$saltsaltsaltsalt$"));
    }

    #[test]
    fn test_sha512_crypt_matches_drepper_kat_vector() {
        // Known-answer test from Ulrich Drepper's SHA-crypt
        // specification (test vector with rounds=10000). Confirms our
        // libxcrypt FFI shim emits the standard crypt(3) digest that
        // `/etc/shadow` / PAM expects.
        let hash = sha512_crypt("Hello world!", "saltstringsaltst", 10_000).unwrap();
        assert_eq!(
            hash,
            "$6$rounds=10000$saltstringsaltst$OW1/O6BYHV6BcXZu8QVeXbDWra3Oeqh0sbHbbMCVNSnCM/UrjmM0Dp8vOuZeHBy/YTBmSK6H9qs/y3RnOaw5v."
        );
    }

    #[test]
    fn test_sha512_crypt_rejects_nul_in_password() {
        // CString::new must reject embedded NUL bytes before we hand
        // the phrase to crypt_r; we should surface a hard error, not
        // truncate silently.
        assert!(sha512_crypt("bad\0pass", "saltsaltsaltsalt", 10_000).is_err());
    }

    #[test]
    fn test_sha512_crypt_rejects_nul_in_salt() {
        // Same guarantee as for the password, applied to the salt
        // argument that is interpolated into the setting string.
        assert!(sha512_crypt("hunter2", "bad\0salt12345678", 10_000).is_err());
    }

    #[test]
    fn test_generate_salt_alphabet_and_length() {
        for _ in 0..16 {
            let salt = generate_salt().unwrap();
            assert_eq!(salt.len(), SALT_LEN);
            assert!(salt.bytes().all(|b| SALT_ALPHABET.contains(&b)));
        }
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
