//! Metadata fetcher for KubeVirt instances.
//!
//! This provider supports platforms based on KubeVirt.
//! It provides a config-drive as the only metadata source, whose layout
//! follows the `cloud-init ConfigDrive v2` [datasource][configdrive], with
//! the following details:
//!  - disk filesystem label is `config-2` (lowercase)
//!  - filesystem is `iso9660`
//!  - drive contains a single directory at `/openstack/latest/`
//!  - content is exposed as JSON files called `meta_data.json`.
//!
//! configdrive: https://cloudinit.readthedocs.io/en/latest/topics/datasources/configdrive.html

use anyhow::{bail, Context, Result};
use openssh_keys::PublicKey;
use serde::Deserialize;
use slog_scope::warn;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::{process::Command, path::{Path, PathBuf}};
use std::str::FromStr;
use std::net::{AddrParseError, IpAddr};
use tempfile::TempDir;
use ipnetwork::IpNetwork;
use pnet_base::MacAddr;

use crate::network::{self, DhcpSetting, NetworkRoute};
use crate::providers::MetadataProvider;

// Filesystem label for the Config Drive.
static CONFIG_DRIVE_FS_LABEL: &str = "config-2";

// Filesystem type for the Config Drive.
static CONFIG_DRIVE_FS_TYPE: &str = "iso9660";

///KubeVirt provider.
#[derive(Debug)]
pub struct KubeVirtProvider {
    /// Path to the top directory of the mounted config-drive.
    drive_path: PathBuf,
    /// Temporary directory for own mountpoint.
    temp_dir: TempDir,
}

/// Partial object for `meta_data.json`
#[derive(Debug, Deserialize)]
pub struct MetaDataJSON {
    /// Local hostname
    pub hostname: String,
    /// Instance ID (UUID).
    #[serde(rename = "uuid")]
    pub instance_id: String,
    /// Instance type.
    pub instance_type: Option<String>,
    /// SSH public keys.
    pub public_keys: Option<HashMap<String, String>>,
}

/// Partial object for `network_data.json`
#[derive(Debug, Deserialize)]
pub struct NetworkDataJSON {
    pub version: u32,
    pub config: Vec<NetworkConfigEntry>,
}

/// JSON entry in `config` array.
#[derive(Debug, Deserialize)]
pub struct NetworkConfigEntry {
    #[serde(rename = "type")]
    pub network_type: String,
    pub name: Option<String>,
    pub mac_address: Option<String>,
    #[serde(default)]
    pub address: Vec<String>,
    #[serde(default)]
    pub subnets: Vec<NetworkConfigSubnet>,
}

/// JSON entry in `config.subnets` array.
#[derive(Debug, Deserialize)]
pub struct NetworkConfigSubnet {
    #[serde(rename = "type")]
    pub subnet_type: String,
    pub address: Option<String>,
    pub netmask: Option<String>,
    pub gateway: Option<String>,
}

impl KubeVirtProvider {
    fn find_config_device() -> Result<String> {
        // Diagnostic commands to understand the environment
        slog_scope::info!("Starting config device detection diagnostics");

        // Check available vd devices
        if let Ok(output) = Command::new("ls").args(["-la", "/dev/vd*"]).output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            slog_scope::info!("Available vd devices: {}", stdout.trim());
        }

