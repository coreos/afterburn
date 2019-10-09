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

mod crypto;

use std::collections::HashMap;
use std::net::IpAddr;

use openssh_keys::PublicKey;
use reqwest::header::{HeaderName, HeaderValue};

use self::crypto::x509;
use crate::errors::*;
use crate::network;
use crate::providers::MetadataProvider;
use crate::retry;

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

/// This is a known working wireserver endpoint within Azure.
/// See: https://blogs.msdn.microsoft.com/mast/2015/05/18/what-is-the-ip-address-168-63-129-16/
#[cfg(not(test))]
const FALLBACK_WIRESERVER_ADDR: [u8; 4] = [168, 63, 129, 16]; // for grep: 168.63.129.16

macro_rules! ready_state {
    ($container:expr, $instance:expr) => {
        format!(r#"<?xml version="1.0" encoding="utf-8"?>
<Health xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema">
  <GoalStateIncarnation>1</GoalStateIncarnation>
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
                $container, $instance)
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
struct GoalState {
    #[serde(rename = "Container")]
    pub container: Container,
}

#[derive(Debug, Deserialize, Clone, Default)]
struct Container {
    #[serde(rename = "ContainerId")]
    pub container_id: String,
    #[serde(rename = "RoleInstanceList")]
    pub role_instance_list: RoleInstanceList,
}

#[derive(Debug, Deserialize, Clone, Default)]
struct RoleInstanceList {
    #[serde(rename = "RoleInstance", default)]
    pub role_instances: Vec<RoleInstance>,
}

#[derive(Debug, Deserialize, Clone)]
struct RoleInstance {
    #[serde(rename = "Configuration")]
    pub configuration: Configuration,
    #[serde(rename = "InstanceId")]
    pub instance_id: String,
}

#[derive(Debug, Deserialize, Clone)]
struct Configuration {
    #[serde(rename = "Certificates", default)]
    pub certificates: String,
    #[serde(rename = "SharedConfig", default)]
    pub shared_config: String,
}

#[derive(Debug, Deserialize, Clone)]
struct CertificatesFile {
    #[serde(rename = "Data", default)]
    pub data: String,
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

#[derive(Debug, Deserialize, Clone)]
struct SharedConfig {
    #[serde(rename = "Incarnation")]
    pub incarnation: Incarnation,
    #[serde(rename = "Instances")]
    pub instances: Instances,
}

#[derive(Debug, Deserialize, Clone)]
struct Incarnation {
    pub instance: String,
}

#[derive(Debug, Deserialize, Clone)]
struct Instances {
    #[serde(rename = "Instance", default)]
    pub instances: Vec<Instance>,
}

#[derive(Debug, Deserialize, Clone)]
struct Instance {
    pub id: String,
    pub address: String,
    #[serde(rename = "InputEndpoints")]
    pub input_endpoints: InputEndpoints,
}

#[derive(Debug, Deserialize, Clone)]
struct InputEndpoints {
    #[serde(rename = "Endpoint", default)]
    pub endpoints: Vec<Endpoint>,
}

#[derive(Debug, Deserialize, Clone)]
struct Endpoint {
    #[serde(rename = "loadBalancedPublicAddress", default)]
    pub load_balanced_public_address: String,
}

#[derive(Debug, Copy, Clone, Default)]
struct Attributes {
    pub virtual_ipv4: Option<IpAddr>,
    pub dynamic_ipv4: Option<IpAddr>,
}

#[derive(Debug, Clone)]
pub struct Azure {
    client: retry::Client,
    endpoint: IpAddr,
    goal_state: GoalState,
}

impl Azure {
    pub fn try_new() -> Result<Azure> {
        let addr = Azure::get_fabric_address();
        let client = retry::Client::try_new()?
            .header(
                HeaderName::from_static(HDR_AGENT_NAME),
                HeaderValue::from_static(MS_AGENT_NAME),
            )
            .header(
                HeaderName::from_static(HDR_VERSION),
                HeaderValue::from_static(MS_VERSION),
            );

        let mut azure = Azure {
            client,
            endpoint: addr,
            goal_state: GoalState::default(),
        };

        // make sure the metadata service is compatible with our version
        azure
            .is_fabric_compatible(MS_VERSION)
            .chain_err(|| "failed version compatibility check")?;

        // populate goalstate
        azure.goal_state = azure.get_goal_state()?;
        Ok(azure)
    }

    fn get_goal_state(&self) -> Result<GoalState> {
        self.client
            .get(
                retry::Xml,
                format!("{}/machine/?comp=goalstate", self.fabric_base_url()),
            )
            .send()
            .chain_err(|| "failed to get goal state")?
            .ok_or_else(|| "failed to get goal state: not found response".into())
    }

    #[cfg(not(test))]
    fn get_fabric_address() -> IpAddr {
        // try to fetch from dhcp, else use fallback; this is similar to what WALinuxAgent does
        Azure::get_fabric_address_from_dhcp().unwrap_or_else(|e| {
            warn!("Failed to get fabric address from DHCP: {}", e);
            info!("Using fallback address");
            IpAddr::from(FALLBACK_WIRESERVER_ADDR)
        })
    }

    #[cfg(not(test))]
    fn get_fabric_address_from_dhcp() -> Result<IpAddr> {
        let v = crate::util::dns_lease_key_lookup("OPTION_245")?;
        // value is an 8 digit hex value. convert it to u32 and
        // then parse that into an ip. Ipv4Addr::from(u32)
        // performs conversion from big-endian
        trace!("found fabric address in hex - {:?}", v);
        let dec = u32::from_str_radix(&v, 16)
            .chain_err(|| format!("failed to convert '{}' from hex", v))?;
        Ok(IpAddr::V4(dec.into()))
    }

    #[cfg(not(test))]
    fn fabric_base_url(&self) -> String {
        format!("http://{}", self.endpoint)
    }

    #[cfg(test)]
    fn get_fabric_address() -> IpAddr {
        use std::net::Ipv4Addr;
        IpAddr::from(Ipv4Addr::new(127, 0, 0, 1))
    }

    #[cfg(test)]
    fn fabric_base_url(&self) -> String {
        ::mockito::server_url().to_string()
    }

    fn is_fabric_compatible(&self, version: &str) -> Result<()> {
        let versions: Versions = self
            .client
            .get(
                retry::Xml,
                format!("{}/?comp=versions", self.fabric_base_url()),
            )
            .send()
            .chain_err(|| "failed to get versions")?
            .ok_or_else(|| "failed to get versions: not found")?;

        if versions.supported.versions.iter().any(|v| v == version) {
            Ok(())
        } else {
            Err(format!(
                "fabric version {} not compatible with fabric address {}",
                MS_VERSION, self.endpoint
            )
            .into())
        }
    }

    #[cfg(test)]
    fn metadata_endpoint() -> String {
        mockito::server_url()
    }

    #[cfg(not(test))]
    fn metadata_endpoint() -> String {
        const URL: &str = "http://169.254.169.254";
        URL.to_string()
    }

    fn get_certs_endpoint(&self) -> Result<String> {
        // grab the certificates endpoint from the xml and return it
        let cert_endpoint: &str = &self.goal_state.container.role_instance_list.role_instances[0]
            .configuration
            .certificates;
        Ok(String::from(cert_endpoint))
    }

    fn get_certs<S: AsRef<str>>(&self, mangled_pem: S) -> Result<String> {
        // get the certificates
        let endpoint = self
            .get_certs_endpoint()
            .chain_err(|| "failed to get certs endpoint")?;

        let certs: CertificatesFile = self
            .client
            .get(retry::Xml, endpoint)
            .header(
                HeaderName::from_static(HDR_CIPHER_NAME),
                HeaderValue::from_static("DES_EDE3_CBC"),
            )
            .header(
                HeaderName::from_static(HDR_CERT),
                HeaderValue::from_str(mangled_pem.as_ref())?,
            )
            .send()
            .chain_err(|| "failed to get certificates")?
            .ok_or_else(|| "failed to get certificates: not found")?;

        // the cms decryption expects it to have MIME information on the top
        // since cms is really for email attachments...don't tell the cops.
        let mut smime = String::from(SMIME_HEADER);
        smime.push_str(&certs.data);

        Ok(smime)
    }

    // put it all together
    fn get_ssh_pubkey(&self) -> Result<PublicKey> {
        // first we have to get the certificates endoint.
        // we have to generate the rsa public/private keypair and the x509 cert
        // that we use to make the request. this is equivalent to
        // `openssl req -x509 -nodes -subj /CN=LinuxTransport -days 365 -newkey rsa:2048 -keyout private.pem -out cert.pem`
        let (x509, pkey) = x509::generate_cert(&x509::Config::new(2048, 365))
            .chain_err(|| "failed to generate keys")?;

        // mangle the pem file for the request
        let mangled_pem = crypto::mangle_pem(&x509).chain_err(|| "failed to mangle pem")?;

        // fetch the encrypted cms blob from the certs endpoint
        let smime = self
            .get_certs(mangled_pem)
            .chain_err(|| "failed to get certs")?;

        // decrypt the cms blob
        let p12 = crypto::decrypt_cms(smime.as_bytes(), &pkey, &x509)
            .chain_err(|| "failed to decrypt cms blob")?;

        // convert that to the OpenSSH public key format
        let ssh_pubkey = crypto::p12_to_ssh_pubkey(&p12)
            .chain_err(|| "failed to convert pkcs12 blob to ssh pubkey")?;

        Ok(ssh_pubkey)
    }

    #[cfg(test)]
    fn get_attributes(&self) -> Result<Attributes> {
        Ok(Attributes{
            virtual_ipv4: Some(Azure::get_fabric_address()),
            dynamic_ipv4: Some(Azure::get_fabric_address()),
        })
    }

    #[cfg(not(test))]
    fn get_attributes(&self) -> Result<Attributes> {
        use std::net::SocketAddr;

        let endpoint = &self.goal_state.container.role_instance_list.role_instances[0]
            .configuration
            .shared_config;

        let shared_config: SharedConfig = self
            .client
            .get(retry::Xml, endpoint.to_string())
            .send()
            .chain_err(|| "failed to get shared configuration")?
            .ok_or_else(|| "failed to get shared configuration: not found")?;

        let mut attributes = Attributes::default();

        for instance in shared_config.instances.instances {
            if instance.id == shared_config.incarnation.instance {
                attributes.dynamic_ipv4 = Some(instance.address.parse().chain_err(|| {
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

    /// Return this instance `ContainerId`.
    pub(crate) fn container_id(&self) -> &str {
        &self.goal_state.container.container_id
    }

    /// Return this instance `InstanceId`.
    pub(crate) fn instance_id(&self) -> Result<&str> {
        Ok(&self
            .goal_state
            .container
            .role_instance_list
            .role_instances
            .get(0)
            .ok_or_else(|| "empty RoleInstanceList".to_string())?
            .instance_id)
    }

    fn fetch_hostname(&self) -> Result<Option<String>> {
        const NAME_URL: &str = "metadata/instance/compute/name?api-version=2017-08-01&format=text";
        let url = format!("{}/{}", Self::metadata_endpoint(), NAME_URL);

        let name = retry::Client::try_new()?
            .header(
                HeaderName::from_static("metadata"),
                HeaderValue::from_static("true"),
            )
            .get(retry::Raw, url)
            .send()
            .chain_err(|| "failed to get hostname")?;
        Ok(name)
    }

    fn fetch_vmsize (&self) -> Result<String> {
        const VMSIZE_URL: &str = "metadata/instance/compute/vmSize?api-version=2017-08-01&format=text";
        let url = format!("{}/{}", Self::metadata_endpoint(), VMSIZE_URL);

        let vmsize = retry::Client::try_new()?
            .header(
                HeaderName::from_static("metadata"),
                HeaderValue::from_static("true"),
            )
            .get(retry::Raw, url)
            .send()?
            .chain_err(|| "failed to get vmsize")?;
        Ok(vmsize)
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

        out.insert("AZURE_VMSIZE".to_string(), vmsize.to_string());

        Ok(out)
    }

    fn hostname(&self) -> Result<Option<String>> {
        self.fetch_hostname()
    }

    fn ssh_keys(&self) -> Result<Vec<PublicKey>> {
        let key = self.get_ssh_pubkey()?;
        Ok(vec![key])
    }

    fn networks(&self) -> Result<Vec<network::Interface>> {
        Ok(vec![])
    }

    fn network_devices(&self) -> Result<Vec<network::Device>> {
        Ok(vec![])
    }

    fn boot_checkin(&self) -> Result<()> {
        let body = ready_state!(self.container_id(), self.instance_id()?);
        let url = self.fabric_base_url() + "/machine/?comp=health";
        self.client
            .post(retry::Xml, url, Some(body.into()))
            .dispatch_post()?;
        Ok(())
    }
}
