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

//! Azure provider, metadata and wireserver fetcher.

use super::goalstate;

use std::collections::HashMap;
use std::net::IpAddr;

use anyhow::{anyhow, Context, Result};
use openssh_keys::PublicKey;
use reqwest::header::{HeaderName, HeaderValue};
use serde::Deserialize;
use slog_scope::warn;

use crate::providers::MetadataProvider;
use crate::retry;
use nix::unistd::Uid;

#[cfg(test)]
mod mock_tests;

static HDR_AGENT_NAME: &str = "x-ms-agent-name";
static HDR_VERSION: &str = "x-ms-version";

const MS_AGENT_NAME: &str = "com.coreos.afterburn";
const MS_VERSION: &str = "2012-11-30";

/// This is a known working wireserver endpoint within Azure.
/// See: https://blogs.msdn.microsoft.com/mast/2015/05/18/what-is-the-ip-address-168-63-129-16/
#[cfg(not(test))]
const FALLBACK_WIRESERVER_ADDR: [u8; 4] = [168, 63, 129, 16]; // for grep: 168.63.129.16

macro_rules! ready_state {
    ($container:expr, $instance:expr, $incarnation:expr) => {
        format!(r#"<?xml version="1.0" encoding="utf-8"?>
<Health xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema">
  <GoalStateIncarnation>{}</GoalStateIncarnation>
  <Container>
    <ContainerId>{}</ContainerId>
    <RoleInstanceList>
      <Role>
        <InstanceId>{}</InstanceId>
        <Health>
          <State>Ready</State>
        </Health>
      </Role>
    </RoleInstanceList>
  </Container>
</Health>
"#,
                $incarnation, $container, $instance)
    }
}

#[derive(Debug, Deserialize, Clone)]
struct Versions {
    #[serde(rename = "Supported")]
    pub supported: Supported,
}

#[derive(Debug, Deserialize, Clone)]
struct Supported {
    #[serde(rename = "Version", default)]
    pub versions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Azure {
    client: retry::Client,
    endpoint: IpAddr,
}

#[derive(Debug, Default)]
struct Attributes {
    pub virtual_ipv4: Option<IpAddr>,
    pub dynamic_ipv4: Option<IpAddr>,
}

impl Azure {
    /// Try to build a new provider agent for Azure.
    ///
    /// This internally tries to reach the WireServer and verify compatibility.
    pub fn try_new() -> Result<Self> {
        Self::with_client(None)
    }

    /// Try to build a new provider agent for Azure, with a given client.
    pub(crate) fn with_client(client: Option<retry::Client>) -> Result<Azure> {
        let wireserver_ip = Azure::get_fabric_address();
        Self::verify_platform(client, wireserver_ip)
    }

    /// Try to reach cloud endpoint to ensure we are on a compatible Azure platform.
    pub(crate) fn verify_platform(
        client: Option<retry::Client>,
        endpoint: IpAddr,
    ) -> Result<Azure> {
        let mut client = match client {
            Some(c) => c,
            None => retry::Client::try_new()?,
        };

        // Add headers required by API.
        client = client
            .header(
                HeaderName::from_static(HDR_AGENT_NAME),
                HeaderValue::from_static(MS_AGENT_NAME),
            )
            .header(
                HeaderName::from_static(HDR_VERSION),
                HeaderValue::from_static(MS_VERSION),
            );

        let azure = Azure { client, endpoint };

        // Make sure WireServer API version is compatible with our logic.
        azure
            .is_fabric_compatible(MS_VERSION)
            .inspect_err(|_e| {
                let is_root = Uid::current().is_root();
                if !is_root {
                    // Firewall rules may be blocking requests from non-root
                    // processes, see https://github.com/coreos/bugs/issues/2468.
                    warn!("unable to reach Azure endpoints, please check whether firewall rules are blocking access to them");
                }
            })
            .context("failed version compatibility check")?;

        Ok(azure)
    }

