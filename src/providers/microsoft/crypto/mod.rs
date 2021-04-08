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

//! crypto module takes care of cryptographic functions

pub mod x509;

use openssl::cms::CmsContentInfo;
use openssl::pkcs12::Pkcs12;
use openssl::pkey::{PKey, Private};
use openssl::x509::X509;

use anyhow::{anyhow, Context, Result};
use openssh_keys::PublicKey;

pub fn mangle_pem(x509: &X509) -> Result<String> {
    // get the pem
    let pem = x509
        .to_pem()
        .context("failed to convert x509 cert to pem")?;
    let pem =
        String::from_utf8(pem).context("failed to convert x509 pem file from utf8 to a string")?;

    // the pem needs to be mangled to send as a header
    Ok(pem
        .lines()
        .filter(|l| !l.contains("BEGIN CERTIFICATE") && !l.contains("END CERTIFICATE"))
        .fold(String::new(), |mut s, l| {
            s.push_str(l);
            s
        }))
}

pub fn decrypt_cms(smime: &[u8], pkey: &PKey<Private>, x509: &X509) -> Result<Vec<u8>> {
    // now we need to read in that mime file
    let cms = CmsContentInfo::smime_read_cms(smime).context("failed to read cms file")?;

    // and decrypt it's contents
    let p12_der = cms
        .decrypt(pkey, x509)
        .context("failed to decrypt cms file")?;

    Ok(p12_der)
}

pub fn p12_to_ssh_pubkey(p12_der: &[u8]) -> Result<PublicKey> {
    // the contents of that encrypted cms blob we got are actually a different
    // cryptographic structure. we read that in from the contents and parse it.
    // PKCS12 has the ability to have a password, but we don't have one, hence
    // empty string.
    let p12 = Pkcs12::from_der(p12_der).context("failed to get pkcs12 blob from der")?;
    let p12 = p12.parse("").context("failed to parse pkcs12 blob")?;

    // PKCS12 has three parts: a pkey, a main x509 cert, and a list of other
    // x509 certs. The list of other x509 certs is called the chain. There is
    // only one cert in this chain, and it is the ssh public key.
    let chain = p12
        .chain
        .ok_or_else(|| anyhow!("failed to get chain from pkcs12"))?;
    let ssh_pem = chain
        .get(0)
        .ok_or_else(|| anyhow!("failed to get cert from pkcs12 chain"))?;
    // get the public key from the x509 cert
    let ssh_pubkey_pem = ssh_pem
        .public_key()
        .context("failed to get public key from cert")?;
    // get the rsa contents from the pkey struct
    let ssh_pubkey_rsa = ssh_pubkey_pem
        .rsa()
        .context("failed to get rsa contents from pkey")?;

    // convert the openssl Rsa public key to an OpenSSH public key in string format
    let e = ssh_pubkey_rsa.e().to_vec();
    let n = ssh_pubkey_rsa.n().to_vec();
    let ssh_pubkey = PublicKey::from_rsa(e, n);

    Ok(ssh_pubkey)
}
