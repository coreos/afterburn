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
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::time::Duration;
use zbus::{proxy, zvariant};

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
                match self.try_nm() {
                    Ok(res) => return Ok(res),
                    Err(e) => trace!("failed querying NetworkManager: {e:#}"),
                }
                match self.try_networkd() {
                    Ok(res) => return Ok(res),
                    Err(e) => trace!("failed querying networkd: {e:#}"),
                }
                Err(anyhow!("failed to acquire DHCP option"))
            })
    }

    fn try_nm(&self) -> Result<String> {
        let key = match *self {
            Self::DhcpServerId => "dhcp_server_identifier",
            Self::AzureFabricAddress => "private_245",
        };

        // We set up everything from scratch on every attempt.  This isn't
        // super-efficient but is simple and clear.
        //
        // We'd like to set both `property` and `object` attributes on the
        // trait methods, but that fails to compile, so we create proxies by
        // hand.

        // query NM for active connections
        let bus = zbus::blocking::Connection::system().context("connecting to D-Bus")?;
        let nm = NetworkManagerProxyBlocking::new(&bus).context("creating NetworkManager proxy")?;
        let conn_paths = nm
            .active_connections()
            .context("listing active connections")?;

        // walk active connections
        for conn_path in conn_paths {
            if conn_path == "/" {
                continue;
            }
            trace!("found NetworkManager connection: {conn_path}");
            let conn = NMActiveConnectionProxyBlocking::builder(&bus)
                .path(conn_path)
                .context("setting connection path")?
                .build()
                .context("creating connection proxy")?;

            // get DHCP options
            let dhcp_path = conn.dhcp4_config().context("getting DHCP config")?;
            if dhcp_path == "/" {
                continue;
            }
            debug!("checking DHCP config: {dhcp_path}");
            let dhcp = NMDhcp4ConfigProxyBlocking::builder(&bus)
                .path(dhcp_path)
                .context("setting DHCP config path")?
                .build()
                .context("creating DHCP config proxy")?;
            let options = dhcp.options().context("getting DHCP options")?;

            // check for option
            if let Some(value) = options.get(key) {
                return value.try_into().context("reading DHCP option as string");
            }
        }

        // not found
        Err(anyhow!("failed to acquire DHCP option {key}"))
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
                    .with_context(|| format!("failed to open lease file ({lease_path:?})"))?;

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

#[proxy(
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager",
    interface = "org.freedesktop.NetworkManager"
)]
trait NetworkManager {
    #[zbus(property)]
    fn active_connections(&self) -> zbus::Result<Vec<zvariant::ObjectPath<'_>>>;
}

#[proxy(
    default_service = "org.freedesktop.NetworkManager",
    interface = "org.freedesktop.NetworkManager.Connection.Active"
)]
trait NMActiveConnection {
    #[zbus(property)]
    fn dhcp4_config(&self) -> zbus::Result<zvariant::ObjectPath<'_>>;
}

#[proxy(
    default_service = "org.freedesktop.NetworkManager",
    interface = "org.freedesktop.NetworkManager.DHCP4Config"
)]
trait NMDhcp4Config {
    #[zbus(property)]
    fn options(&self) -> Result<HashMap<String, zvariant::Value<'_>>>;
}
