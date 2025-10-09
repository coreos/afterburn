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

use anyhow::{bail, Result};

use crate::providers;
use crate::providers::akamai::AkamaiProvider;
use crate::providers::aliyun::AliyunProvider;
use crate::providers::aws::AwsProvider;
use crate::providers::cloudstack::configdrive::ConfigDrive;
use crate::providers::cloudstack::network::CloudstackNetwork;
use crate::providers::digitalocean::DigitalOceanProvider;
use crate::providers::exoscale::ExoscaleProvider;
use crate::providers::gcp::GcpProvider;
use crate::providers::hetzner::HetznerProvider;
use crate::providers::ibmcloud::IBMGen2Provider;
use crate::providers::ibmcloud_classic::IBMClassicProvider;
use crate::providers::kubevirt;
use crate::providers::microsoft::azure::Azure;
use crate::providers::microsoft::azurestack::AzureStack;
use crate::providers::openstack;
use crate::providers::openstack::network::OpenstackProviderNetwork;
use crate::providers::oraclecloud::OracleCloudProvider;
use crate::providers::packet::PacketProvider;
use crate::providers::powervs::PowerVSProvider;
use crate::providers::proxmoxve;
use crate::providers::scaleway::ScalewayProvider;
use crate::providers::upcloud::UpCloudProvider;
use crate::providers::vmware::VmwareProvider;
use crate::providers::vultr::VultrProvider;

macro_rules! box_result {
    ($exp:expr) => {
        Ok(Box::new($exp))
    };
}

/// Fetch metadata for the given provider.
///
/// This is the generic, top-level function to fetch provider metadata.
/// The configured provider is passed in and this function dispatches the call
/// to the provider-specific fetch logic.
pub fn fetch_metadata(provider: &str) -> Result<Box<dyn providers::MetadataProvider>> {
    match provider {
        "akamai" => box_result!(AkamaiProvider::try_new()?),
        "aliyun" => box_result!(AliyunProvider::try_new()?),
        "aws" => box_result!(AwsProvider::try_new()?),
        "azure" => box_result!(Azure::try_new()?),
        "azurestack" => box_result!(AzureStack::try_new()?),
        "cloudstack-metadata" => box_result!(CloudstackNetwork::try_new()?),
        "cloudstack-configdrive" => box_result!(ConfigDrive::try_new()?),
        "digitalocean" => box_result!(DigitalOceanProvider::try_new()?),
        "exoscale" => box_result!(ExoscaleProvider::try_new()?),
        "gcp" => box_result!(GcpProvider::try_new()?),
        "hetzner" => box_result!(HetznerProvider::try_new()?),
        // IBM Cloud - VPC Generation 2.
        "ibmcloud" => box_result!(IBMGen2Provider::try_new()?),
        // IBM Cloud - Classic infrastructure.
        "ibmcloud-classic" => box_result!(IBMClassicProvider::try_new()?),
        "kubevirt" => kubevirt::try_new_provider_else_noop(),
        "openstack" => openstack::try_config_drive_else_network(),
        "openstack-metadata" => box_result!(OpenstackProviderNetwork::try_new()?),
        "oraclecloud" => box_result!(OracleCloudProvider::try_new()?),
        "packet" => box_result!(PacketProvider::try_new()?),
        "powervs" => box_result!(PowerVSProvider::try_new()?),
        "proxmoxve" => proxmoxve::try_config_drive_else_leave(),
        "scaleway" => box_result!(ScalewayProvider::try_new()?),
        "upcloud" => box_result!(UpCloudProvider::try_new()?),
        "vmware" => box_result!(VmwareProvider::try_new()?),
        "vultr" => box_result!(VultrProvider::try_new()?),
        _ => bail!("unknown provider '{}'", provider),
    }
}
