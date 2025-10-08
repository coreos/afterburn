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

mod provider;
pub use provider::*;

mod cloudconfig;
pub use cloudconfig::*;

mod configdrive;

mod nocloud;

#[cfg(test)]
mod tests;

pub fn try_new_provider_else_noop() -> Result<Box<dyn providers::MetadataProvider>> {
    match KubeVirtProvider::try_new() {
        Ok(Some(provider)) => Ok(Box::new(provider)),
        Ok(None) => {
            warn!("config device not found");
            warn!("aborting KubeVirt provider");
            Ok(Box::new(NoopProvider::try_new()?))
        }
        Err(e) => {
            warn!("failed to read config device: {}", e);
            warn!("aborting KubeVirt provider");
            Ok(Box::new(NoopProvider::try_new()?))
        }
    }
}
