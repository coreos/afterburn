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
pub mod ssh;

use openssl::x509::X509;
use openssl::pkey::PKey;
use openssl::cms::CmsContentInfo;
use openssl::pkcs12::Pkcs12;

macro_rules! wrap_error {
    ($x:expr) => {
        |err| {
            format!("{}: {}", $x, err)
        }
    };
}

pub fn mangle_pem(x509: &X509) -> Result<String, String> {
    // get the pem
    let pem = x509.to_pem()
        .map_err(wrap_error!("failed to convert x509 cert to pem"))?;
    let pem = String::from_utf8(pem)
        .map_err(wrap_error!("failed to convert x509 pem file from utf8 to a string"))?;

    // the pem needs to be mangled to send as a header
    Ok(pem.lines()
        .filter(|l| !l.contains("BEGIN CERTIFICATE") && !l.contains("END CERTIFICATE"))
        .fold(String::new(), |mut s, l| {s.push_str(l); s}))
}

pub fn decrypt_cms(smime: &[u8], pkey: &PKey, x509: &X509) -> Result<Vec<u8>, String> {
    // now we need to read in that mime file
    let cms = CmsContentInfo::smime_read_cms(smime)
        .map_err(wrap_error!("failed to read cms file"))?;

    // and decrypt it's contents
    let p12_der = cms.decrypt(&pkey, &x509)
        .map_err(wrap_error!("failed to decrypt cms file"))?;

    Ok(p12_der)
}

pub fn p12_to_ssh_pubkey(p12_der: &[u8]) -> Result<String, String> {
    // the contents of that encrypted cms blob we got are actually a different
    // cryptographic structure. we read that in from the contents and parse it.
    // PKCS12 has the ability to have a password, but we don't have one, hence
    // empty string.
    let p12 = Pkcs12::from_der(&p12_der)
        .map_err(wrap_error!("failed to get pkcs12 blob from der"))?;
    let p12 = p12.parse("")
        .map_err(wrap_error!("failed to parse pkcs12 blob"))?;

    // PKCS12 has three parts. A pkey, a main x509 cert, and a list of other
    // x509 certs. The list of other x509 certs is called the chain. there is
    // only one cert in this chain, and it is the ssh public key.
    let ssh_pubkey_pem = p12.chain.get(0).unwrap();
    // .map_err(wrap_error!("failed to get cert from pkcs12 chain"))?;
    let ssh_pubkey_pem = ssh_pubkey_pem.public_key() // get the public key from the x509 cert
        .map_err(wrap_error!("failed to get public key from cert"))?;
    let ssh_pubkey_pem = ssh_pubkey_pem.rsa()        // get the rsa contents from the pkey struct
        .map_err(wrap_error!("failed to get rsa contents from pkey"))?;

    // so now I have a pkey struct which represents my rsa public key. This is
    // my ssh public key. However, I can only export it in pem or der. Ther are
    // no existing rust (or go for that matter) libraries which do the
    // conversion I want, so I guess I'll have to write my own function to do it
    let ssh_pubkey = ssh::rsa_to_ssh(&ssh_pubkey_pem, "");

    Ok(ssh_pubkey)
}
