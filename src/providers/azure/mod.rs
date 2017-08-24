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

//! azure metadata fetcher

mod crypto;

use self::crypto::x509;

use errors::*;

use metadata::Metadata;

use pnet;

use retry;

use std::net::{IpAddr, Ipv4Addr};
use std::fs::File;
use std::path::Path;
use std::io::{BufRead, BufReader};

header! {(MSAgentName, "x-ms-agent-name") => [String]}
header! {(MSVersion, "x-ms-version") => [String]}
header! {(MSCipherName, "x-ms-cipher-name") => [String]}
header! {(MSCert, "x-ms-guest-agent-public-x509-cert") => [String]}

const OPTION_245: &str = "OPTION_245";
const MS_AGENT_NAME: &str = "com.coreos.metadata";
const MS_VERSION: &str = "2012-11-30";
const SMIME_HEADER: &str = "\
MIME-Version:1.0
Content-Disposition: attachment; filename=/home/core/encrypted-ssh-cert.pem
Content-Type: application/x-pkcs7-mime; name=/home/core/encrypted-ssh-cert.pem
Content-Transfer-Encoding: base64

";

#[derive(Debug, Deserialize)]
struct GoalState {
    #[serde(rename = "Container")]
    pub container: Container
}

#[derive(Debug, Deserialize)]
struct Container {
    #[serde(rename = "RoleInstanceList")]
    pub role_instance_list: RoleInstanceList
}

#[derive(Debug, Deserialize)]
struct RoleInstanceList {
    #[serde(rename = "RoleInstance")]
    pub role_instances: Vec<RoleInstance>
}

#[derive(Debug, Deserialize)]
struct RoleInstance {
    #[serde(rename = "Configuration")]
    pub configuration: Configuration
}

#[derive(Debug, Deserialize)]
struct Configuration {
    #[serde(rename = "Certificates", default)]
    pub certificates: String
}

#[derive(Debug, Deserialize)]
struct CertificatesFile {
    #[serde(rename = "Data", default)]
    pub data: String
}

#[derive(Debug, Deserialize)]
struct Versions {
    #[serde(rename = "Supported")]
    pub supported: Supported
}

#[derive(Debug, Deserialize)]
struct Supported {
    #[serde(rename = "Version", default)]
    pub versions: Vec<String>
}

struct Azure {
    client: retry::Client,
    endpoint: IpAddr,
}

impl Azure {
    fn new() -> Result<Self> {
        let addr = Azure::get_fabric_address()
            .chain_err(|| format!("failed to get fabric address"))?;
        let client = retry::Client::new()?
            .header(MSAgentName(MS_AGENT_NAME.to_owned()))
            .header(MSVersion(MS_VERSION.to_owned()));
        let azure = Azure {
            client: client,
            endpoint: addr,
        };
        azure.is_fabric_compatible(MS_VERSION)
            .chain_err(|| format!("failed version compatibility check"))?;
        Ok(azure)
    }

    // I don't really understand why this is how we need to get this ip
    // address but this is how it works in the original implementation
    // and nobody complains about it, so w/e
    fn get_fabric_address() -> Result<IpAddr> {
        // get the interfaces on the machine
        let interfaces = pnet::datalink::interfaces();
        trace!("interfaces - {:?}", interfaces);

        for interface in interfaces {
            trace!("looking at interface {:?}", interface);
            let lease_path = format!("/run/systemd/netif/leases/{}", interface.index);
            let lease_path = Path::new(&lease_path);
            if lease_path.exists() {
                debug!("found lease file - {:?}", lease_path);
                let lease = File::open(&lease_path)
                    .chain_err(|| format!("failed to open lease file ({:?})", lease_path))?;
                let lease = BufReader::new(&lease);

                // find the OPTION_245 flag
                for line in lease.lines() {
                    let line = line
                        .chain_err(|| format!("failed to read from lease file ({:?})", lease_path))?;
                    let option: Vec<&str> = line.split('=').collect();
                    if option.len() > 1 && option[0] == OPTION_245 {
                        // value is an 8 digit hex value. convert it to u32 and
                        // then parse that into an ip. Ipv4Addr::from(u32)
                        // performs conversion from big-endian
                        trace!("found fabric address in hex - {:?}", option[1]);
                        let dec = u32::from_str_radix(option[1], 16)
                            .chain_err(|| format!("failed to convert '{}' from hex", option[1]))?;
                        return Ok(IpAddr::V4(Ipv4Addr::from(dec)));
                    }
                }

                debug!("failed to get fabric address from existing lease file '{:?}'", lease_path);
            }
        }

        Err(format!("failed to retrieve fabric address").into())
    }