        // Check available sr devices
        if let Ok(output) = Command::new("ls").args(["-la", "/dev/sr*"]).output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            slog_scope::info!("Available sr devices: {}", stdout.trim());
        }

        // Check partition table
        if let Ok(output) = Command::new("cat").arg("/proc/partitions").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            slog_scope::info!("Partition table: {}", stdout.trim());
        }

        // Check sysfs block devices
        if let Ok(output) = Command::new("ls").args(["-la", "/sys/block/"]).output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            slog_scope::info!("Block devices in sysfs: {}", stdout.trim());
        }

        // Check all block devices without filter
        if let Ok(output) = Command::new("blkid").args(["--cache-file", "/dev/null"]).output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            slog_scope::info!("All blkid devices - stdout: {}, stderr: {}", stdout.trim(), stderr.trim());
        }

        // Try with retry and sleep for label-based detection (common in initrd timing issues)
        const MAX_RETRIES: u32 = 5;
        const SLEEP_DURATION_MS: u64 = 1000;

        for attempt in 1..=MAX_RETRIES {
            slog_scope::info!("Attempt {} to find config device with label {}", attempt, CONFIG_DRIVE_FS_LABEL);

            let output = Command::new("blkid")
                .args(["--cache-file", "/dev/null", "-L", CONFIG_DRIVE_FS_LABEL])
                .output()
                .context("failed to execute blkid command")?;

            if output.status.success() {
                let device = String::from_utf8_lossy(&output.stdout).trim().to_string();
                slog_scope::info!("Found config device: {}", device);
                return Ok(device);
            }

            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            slog_scope::warn!("Attempt {} failed - exit code: {}, stdout: {}, stderr: {}",
                            attempt, output.status.code().unwrap_or(-1), stdout.trim(), stderr.trim());

            if attempt < MAX_RETRIES {
                slog_scope::info!("Sleeping {}ms before retry", SLEEP_DURATION_MS);
                std::thread::sleep(std::time::Duration::from_millis(SLEEP_DURATION_MS));
            }
        }

        // Final diagnostic: try to examine specific devices directly
        for device in ["/dev/vdb", "/dev/sr0", "/dev/sr1"] {
            if std::path::Path::new(device).exists() {
                slog_scope::info!("Checking device {} directly", device);

                if let Ok(output) = Command::new("blkid").args(["--cache-file", "/dev/null", device]).output() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    slog_scope::info!("Device {} blkid output: {}", device, stdout.trim());
                }

                if let Ok(output) = Command::new("file").args(["-s", device]).output() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    slog_scope::info!("Device {} file output: {}", device, stdout.trim());
                }
            }
        }

        bail!("could not find config device after {} attempts", MAX_RETRIES)
    }

    /// Try to build a new provider client.
    ///
    /// This internally tries to mount (and own) the config-drive.
    pub fn try_new() -> Result<Self> {
        let target = tempfile::Builder::new()
            .prefix("afterburn-")
            .tempdir()
            .context("failed to create temporary directory")?;

        let device_path = Self::find_config_device()?;

        crate::util::mount_ro(
            Path::new(&device_path),
            target.path(),
            CONFIG_DRIVE_FS_TYPE,
            3, // maximum retries
        )?;

        let provider = Self {
            drive_path: target.path().to_owned(),
            temp_dir: target,
        };
        Ok(provider)
    }

    /// Return the path to the metadata directory.
    fn metadata_dir(&self) -> PathBuf {
        let drive = self.drive_path.clone();
        drive.join("openstack").join("latest")
    }

    /// Read and parse metadata file.
    fn read_metadata(&self) -> Result<MetaDataJSON> {
        let filename = self.metadata_dir().join("meta_data.json");
        let file =
            File::open(&filename).with_context(|| format!("failed to open file '{filename:?}'"))?;
        let bufrd = BufReader::new(file);
        Self::parse_metadata(bufrd)
    }

    /// Parse metadata attributes.
    ///
    /// Metadata file contains a JSON object, corresponding to `MetaDataJSON`.
    fn parse_metadata<T: Read>(input: BufReader<T>) -> Result<MetaDataJSON> {
        serde_json::from_reader(input).context("failed to parse JSON metadata")
    }

    /// Read and parse network configuration.
    fn read_network_data(&self) -> Result<NetworkDataJSON> {
        let filename = self.metadata_dir().join("network_data.json");
        let file =
            File::open(&filename).with_context(|| format!("failed to open file '{filename:?}'"))?;
        let bufrd = BufReader::new(file);
        Self::parse_network_data(bufrd)
    }

    /// Parse network configuration.
    ///
    /// Network configuration file contains a JSON object, corresponding to `NetworkDataJSON`.
    fn parse_network_data<T: Read>(input: BufReader<T>) -> Result<NetworkDataJSON> {
        serde_json::from_reader(input).context("failed to parse JSON network data")
    }

    /// Transform network JSON data into a set of interface configurations.
    fn network_interfaces(input: NetworkDataJSON) -> Result<Vec<network::Interface>> {
        let nameservers = input
            .config
            .iter()
            .filter(|config| config.network_type == "nameserver")
            .collect::<Vec<_>>();

        if nameservers.len() > 1 {
            return Err(anyhow::anyhow!("too many nameservers, only one supported"));
        }

        let mut interfaces = input
            .config
            .iter()
            .filter(|config| config.network_type == "physical")
            .map(|entry| entry.to_interface())
            .collect::<Result<Vec<_>, _>>()?;

        if let Some(iface) = interfaces.first_mut() {
            if let Some(nameserver) = nameservers.first() {
                iface.nameservers = nameserver
                    .address
                    .iter()
                    .map(|ip| IpAddr::from_str(ip))
                    .collect::<Result<Vec<IpAddr>, AddrParseError>>()?;
            }
        }

        Ok(interfaces)
    }

    /// Extract supported metadata values and convert to Afterburn attributes.
    ///
    /// The `AFTERBURN_` prefix is added later on, so it is not part of the
    /// key-labels here.
    fn known_attributes(metadata: MetaDataJSON) -> Result<HashMap<String, String>> {
        if metadata.instance_id.is_empty() {
            bail!("empty instance ID");
        }

        if metadata.hostname.is_empty() {
            bail!("empty local hostname");
        }

        let mut attrs = maplit::hashmap! {
            "KUBEVIRT_INSTANCE_ID".to_string() => metadata.instance_id,
            "KUBEVIRT_HOSTNAME".to_string() => metadata.hostname,
        };
        if let Some(instance_type) = metadata.instance_type {
            attrs.insert("KUBEVIRT_INSTANCE_TYPE".to_string(), instance_type);
        }
        Ok(attrs)
    }

    /// The public key is stored as key:value pair in openstack/latest/meta_data.json file
    fn public_keys(metadata: MetaDataJSON) -> Result<Vec<PublicKey>> {
        let public_keys_map = metadata.public_keys.unwrap_or_default();
        let public_keys_vec: Vec<&std::string::String> = public_keys_map.values().collect();
        let mut out = vec![];
        for key in public_keys_vec {
            let key = PublicKey::parse(key)?;
            out.push(key);
        }
        Ok(out)
    }
}

