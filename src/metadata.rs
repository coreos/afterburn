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

use errors;
use providers::*;

macro_rules! box_result {
    ($exp:expr) => {
        Ok(Box::new($exp?))
    };
}

/// `fetch_metadata` is the generic, top-level function that is used by the main
/// function to fetch metadata. The configured provider is passed in and this
/// function dispatches the call to the correct provider-specific fetch function
pub fn fetch_metadata(provider: &str) -> errors::Result<Box<MetadataProvider>> {
    match provider {
        "azure" => box_result!(azure::Azure::new()),
        "cloudstack-metadata" => box_result!(cloudstack::network::CloudstackNetwork::new()),
        "cloudstack-configdrive" => box_result!(cloudstack::configdrive::ConfigDrive::new()),
        "digitalocean" => box_result!(digitalocean::DigitalOceanProvider::new()),
        "ec2" => box_result!(ec2::Ec2Provider::new()),
        "gce" => box_result!(gce::GceProvider::new()),
        "hcloud" => box_result!(hcloud::HetznerCloudProvider::new()),
        "openstack-metadata" => box_result!(openstack::network::OpenstackProvider::new()),
        "packet" => box_result!(packet::PacketProvider::new()),
        "vagrant-virtualbox" => box_result!(vagrant_virtualbox::VagrantVirtualboxProvider::new()),
        _ => Err(errors::ErrorKind::UnknownProvider(provider.to_owned()).into()),
    }
}
