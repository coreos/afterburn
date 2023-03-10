//! VMware provider on x86_64.
//!
//! This uses the guest->host backdoor protocol for introspection.

use super::VmwareProvider;
use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose, Engine as _};
use libflate::gzip::Decoder;
use serde_json::json;
use std::io::Read;

/// Guestinfo key for network kargs.
static INITRD_NET_KARGS: &str = "guestinfo.afterburn.initrd.network-kargs";
static METADATA: &str = "guestinfo.metadata";
static METADATA_ENCODING: &str = "guestinfo.metadata.encoding";

impl VmwareProvider {
    /// Build the VMware provider, fetching and caching guestinfo entries.
    pub fn try_new() -> Result<Self> {
        if !vmw_backdoor::is_vmware_cpu() {
            bail!("not running on VMWare CPU");
        }

        // NOTE(lucab): privileged mode is in theory more reliable but
        //  `kernel_lockdown(7)` may block it due to `iopl()` usage.
        //  Thus, we try that first and fall back if kernel blocks it.
        let mut backdoor = vmw_backdoor::probe_backdoor_privileged().or_else(|e| {
            slog_scope::warn!("failed to probe backdoor in privileged mode: {}", e);
            slog_scope::warn!("falling back to unprivileged backdoor access");
            vmw_backdoor::probe_backdoor()
        })?;

        let guestinfo_net_kargs = {
            // Use a block, otherwise we would have to drop(erpc) manually
            let mut erpc = vmw_backdoor::EnhancedChan::open(&mut backdoor)?;
            Self::fetch_guestinfo(&mut erpc, INITRD_NET_KARGS)?
        };

        let guestinfo_metadata_raw = {
            let mut erpc = vmw_backdoor::EnhancedChan::open(&mut backdoor)?;
            Self::fetch_guestinfo(&mut erpc, METADATA)?
        };

        let guestinfo_metadata_encoding = {
            let mut erpc = vmw_backdoor::EnhancedChan::open(&mut backdoor)?;
            Self::fetch_guestinfo(&mut erpc, METADATA_ENCODING)?
        };

        let guestinfo_metadata =
            parse_metadata(guestinfo_metadata_encoding, guestinfo_metadata_raw)?;

        let provider = Self {
            guestinfo_net_kargs,
            guestinfo_metadata,
        };

        slog_scope::trace!("cached vmware provider: {:?}", provider);
        Ok(provider)
    }

    /// Retrieve the value of a guestinfo string property, by key.
    fn fetch_guestinfo(erpc: &mut vmw_backdoor::EnhancedChan, key: &str) -> Result<Option<String>> {
        let guestinfo = erpc
            .get_guestinfo(key.as_bytes())
            .with_context(|| format!("failed to retrieve guestinfo for {}", key))?
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned());
        Ok(guestinfo)
    }

    pub fn parse_netplan_config(&self) -> Result<Option<String>> {
        if let Some(metadata) = &self.guestinfo_metadata {
            // We need to parse the netplan config to remove the non-Netplan keys.
            // The data can either be JSON or YAML, but since JSON is a subset of
            // YAML we don't need to try serde_json::from_str here.
            let netplan_config_unfiltered: serde_json::Value =
                serde_yaml::from_str(metadata).context("invalid YAML/JSON metadata")?;
            // Only the "network" key is allowed to be present.
            // We use the json! macro but this is only about creating a serde value::Value,
            // even though its name sounds like it would create JSON.
            let netplan_config_filtered = json!({
                "network": netplan_config_unfiltered.get("network").context("no 'network' key found")?
            });
            Ok(Some(serde_yaml::to_string(&netplan_config_filtered)?))
        } else {
            Ok(None)
        }
    }

    #[cfg(test)]
    pub fn new_from_metadata(metadata: String) -> Result<Self> {
        Ok(Self {
            guestinfo_net_kargs: None,
            guestinfo_metadata: Some(metadata),
        })
    }
}