impl MetadataProvider for KubeVirtProvider {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        let metadata = self.read_metadata()?;
        Self::known_attributes(metadata)
    }

    fn hostname(&self) -> Result<Option<String>> {
        let metadata = self.read_metadata()?;
        let hostname = if metadata.hostname.is_empty() {
            None
        } else {
            Some(metadata.hostname)
        };
        Ok(hostname)
    }

    fn ssh_keys(&self) -> Result<Vec<PublicKey>> {
        let metadata = self.read_metadata()?;
        Self::public_keys(metadata)
    }

    fn networks(&self) -> Result<Vec<network::Interface>> {
        let data = self.read_network_data()?;
        let interfaces = Self::network_interfaces(data)?;
        Ok(interfaces)
    }

    fn virtual_network_devices(&self) -> Result<Vec<network::VirtualNetDev>> {
        warn!("virtual network devices metadata requested, but not supported on this platform");
        Ok(vec![])
    }

    fn boot_checkin(&self) -> Result<()> {
        warn!("boot check-in requested, but not supported on this platform");
        Ok(())
    }

    fn rd_network_kargs(&self) -> Result<Option<String>> {
        let mut kargs = Vec::new();

        if let Ok(networks) = self.networks() {
            for iface in networks {
                // Add IP configuration if static
                for addr in iface.ip_addresses {
                    match addr {
                        IpNetwork::V4(network) => {
                            if let Some(gateway) = iface
                                .routes
                                .iter()
                                .find(|r| r.destination.is_ipv4() && r.destination.prefix() == 0)
                            {
                                kargs.push(format!(
                                    "ip={}::{}:{}",
                                    network.ip(),
                                    gateway.gateway,
                                    network.mask()
                                ));
                            } else {
                                kargs.push(format!("ip={}:::{}", network.ip(), network.mask()));
                            }
                        }
                        IpNetwork::V6(network) => {
                            if let Some(gateway) = iface
                                .routes
                                .iter()
                                .find(|r| r.destination.is_ipv6() && r.destination.prefix() == 0)
                            {
                                kargs.push(format!(
                                    "ip={}::{}:{}",
                                    network.ip(),
                                    gateway.gateway,
                                    network.prefix()
                                ));
                            } else {
                                kargs.push(format!("ip={}:::{}", network.ip(), network.prefix()));
                            }
                        }
                    }
                }

                // Add DHCP configuration
                if let Some(dhcp) = iface.dhcp {
                    match dhcp {
                        DhcpSetting::V4 => kargs.push("ip=dhcp".to_string()),
                        DhcpSetting::V6 => kargs.push("ip=dhcp6".to_string()),
                        DhcpSetting::Both => kargs.push("ip=dhcp,dhcp6".to_string()),
                    }
                }

                // Add nameservers
                if !iface.nameservers.is_empty() {
                    let nameservers = iface
                        .nameservers
                        .iter()
                        .map(|ns| ns.to_string())
                        .collect::<Vec<_>>()
                        .join(",");
                    kargs.push(format!("nameserver={}", nameservers));
                }
            }
        }

        if kargs.is_empty() {
            Ok(None)
        } else {
            Ok(Some(kargs.join(" ")))
        }
    }

}

