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

mod crypto;

use self::crypto::x509;

use metadata::Metadata;

use hyper::client::Client;
use hyper::header;
use hyper::mime;

use serde_xml_rs::deserialize;

use std::net::{IpAddr, Ipv4Addr};
use std::io::Read;

header! {(MSAgentName, "x-ms-agent-name") => [String]}
header! {(MSVersion, "x-ms-version") => [String]}
header! {(MSCipherName, "x-ms-cipher-name") => [String]}
header! {(MSCert, "x-ms-guest-agent-public-x509-cert") => [String]}

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

struct Azure {
    client: Client,
}

impl Azure {
    fn new() -> Self {
        Azure {
            client: Client::new(),
        }
    }

    //TODO(sdemos): get the actual fabric address, instead of hardcoding it
    fn get_fabric_address(&self) -> Result<IpAddr, String> {
        Ok(IpAddr::V4(Ipv4Addr::new(168, 63, 129, 16)))
    }

    fn get_certs_endpoint(&self) -> Result<String, String> {
        // get the address that we need to call to get the goalstate
        let addr = self.get_fabric_address()
            .map_err(wrap_error!("failed to get fabric address"))?;

        // make the request to the goalstate endpoint
        let mut res = self.client.get(&format!("http://{}/machine/?comp=goalstate", addr))
            .header(MSAgentName(MS_AGENT_NAME.to_owned()))
            .header(MSVersion(MS_VERSION.to_owned()))
            .header(header::ContentType(mime::Mime(mime::TopLevel::Text, mime::SubLevel::Xml, vec![(mime::Attr::Charset, mime::Value::Utf8)])))
            .send()
            .map_err(wrap_error!("failed to request goal state"))?;

        // read the goalstate body
        let mut body = String::new();
        res.read_to_string(&mut body)
            .map_err(wrap_error!("failed to read goal state response body"))?;

        // then deserialize the response into xml
        let goalstate: GoalState = deserialize(body.as_bytes())
            .map_err(wrap_error!("failed to deserialize xml into goalstate struct"))?;

        // grab the certificates endpoint from the xml and return it
        let cert_endpoint: &str = &goalstate.container.role_instance_list.role_instances[0].configuration.certificates;
        Ok(String::from(cert_endpoint))
    }

    fn get_certs(&self, endpoint: String, mangled_pem: String) -> Result<String, String> {
        // we need to make the request to the endpoint we got earlier to get the
        // certificates file.
        let mut res = self.client.get(&endpoint)
            .header(MSAgentName(MS_AGENT_NAME.to_owned()))
            .header(MSVersion(MS_VERSION.to_owned()))
            .header(header::ContentType(mime::Mime(mime::TopLevel::Text, mime::SubLevel::Xml, vec![(mime::Attr::Charset, mime::Value::Utf8)])))
            .header(MSCipherName("DES_EDE3_CBC".to_owned()))
            .header(MSCert(mangled_pem))
            .send()
            .map_err(wrap_error!("failed to fetch certificates"))?;

        let mut body = String::new();
        res.read_to_string(&mut body)
            .map_err(wrap_error!("failed to read certificates file from response body"))?;

        // deserialize that too.
        let certs: CertificatesFile = deserialize(body.as_bytes())
            .map_err(wrap_error!("failed to deserialize xml into CertificatesFile struct"))?;

        // the cms decryption expects it to have MIME information on the top
        // since cms is really for email attachments...don't tell the cops.
        let mut smime = String::from(SMIME_HEADER);
        smime.push_str(&certs.data);

        Ok(smime)
    }

    // put it all together
    fn get_ssh_pubkey(&self) -> Result<String, String> {
        // first we have to get the certificates endoint.
        let certs_endpoint = self.get_certs_endpoint()
            .map_err(wrap_error!("failed to get certs endpoint"))?;

        // we have to generate the rsa public/private keypair and the x509 cert
        // that we use to make the request. this is equivalent to
        // `openssl req -x509 -nodes -subj /CN=LinuxTransport -days 365 -newkey rsa:2048 -keyout private.pem -out cert.pem`
        let (x509, pkey) = x509::generate_cert(x509::Config::new(2048, 365))
            .map_err(wrap_error!("failed to generate keys"))?;

        // mangle the pem file for the request
        let mangled_pem = crypto::mangle_pem(&x509)
            .map_err(wrap_error!("failed to mangle pem"))?;

        // fetch the encrypted cms blob from the certs endpoint
        let smime = self.get_certs(certs_endpoint, mangled_pem)
            .map_err(wrap_error!("failed to get certs"))?;

        // decrypt the cms blob
        let p12 = crypto::decrypt_cms(smime.as_bytes(), &pkey, &x509)
            .map_err(wrap_error!("failed to decrypt cms blob"))?;

        // convert that to the OpenSSH public key format
        let ssh_pubkey = crypto::p12_to_ssh_pubkey(&p12)
            .map_err(wrap_error!("failed to convert pkcs12 blob to ssh pubkey"))?;

        Ok(ssh_pubkey)
    }
}

pub fn fetch_metadata() -> Result<Metadata, String> {
    let provider = Azure::new();

    let ssh_pubkey = provider.get_ssh_pubkey()
        .map_err(wrap_error!("azure: failed to get ssh pubkey"))?;

    Ok(Metadata::builder()
       .add_ssh_key(ssh_pubkey)
       .build())
}
