// Copyright 2021 Red Hat, Inc.
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

//! AzureStack provider, metadata and wireserver fetcher.

use super::crypto;
use super::goalstate;

use std::net::IpAddr;

use anyhow::{anyhow, bail, Context, Result};
use openssh_keys::PublicKey;
use reqwest::header::{HeaderName, HeaderValue};
use serde::Deserialize;
use slog_scope::warn;

use self::crypto::x509;
use crate::providers::MetadataProvider;
use crate::retry;
use nix::unistd::Uid;

#[cfg(test)]
mod mock_tests;

static HDR_AGENT_NAME: &str = "x-ms-agent-name";
static HDR_VERSION: &str = "x-ms-version";
static HDR_CIPHER_NAME: &str = "x-ms-cipher-name";
static HDR_CERT: &str = "x-ms-guest-agent-public-x509-cert";

const MS_AGENT_NAME: &str = "com.coreos.afterburn";
const MS_VERSION: &str = "2012-11-30";
const SMIME_HEADER: &str = "\
MIME-Version:1.0
Content-Disposition: attachment; filename=/home/core/encrypted-ssh-cert.pem
Content-Type: application/x-pkcs7-mime; name=/home/core/encrypted-ssh-cert.pem
Content-Transfer-Encoding: base64

";

/// This is a known working wireserver endpoint within AzureStack.
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
pub struct AzureStack {
    client: retry::Client,
    endpoint: IpAddr,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct InstanceMetadata {
    pub vm_name: String,
    pub subscription_id: String,
}

impl AzureStack {
    /// Try to build a new provider agent for AzureStack.
    ///
    /// This internally tries to reach the WireServer and verify compatibility.
    pub fn try_new() -> Result<Self> {
        Self::with_client(None)
    }

    /// Try to build a new provider agent for AzureStack, with a given client.
    pub(crate) fn with_client(client: Option<retry::Client>) -> Result<AzureStack> {
        let wireserver_ip = AzureStack::get_fabric_address();
        Self::verify_platform(client, wireserver_ip)
    }

    /// Try to reach cloud endpoint to ensure we are on a compatible AzureStack platform.
    pub(crate) fn verify_platform(
        client: Option<retry::Client>,
        endpoint: IpAddr,
    ) -> Result<AzureStack> {
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

        let azure_stack = AzureStack { client, endpoint };

        // Make sure WireServer API version is compatible with our logic.
        azure_stack
            .is_fabric_compatible(MS_VERSION)
            .inspect_err(|_e| {
                let is_root = Uid::current().is_root();
                if !is_root {
                    // Firewall rules may be blocking requests from non-root
                    // processes, see https://github.com/coreos/bugs/issues/2468.
                    warn!("unable to reach AzureStack endpoints, please check whether firewall rules are blocking access to them");
                }
            })
            .context("failed version compatibility check")?;

        Ok(azure_stack)
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

    fn fetch_identity(&self) -> Result<InstanceMetadata> {
        const NAME_URL: &str = "Microsoft.Compute/identity?api-version=2019-03-11";
        let url = format!("{}/{}", Self::metadata_endpoint(), NAME_URL);
        self.client
            .clone()
            .header(
                HeaderName::from_static("metadata"),
                HeaderValue::from_static("true"),
            )
            .get(retry::Json, url)
            .send()
            .context("failed to get metadata JSON")?
            .ok_or_else(|| anyhow!("failed to get identity: not found response"))
    }

    #[cfg(not(test))]
    fn get_fabric_address() -> IpAddr {
        // try to fetch from dhcp, else use fallback; this is similar to what WALinuxAgent does
        AzureStack::get_fabric_address_from_dhcp().unwrap_or_else(|e| {
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

    // Fetch the certificate.
    fn fetch_cert(&self, certs_endpoint: String, mangled_pem: impl AsRef<str>) -> Result<String> {
        let certs: goalstate::CertificatesFile = self
            .client
            .get(retry::Xml, certs_endpoint)
            .header(
                HeaderName::from_static(HDR_CIPHER_NAME),
                HeaderValue::from_static("DES_EDE3_CBC"),
            )
            .header(
                HeaderName::from_static(HDR_CERT),
                HeaderValue::from_str(mangled_pem.as_ref())?,
            )
            .send()
            .context("failed to get certificates")?
            .ok_or_else(|| anyhow!("failed to get certificates: not found"))?;

        // the cms decryption expects it to have MIME information on the top
        // since cms is really for email attachments....
        let mut smime = String::from(SMIME_HEADER);
        smime.push_str(&certs.data);

        Ok(smime)
    }

    // put it all together
    fn get_ssh_pubkey(&self, certs_endpoint: String) -> Result<Option<PublicKey>> {
        // we have to generate the rsa public/private keypair and the x509 cert
        // that we use to make the request. this is equivalent to
        // `openssl req -x509 -nodes -subj /CN=LinuxTransport -days 365 -newkey rsa:2048 -keyout private.pem -out cert.pem`
        let (x509, pkey) = x509::generate_cert(&x509::Config::new(2048, 365))
            .context("failed to generate keys")?;

        // mangle the pem file for the request
        let mangled_pem = crypto::mangle_pem(&x509).context("failed to mangle pem")?;

        // fetch the encrypted cms blob from the certs endpoint
        let smime = self
            .fetch_cert(certs_endpoint, mangled_pem)
            .context("failed to fetch certificate")?;

        // decrypt the cms blob
        let p12 = crypto::decrypt_cms(smime.as_bytes(), &pkey, &x509)
            .context("failed to decrypt cms blob")?;

        // convert that to the OpenSSH public key format
        crypto::p12_to_ssh_pubkey(&p12).context("failed to convert pkcs12 blob to ssh pubkey")
    }

    fn fetch_hostname(&self) -> Result<Option<String>> {
        let instance_metadata = self.fetch_identity()?;
        Ok(Some(instance_metadata.vm_name))
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

impl MetadataProvider for AzureStack {
    fn hostname(&self) -> Result<Option<String>> {
        self.fetch_hostname()
    }

    fn ssh_keys(&self) -> Result<Vec<PublicKey>> {
        let goalstate = self.fetch_goalstate()?;
        let certs_endpoint = match goalstate.certs_endpoint() {
            Some(ep) => ep,
            None => return Ok(vec![]),
        };

        if certs_endpoint.is_empty() {
            bail!("unexpected empty certificates endpoint");
        }

        let maybe_key = self.get_ssh_pubkey(certs_endpoint)?;
        let key: Vec<PublicKey> = maybe_key.into_iter().collect();

        Ok(key)
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