impl Drop for KubeVirtProvider {
    fn drop(&mut self) {
        if let Err(e) = crate::util::unmount(
            self.temp_dir.path(),
            3, // maximum retries
        ) {
            slog_scope::error!("failed to unmount kubevirt config-drive: {}", e);
        };
    }
}

impl NetworkConfigEntry {
    pub fn to_interface(&self) -> Result<network::Interface> {
        if self.network_type != "physical" {
            return Err(anyhow::anyhow!(
                "cannot convert config to interface: unsupported config type \"{}\"",
                self.network_type
            ));
        }

        let mut iface = network::Interface {
            name: self.name.clone(),

            // filled later
            nameservers: vec![],
            // filled below
            ip_addresses: vec![],
            // filled below
            routes: vec![],
            // filled below
            dhcp: None,
            // filled below because Option::try_map doesn't exist yet
            mac_address: None,

            // unsupported by kubevirt
            bond: None,

            // default values
            path: None,
            priority: 20,
            unmanaged: false,
            required_for_online: None,
        };

        for subnet in &self.subnets {
            if subnet.subnet_type.contains("static") {
                if subnet.address.is_none() {
                    return Err(anyhow::anyhow!(
                        "cannot convert static subnet to interface: missing address"
                    ));
                }

                if let Some(netmask) = &subnet.netmask {
                    iface.ip_addresses.push(IpNetwork::with_netmask(
                        IpAddr::from_str(subnet.address.as_ref().unwrap())?,
                        IpAddr::from_str(netmask)?,
                    )?);
                } else {
                    iface
                        .ip_addresses
                        .push(IpNetwork::from_str(subnet.address.as_ref().unwrap())?);
                }

                if let Some(gateway) = &subnet.gateway {
                    let gateway = IpAddr::from_str(gateway)?;

                    let destination = if gateway.is_ipv6() {
                        IpNetwork::from_str("::/0")?
                    } else {
                        IpNetwork::from_str("0.0.0.0/0")?
                    };

                    iface.routes.push(NetworkRoute {
                        destination,
                        gateway,
                    });
                } else {
                    warn!("found subnet type \"static\" without gateway");
                }
            }

            if subnet.subnet_type == "dhcp" || subnet.subnet_type == "dhcp4" {
                iface.dhcp = Some(DhcpSetting::V4)
            }
            if subnet.subnet_type == "dhcp6" {
                iface.dhcp = Some(DhcpSetting::V6)
            }
            if subnet.subnet_type == "ipv6_slaac" {
                warn!("subnet type \"ipv6_slaac\" not supported, ignoring");
            }
        }

        if let Some(mac) = &self.mac_address {
            iface.mac_address = Some(MacAddr::from_str(mac)?);
        }

        Ok(iface)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_kubevirt_basic_attributes() {
        let metadata = r#"
{
  "hostname": "test_instance-kubevirt.foo.cloud",
  "uuid": "41b4fb82-ca29-11eb-b8bc-0242ac130003"
}
"#;

        let bufrd = BufReader::new(Cursor::new(metadata));
        let parsed = KubeVirtProvider::parse_metadata(bufrd).unwrap();
        assert_eq!(parsed.instance_id, "41b4fb82-ca29-11eb-b8bc-0242ac130003");
        assert_eq!(parsed.hostname, "test_instance-kubevirt.foo.cloud");

        let attrs = KubeVirtProvider::known_attributes(parsed).unwrap();
        assert_eq!(attrs.len(), 2);
        assert_eq!(
            attrs.get("KUBEVIRT_INSTANCE_ID"),
            Some(&"41b4fb82-ca29-11eb-b8bc-0242ac130003".to_string())
        );
        assert_eq!(
            attrs.get("KUBEVIRT_HOSTNAME"),
            Some(&"test_instance-kubevirt.foo.cloud".to_string())
        );
    }

    #[test]
    fn test_kubevirt_extended_attributes() {
        let metadata = r#"
{
  "hostname": "test_instance-kubevirt.foo.cloud",
  "uuid": "41b4fb82-ca29-11eb-b8bc-0242ac130003",
  "instance_type": "some_type"
}
"#;

        let bufrd = BufReader::new(Cursor::new(metadata));
        let parsed = KubeVirtProvider::parse_metadata(bufrd).unwrap();
        assert_eq!(parsed.instance_id, "41b4fb82-ca29-11eb-b8bc-0242ac130003");
        assert_eq!(parsed.hostname, "test_instance-kubevirt.foo.cloud");
        assert_eq!(parsed.instance_type.as_deref().unwrap(), "some_type");

        let attrs = KubeVirtProvider::known_attributes(parsed).unwrap();
        assert_eq!(attrs.len(), 3);
        assert_eq!(
            attrs.get("KUBEVIRT_INSTANCE_ID"),
            Some(&"41b4fb82-ca29-11eb-b8bc-0242ac130003".to_string())
        );
        assert_eq!(
            attrs.get("KUBEVIRT_HOSTNAME"),
            Some(&"test_instance-kubevirt.foo.cloud".to_string())
        );
        assert_eq!(
            attrs.get("KUBEVIRT_INSTANCE_TYPE"),
            Some(&"some_type".to_string())
        );
    }

    #[test]
    fn test_kubevirt_parse_metadata_json() {
        let fixture = File::open("./tests/fixtures/kubevirt/meta_data.json").unwrap();
        let bufrd = BufReader::new(fixture);
        let parsed = KubeVirtProvider::parse_metadata(bufrd).unwrap();

        assert!(!parsed.instance_id.is_empty());
        assert!(!parsed.hostname.is_empty());
        assert!(parsed.public_keys.is_some());
    }

    #[test]
    fn test_kubevirt_ssh_keys() {
        let fixture = File::open("./tests/fixtures/kubevirt/meta_data.json").unwrap();

        let bufrd = BufReader::new(fixture);
        let parsed = KubeVirtProvider::parse_metadata(bufrd).unwrap();
        let keys = KubeVirtProvider::public_keys(parsed).unwrap();
        let expect = PublicKey::parse("ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAAAgQDYVEprvtYJXVOBN0XNKVVRNCRX6BlnNbI+USLGais1sUWPwtSg7z9K9vhbYAPUZcq8c/s5S9dg5vTHbsiyPCIDOKyeHba4MUJq8Oh5b2i71/3BISpyxTBH/uZDHdslW2a+SrPDCeuMMoss9NFhBdKtDkdG9zyi0ibmCP6yMdEX8Q== Generated by Nova").unwrap();

        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0], expect);
    }

    #[test]
    fn test_kubevirt_parse_network_data_json() {
        let fixture = File::open("./tests/fixtures/kubevirt/network_data.json").unwrap();
        let bufrd = BufReader::new(fixture);
        let parsed = KubeVirtProvider::parse_network_data(bufrd).unwrap();

        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.config.len(), 3);

        let interfaces = KubeVirtProvider::network_interfaces(parsed).unwrap();
        assert_eq!(interfaces.len(), 2);
    }

    #[test]
    fn test_kubevirt_network_static() {
        let network_data = r#"
{
  "version": 1,
  "config": [
    {
      "type": "physical",
      "name": "eth0",
      "mac_address": "06:52:db:01:ff:d9",
      "subnets": [
        {
          "type": "static",
          "address": "192.168.1.10",
          "netmask": "255.255.255.0",
          "gateway": "192.168.1.1"
        }
      ]
    },
    {
      "type": "nameserver",
      "address": [
        "8.8.8.8",
        "8.8.4.4"
      ]
    }
  ]
}
"#;

        let bufrd = BufReader::new(Cursor::new(network_data));
        let parsed = KubeVirtProvider::parse_network_data(bufrd).unwrap();
        let interfaces = KubeVirtProvider::network_interfaces(parsed).unwrap();

        assert_eq!(interfaces.len(), 1);
        assert_eq!(interfaces[0].name, Some("eth0".to_string()));
        assert_eq!(interfaces[0].mac_address, Some(MacAddr::from_str("06:52:db:01:ff:d9").unwrap()));
        assert_eq!(interfaces[0].ip_addresses.len(), 1);
        assert_eq!(interfaces[0].ip_addresses[0], IpNetwork::from_str("192.168.1.10/24").unwrap());
        assert_eq!(interfaces[0].routes.len(), 1);
        assert_eq!(interfaces[0].routes[0].gateway, IpAddr::from_str("192.168.1.1").unwrap());
        assert_eq!(interfaces[0].nameservers.len(), 2);
        assert_eq!(interfaces[0].nameservers[0], IpAddr::from_str("8.8.8.8").unwrap());
        assert_eq!(interfaces[0].nameservers[1], IpAddr::from_str("8.8.4.4").unwrap());
    }

    #[test]
    fn test_kubevirt_network_dhcp() {
        let network_data = r#"
{
  "version": 1,
  "config": [
    {
      "type": "physical",
      "name": "eth0",
      "mac_address": "06:52:db:01:ff:d9",
      "subnets": [
        {
          "type": "dhcp"
        }
      ]
    },
    {
      "type": "nameserver",
      "address": [
        "8.8.8.8"
      ]
    }
  ]
}
"#;

        let bufrd = BufReader::new(Cursor::new(network_data));
        let parsed = KubeVirtProvider::parse_network_data(bufrd).unwrap();
        let interfaces = KubeVirtProvider::network_interfaces(parsed).unwrap();

        assert_eq!(interfaces.len(), 1);
        assert_eq!(interfaces[0].name, Some("eth0".to_string()));
        assert_eq!(interfaces[0].dhcp, Some(DhcpSetting::V4));
        assert_eq!(interfaces[0].ip_addresses.len(), 0);
        assert_eq!(interfaces[0].nameservers.len(), 1);
        assert_eq!(interfaces[0].nameservers[0], IpAddr::from_str("8.8.8.8").unwrap());
    }

    #[test]
    fn test_kubevirt_rd_network_kargs_static() {
        let network_data = r#"
{
  "version": 1,
  "config": [
    {
      "type": "physical",
      "name": "eth0",
      "mac_address": "06:52:db:01:ff:d9",
      "subnets": [
        {
          "type": "static",
          "address": "192.168.1.10",
          "netmask": "255.255.255.0",
          "gateway": "192.168.1.1"
        }
      ]
    },
    {
      "type": "nameserver",
      "address": [
        "8.8.8.8",
        "8.8.4.4"
      ]
    }
  ]
}
"#;

        let bufrd = BufReader::new(Cursor::new(network_data));
        let parsed = KubeVirtProvider::parse_network_data(bufrd).unwrap();
        let interfaces = KubeVirtProvider::network_interfaces(parsed).unwrap();

        // Simulate what rd_network_kargs would do
        let mut kargs = Vec::new();
        for iface in interfaces {
            for addr in iface.ip_addresses {
                if let IpNetwork::V4(network) = addr {
                    if let Some(gateway) = iface
                        .routes
                        .iter()
                        .find(|r| r.destination.is_ipv4() && r.destination.prefix() == 0)
                    {
                        kargs.push(format!(
                            "ip={}::{}:{}",
                            network.ip(),
                            gateway.gateway,
                            network.mask()
                        ));
                    }
                }
            }
            if !iface.nameservers.is_empty() {
                let nameservers = iface
                    .nameservers
                    .iter()
                    .map(|ns| ns.to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                kargs.push(format!("nameserver={}", nameservers));
            }
        }

        let result = kargs.join(" ");
        assert!(result.contains("ip=192.168.1.10::192.168.1.1:255.255.255.0"));
        assert!(result.contains("nameserver=8.8.8.8,8.8.4.4"));
    }

    #[test]
    fn test_kubevirt_rd_network_kargs_dhcp() {
        let network_data = r#"
{
  "version": 1,
  "config": [
    {
      "type": "physical",
      "name": "eth0",
      "mac_address": "06:52:db:01:ff:d9",
      "subnets": [
        {
          "type": "dhcp"
        }
      ]
    },
    {
      "type": "nameserver",
      "address": [
        "8.8.8.8"
      ]
    }
  ]
}
"#;

        let bufrd = BufReader::new(Cursor::new(network_data));
        let parsed = KubeVirtProvider::parse_network_data(bufrd).unwrap();
        let interfaces = KubeVirtProvider::network_interfaces(parsed).unwrap();

        // Simulate what rd_network_kargs would do
        let mut kargs = Vec::new();
        for iface in interfaces {
            if let Some(dhcp) = iface.dhcp {
                match dhcp {
                    DhcpSetting::V4 => kargs.push("ip=dhcp".to_string()),
                    DhcpSetting::V6 => kargs.push("ip=dhcp6".to_string()),
                    DhcpSetting::Both => kargs.push("ip=dhcp,dhcp6".to_string()),
                }
            }
            if !iface.nameservers.is_empty() {
                let nameservers = iface
                    .nameservers
                    .iter()
                    .map(|ns| ns.to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                kargs.push(format!("nameserver={}", nameservers));
            }
        }

        let result = kargs.join(" ");
        assert!(result.contains("ip=dhcp"));
        assert!(result.contains("nameserver=8.8.8.8"));
    }
}