    /// Retrieve `goalstate` content from the WireServer.
    fn fetch_goalstate(&self) -> Result<goalstate::GoalState> {
        self.client
            .get(
                retry::Xml,
                format!("{}/machine/?comp=goalstate", self.fabric_base_url()),
            )
            .send()
            .context("failed to get goal state")?
            .ok_or_else(|| anyhow!("failed to get goal state: not found response"))
    }

    #[cfg(not(test))]
    fn get_fabric_address() -> IpAddr {
        // try to fetch from dhcp, else use fallback; this is similar to what WALinuxAgent does
        Azure::get_fabric_address_from_dhcp().unwrap_or_else(|e| {
            warn!("Failed to get fabric address from DHCP: {}", e);
            slog_scope::info!("using fallback address");
            IpAddr::from(FALLBACK_WIRESERVER_ADDR)
        })
    }

    #[cfg(not(test))]
    fn get_fabric_address_from_dhcp() -> Result<IpAddr> {
        let v = crate::util::DhcpOption::AzureFabricAddress.get_value()?;
        // value is an 8 digit hex value, with colons if it came from
        // NetworkManager.  Convert it to u32 and then parse that into an
        // IP.  Ipv4Addr::from(u32) performs conversion from big-endian.
        slog_scope::trace!("found fabric address in hex - {:?}", v);
        let dec = u32::from_str_radix(&v.replace(':', ""), 16)
            .with_context(|| format!("failed to convert '{v}' from hex"))?;
        Ok(IpAddr::V4(dec.into()))
    }

    fn fabric_base_url(&self) -> String {
        format!("http://{}", self.endpoint)
    }

    #[cfg(test)]
    fn get_fabric_address() -> IpAddr {
        use std::net::Ipv4Addr;
        IpAddr::from(Ipv4Addr::new(127, 0, 0, 1))
    }

    fn is_fabric_compatible(&self, version: &str) -> Result<()> {
        let versions: Versions = self
            .client
            .get(
                retry::Xml,
                format!("{}/?comp=versions", self.fabric_base_url()),
            )
            .send()
            .context("failed to get versions")?
            .ok_or_else(|| anyhow!("failed to get versions: not found"))?;

        if versions.supported.versions.iter().any(|v| v == version) {
            Ok(())
        } else {
            Err(anyhow!(
                "fabric version '{}' not supported by the WireServer at '{}'",
                version,
                self.endpoint
            ))
        }
    }

    fn metadata_endpoint() -> String {
        "http://169.254.169.254".into()
    }

    fn get_attributes(&self) -> Result<Attributes> {
        use std::net::SocketAddr;

        let goalstate = self.fetch_goalstate()?;
        let endpoint = &goalstate.container.role_instance_list.role_instances[0]
            .configuration
            .shared_config;

        let shared_config: goalstate::SharedConfig = self
            .client
            .get(retry::Xml, endpoint.to_string())
            .send()
            .context("failed to get shared configuration")?
            .ok_or_else(|| anyhow!("failed to get shared configuration: not found"))?;

        let mut attributes = Attributes::default();

        for instance in shared_config.instances.instances {
            if instance.id == shared_config.incarnation.instance {
                attributes.dynamic_ipv4 = Some(instance.address.parse().with_context(|| {
                    format!("failed to parse instance ip address: {}", instance.address)
                })?);
                for endpoint in instance.input_endpoints.endpoints {
                    attributes.virtual_ipv4 =
                        match endpoint.load_balanced_public_address.parse::<SocketAddr>() {
                            Ok(lbpa) => Some(lbpa.ip()),
                            Err(_) => continue,
                        };
                }
            }
        }

        Ok(attributes)
    }

    fn fetch_hostname(&self) -> Result<Option<String>> {
        const NAME_URL: &str = "metadata/instance/compute/name?api-version=2017-08-01&format=text";
        let url = format!("{}/{}", Self::metadata_endpoint(), NAME_URL);

        let name = self
            .client
            .clone()
            .header(
                HeaderName::from_static("metadata"),
                HeaderValue::from_static("true"),
            )
            .get(retry::Raw, url)
            .send()
            .context("failed to get hostname")?;
        Ok(name)
    }

