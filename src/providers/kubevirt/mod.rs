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

use crate::providers;
use crate::providers::noop::NoopProvider;
use anyhow::Result;
use slog_scope::warn;

mod configdrive;
pub use configdrive::*;

mod cloudconfig;
pub use cloudconfig::*;

mod networkdata;

#[cfg(test)]
mod tests;

pub fn try_config_drive_else_leave() -> Result<Box<dyn providers::MetadataProvider>> {
    match KubeVirtConfigDrive::try_new() {
        Ok(Some(config_drive)) => Ok(Box::new(config_drive)),
        Ok(None) => {
            warn!("config-2 drive not found");
            warn!("aborting KubeVirt provider");
            Ok(Box::new(NoopProvider::try_new()?))
        }
        Err(e) => {
            warn!("failed to read config-drive: {}", e);
            warn!("aborting KubeVirt provider");
            Ok(Box::new(NoopProvider::try_new()?))
        }
    }
}
