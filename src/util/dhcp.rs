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

//! DHCP lease option lookup

use anyhow::{anyhow, Context, Result};
use slog_scope::{debug, trace};
use std::fs::File;
use std::path::Path;
use std::time::Duration;

use super::key_lookup;
use crate::retry;

pub enum DhcpOption {
    DhcpServerId,
    // avoid dead code warnings with cfg(test)
    #[allow(dead_code)]
    AzureFabricAddress,
}

impl DhcpOption {
    pub fn get_value(&self) -> Result<String> {
        retry::Retry::new()
            .initial_backoff(Duration::from_millis(50))
            .max_backoff(Duration::from_millis(500))
            .max_retries(60)
            .retry(|_| {
                match self.try_networkd() {
                    Ok(res) => return Ok(res),
                    Err(e) => trace!("failed querying networkd: {e:#}"),
                }
                Err(anyhow!("failed to acquire DHCP option"))
            })
    }

    fn try_networkd(&self) -> Result<String> {
        let key = match *self {
            Self::DhcpServerId => "SERVER_ADDRESS",
            Self::AzureFabricAddress => "OPTION_245",
        };

        let interfaces = pnet_datalink::interfaces();
        trace!("interfaces - {:?}", interfaces);

        for interface in interfaces {
            trace!("looking at interface {:?}", interface);
            let lease_path = format!("/run/systemd/netif/leases/{}", interface.index);
            let lease_path = Path::new(&lease_path);
            if lease_path.exists() {
                debug!("found lease file - {:?}", lease_path);
                let lease = File::open(lease_path)
                    .with_context(|| format!("failed to open lease file ({:?})", lease_path))?;

                if let Some(v) = key_lookup('=', key, lease)? {
                    return Ok(v);
                }

                debug!(
                    "failed to get value from existing lease file '{:?}'",
                    lease_path
                );
            }
        }
        Err(anyhow!("failed to acquire DHCP option {key}"))
    }
}
