use crate::{network, providers::MetadataProvider};
use anyhow::Result;
use openssh_keys::PublicKey;
use serde::Deserialize;
use std::{fs::File, path::Path, str::FromStr};

#[derive(Debug)]
pub struct ProxmoxCloudConfig {
    pub meta_data: ProxmoxCloudMetaData,
    pub user_data: ProxmoxCloudUserData,
    pub vendor_data: ProxmoxCloudVendorData,
    pub network_config: ProxmoxCloudNetworkConfig,
}

#[derive(Debug, Deserialize)]
pub struct ProxmoxCloudMetaData {
    #[serde(rename = "instance-id")]
    pub instance_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ProxmoxCloudUserData {
    pub hostname: String,
    pub manage_etc_hosts: bool,
    pub fqdn: String,
    pub chpasswd: ProxmoxCloudChpasswdConfig,
    pub users: Vec<String>,
    pub package_upgrade: bool,
    #[serde(default)]
    pub ssh_authorized_keys: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProxmoxCloudChpasswdConfig {
    pub expire: bool,
}

#[derive(Debug, Deserialize)]
pub struct ProxmoxCloudVendorData {}

#[derive(Debug, Deserialize)]
pub struct ProxmoxCloudNetworkConfig {
    pub version: u32,
    pub config: Vec<ProxmoxCloudNetworkConfigEntry>,
}

#[derive(Debug, Deserialize)]
pub struct ProxmoxCloudNetworkConfigEntry {
    #[serde(rename = "type")]
    pub network_type: String,
    pub name: Option<String>,
    pub mac_address: Option<String>,
    #[serde(default)]
    pub address: Vec<String>,
    #[serde(default)]
    pub search: Vec<String>,
    #[serde(default)]
    pub subnets: Vec<ProxmoxCloudNetworkConfigSubnet>,
}

#[derive(Debug, Deserialize)]
pub struct ProxmoxCloudNetworkConfigSubnet {
    #[serde(rename = "type")]
    pub subnet_type: String,
}

impl ProxmoxCloudConfig {
    pub fn try_new(path: &Path) -> Result<Self> {
        Ok(Self {
            meta_data: serde_yaml::from_reader(File::open(path.join("meta-data"))?)?,
            user_data: serde_yaml::from_reader(File::open(path.join("user-data"))?)?,
            vendor_data: serde_yaml::from_reader(File::open(path.join("vendor-data"))?)?,
            network_config: serde_yaml::from_reader(File::open(path.join("network-config"))?)?,
        })
    }
}

impl MetadataProvider for ProxmoxCloudConfig {
    fn hostname(&self) -> Result<Option<String>> {
        Ok(Some(self.user_data.hostname.clone()))
    }

    fn ssh_keys(&self) -> Result<Vec<PublicKey>> {
        Ok(self
            .user_data
            .ssh_authorized_keys
            .iter()
            .map(|key| PublicKey::from_str(key))
            .collect::<Result<Vec<_>, _>>()?)
    }

    fn networks(&self) -> Result<Vec<network::Interface>> {
        Ok(vec![])
    }
}