    fn fetch_vmsize(&self) -> Result<String> {
        const VMSIZE_URL: &str =
            "metadata/instance/compute/vmSize?api-version=2017-08-01&format=text";
        let url = format!("{}/{}", Self::metadata_endpoint(), VMSIZE_URL);

        let vmsize = self
            .client
            .clone()
            .header(
                HeaderName::from_static("metadata"),
                HeaderValue::from_static("true"),
            )
            .get(retry::Raw, url)
            .send()?
            .context("failed to get vmsize")?;
        Ok(vmsize)
    }

    /// Fetch SSH public keys from Azure Instance Metadata Service (IMDS)
    /// https://learn.microsoft.com/en-us/azure/virtual-machines/instance-metadata-service
    fn fetch_ssh_keys(&self) -> Result<Vec<PublicKey>> {
        const URL: &str = "metadata/instance/compute/publicKeys?api-version=2021-02-01";
        let url = format!("{}/{}", Self::metadata_endpoint(), URL);

        let body = self
            .client
            .clone()
            .header(
                HeaderName::from_static("metadata"),
                HeaderValue::from_static("true"),
            )
            .get(retry::Raw, url)
            .send::<String>()
            .context("failed to query IMDS for publicKeys")?
            .ok_or_else(|| anyhow::anyhow!("IMDS did not return a publicKeys payload"))?;

        #[derive(Debug, Deserialize)]
        struct ImdsSshKey {
            #[serde(rename = "keyData")]
            key_data: String,
            path: String,
        }

        let items: Vec<ImdsSshKey> =
            serde_json::from_str(&body).context("failed to parse IMDS publicKeys JSON")?;

        let keys: Vec<PublicKey> = items
            .into_iter()
            .map(|item| {
                let kd = item
                    .key_data
                    .replace("\r\n", "")
                    .replace('\n', "")
                    .trim()
                    .to_string();

                PublicKey::parse(&kd)
                    .with_context(|| format!("failed to parse IMDS key at path {}", item.path))
            })
            .collect::<Result<_>>()?;

        Ok(keys)
    }

    /// Report ready state to the WireServer.
    ///
    /// This is used to signal to the cloud platform that the VM has
    /// booted into userland. The definition of "ready" is fuzzy.
    fn report_ready_state(&self) -> Result<()> {
        let goalstate = self.fetch_goalstate()?;
        let body = ready_state!(
            goalstate.container_id(),
            goalstate.instance_id()?,
            goalstate.incarnation()
        );
        let url = self.fabric_base_url() + "/machine/?comp=health";
        self.client
            .post(retry::Xml, url, Some(body.into()))
            .dispatch_post()?;
        Ok(())
    }
}

impl MetadataProvider for Azure {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        let attributes = self.get_attributes()?;
        let vmsize = self.fetch_vmsize()?;
        let mut out = HashMap::with_capacity(3);

        if let Some(virtual_ipv4) = attributes.virtual_ipv4 {
            out.insert("AZURE_IPV4_VIRTUAL".to_string(), virtual_ipv4.to_string());
        }

        if let Some(dynamic_ipv4) = attributes.dynamic_ipv4 {
            out.insert("AZURE_IPV4_DYNAMIC".to_string(), dynamic_ipv4.to_string());
        }

        out.insert("AZURE_VMSIZE".to_string(), vmsize);

        Ok(out)
    }

    fn hostname(&self) -> Result<Option<String>> {
        self.fetch_hostname()
    }

    fn ssh_keys(&self) -> Result<Vec<PublicKey>> {
        self.fetch_ssh_keys()
    }

    fn boot_checkin(&self) -> Result<()> {
        let controller = retry::Retry::new().max_retries(5);
        controller.retry(|n| {
            if n > 0 {
                warn!("Retrying ready state report: Attempt #{}", n);
            }
            self.report_ready_state()
        })
    }
}
