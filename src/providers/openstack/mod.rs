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

//! openstack metadata fetcher

use crate::errors;
use crate::providers;
use configdrive::OpenstackConfigDrive;
use network::OpenstackProviderNetwork;
use slog_scope::warn;

pub mod configdrive;
pub mod network;

#[cfg(test)]
mod mock_tests;

/// Read metadata from the config-drive first then fallback to fetch from metadata server.
///
/// Reference: https://github.com/coreos/fedora-coreos-tracker/issues/422
pub fn try_config_drive_else_network() -> errors::Result<Box<dyn providers::MetadataProvider>> {
    if let Ok(config_drive) = OpenstackConfigDrive::try_new() {
        Ok(Box::new(config_drive))
    } else {
        warn!("failed to locate config-drive, using the metadata service API instead");
        Ok(Box::new(OpenstackProviderNetwork::try_new()?))
    }
}