    fn is_fabric_compatible(&self, version: &str) -> Result<()> {
        let versions: Versions = self.client.get(retry::Xml, format!("http://{}/?comp=versions", self.endpoint)).send()
            .chain_err(|| format!("failed to get versions"))?
            .ok_or("failed to get versions: not found".to_owned())?;

        if versions.supported.versions.iter().any(|v| v == version) {
            Ok(())
        } else {
            Err(format!("fabric version {} not compatible with fabric address {}", MS_VERSION, self.endpoint).into())
        }
    }

    fn get_certs_endpoint(&self) -> Result<String> {
        let goalstate: GoalState = self.client.get(retry::Xml, format!("http://{}/machine/?comp=goalstate", self.endpoint)).send()
            .chain_err(|| format!("failed to get goal state"))?
            .ok_or("failed to get goal state: not found response".to_owned())?;

        // grab the certificates endpoint from the xml and return it
        let cert_endpoint: &str = &goalstate.container.role_instance_list.role_instances[0].configuration.certificates;
        Ok(String::from(cert_endpoint))
    }

    fn get_certs(&self, endpoint: String, mangled_pem: String) -> Result<String> {
        // get the certificates
        let certs: CertificatesFile = self.client.get(retry::Xml, endpoint)
            .header(MSCipherName("DES_EDE3_CBC".to_owned()))
            .header(MSCert(mangled_pem))
            .send()
            .chain_err(|| format!("failed to get certificates"))?
            .ok_or("failed to get certificates: not found".to_owned())?;

        // the cms decryption expects it to have MIME information on the top
        // since cms is really for email attachments...don't tell the cops.
        let mut smime = String::from(SMIME_HEADER);
        smime.push_str(&certs.data);

        Ok(smime)
    }

    // put it all together
    fn get_ssh_pubkey(&self) -> Result<String> {
        // first we have to get the certificates endoint.
        let certs_endpoint = self.get_certs_endpoint()
            .chain_err(|| format!("failed to get certs endpoint"))?;

        // we have to generate the rsa public/private keypair and the x509 cert
        // that we use to make the request. this is equivalent to
        // `openssl req -x509 -nodes -subj /CN=LinuxTransport -days 365 -newkey rsa:2048 -keyout private.pem -out cert.pem`
        let (x509, pkey) = x509::generate_cert(x509::Config::new(2048, 365))
            .chain_err(|| format!("failed to generate keys"))?;

        // mangle the pem file for the request
        let mangled_pem = crypto::mangle_pem(&x509)
            .chain_err(|| format!("failed to mangle pem"))?;

        // fetch the encrypted cms blob from the certs endpoint
        let smime = self.get_certs(certs_endpoint, mangled_pem)
            .chain_err(|| format!("failed to get certs"))?;

        // decrypt the cms blob
        let p12 = crypto::decrypt_cms(smime.as_bytes(), &pkey, &x509)
            .chain_err(|| format!("failed to decrypt cms blob"))?;

        // convert that to the OpenSSH public key format
        let ssh_pubkey = crypto::p12_to_ssh_pubkey(&p12)
            .chain_err(|| format!("failed to convert pkcs12 blob to ssh pubkey"))?;

        Ok(ssh_pubkey)
    }
}

pub fn fetch_metadata() -> Result<Metadata> {
    let provider = Azure::new()
        .chain_err(|| format!("azure: failed create metadata client"))?;

    let ssh_pubkey = provider.get_ssh_pubkey()
        .chain_err(|| format!("azure: failed to get ssh pubkey"))?;

    Ok(Metadata::builder()
       .add_ssh_keys(vec![ssh_pubkey])
       .build())
}