fn parse_metadata(
    guestinfo_metadata_encoding: Option<String>,
    guestinfo_metadata_raw: Option<String>,
) -> Result<Option<String>> {
    match (
        guestinfo_metadata_encoding.as_deref(),
        guestinfo_metadata_raw,
    ) {
        (Some("base64" | "b64"), Some(guestinfo_metadata_raw_val)) => {
            let decoded =
                general_purpose::STANDARD.decode(guestinfo_metadata_raw_val.as_bytes())?;
            Ok(Some(String::from_utf8(decoded)?))
        }
        (Some("gzip+base64" | "gz+b64"), Some(guestinfo_metadata_raw_val)) => {
            let decoded =
                general_purpose::STANDARD.decode(guestinfo_metadata_raw_val.as_bytes())?;
            let mut decompressor = Decoder::new(decoded.as_slice())?;
            let mut uncompressed = Vec::new();
            decompressor.read_to_end(&mut uncompressed)?;
            Ok(Some(String::from_utf8(uncompressed)?))
        }
        (Some(""), guestinfo_metadata_raw) => Ok(guestinfo_metadata_raw),
        (Some(encoding), _) => bail!("unknown guestinfo.metadata.encoding '{}'", encoding),
        (None, guestinfo_metadata_raw) => Ok(guestinfo_metadata_raw),
    }
}

#[test]
fn test_netplan_json() {
    let metadata = r#"{
      "network": {
        "ethernets": {
          "nics": {
            "match": {
              "name": "ens*"
            }
          }
        }
      },
      "ExcludeNonNetplanField": 0
    }"#;
    let provider = VmwareProvider::new_from_metadata(metadata.to_owned()).unwrap();
    let netplan_config = provider.parse_netplan_config().unwrap().unwrap();
    let expected = r#"network:
  ethernets:
    nics:
      match:
        name: ens*
"#;
    assert_eq!(netplan_config, expected);
}

#[test]
fn test_netplan_dhcp() {
    let metadata = r#"network:
  ethernets:
    nics:
      match:
        name: ens*
"#;
    let provider = VmwareProvider::new_from_metadata(metadata.to_owned()).unwrap();
    let netplan_config = provider.parse_netplan_config().unwrap().unwrap();
    assert_eq!(netplan_config, metadata);
}

#[test]
fn test_metadata_plain_1() {
    let guestinfo_metadata_raw = Some("hello".to_owned());
    let parsed = parse_metadata(None, guestinfo_metadata_raw)
        .unwrap()
        .unwrap();
    assert_eq!(parsed, "hello");
}

#[test]
fn test_metadata_plain_2() {
    let guestinfo_metadata_raw = Some("hello".to_owned());
    let parsed = parse_metadata(Some("".into()), guestinfo_metadata_raw)
        .unwrap()
        .unwrap();
    assert_eq!(parsed, "hello");
}

#[test]
fn test_metadata_base64() {
    let guestinfo_metadata_raw = Some("aGVsbG8=".to_owned());
    let parsed = parse_metadata(Some("base64".into()), guestinfo_metadata_raw.clone())
        .unwrap()
        .unwrap();
    assert_eq!(parsed, "hello");
    let parsed_b64 = parse_metadata(Some("b64".into()), guestinfo_metadata_raw)
        .unwrap()
        .unwrap();
    assert_eq!(parsed_b64, "hello");
}

#[test]
fn test_metadata_gzip_base64() {
    let guestinfo_metadata_raw = Some("H4sIAAAAAAACA8tIzcnJBwCGphA2BQAAAA==".to_owned());
    let parsed = parse_metadata(Some("gzip+base64".into()), guestinfo_metadata_raw.clone())
        .unwrap()
        .unwrap();
    assert_eq!(parsed, "hello");
    let parsed_b64 = parse_metadata(Some("gz+b64".into()), guestinfo_metadata_raw)
        .unwrap()
        .unwrap();
    assert_eq!(parsed_b64, "hello");
}
