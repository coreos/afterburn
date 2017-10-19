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

//! Generate X509 certificate and associated RSA public/private keypair

use openssl::x509::{X509, X509Name};
use openssl::rsa::Rsa;
use openssl::pkey::PKey;
use openssl::hash::MessageDigest;
use openssl::asn1::Asn1Time;
use openssl::bn;
use openssl::conf::{Conf, ConfMethod};
use openssl::x509::extension;

use errors::*;

pub struct Config {
    rsa_bits: u32,
    expire_in_days: u32,
}

impl Config {
    pub fn new(rsa_bits: u32, expire_in_days: u32) -> Self {
        Config {
            rsa_bits: rsa_bits,
            expire_in_days: expire_in_days,
        }
    }
}

pub fn generate_cert(config: &Config) -> Result<(X509, PKey)> {
    // generate an rsa public/private keypair
    let rsa = Rsa::generate(config.rsa_bits)
        .chain_err(|| "failed to generate rsa keypair")?;
    // put it into the pkey struct
    let pkey = PKey::from_rsa(rsa)
        .chain_err(|| "failed to create pkey struct from rsa keypair")?;

    // make a new x509 certificate with the pkey we generated
    let mut x509builder = X509::builder()
        .chain_err(|| "failed to make x509 builder")?;
    x509builder.set_version(2)
        .chain_err(|| "failed to set x509 version")?;

    // set the serial number to some big random positive integer
    let mut serial = bn::BigNum::new()
        .chain_err(|| "failed to make new bignum")?;
    serial.rand(32, bn::MSB_ONE, false)
        .chain_err(|| "failed to generate random bignum")?;
    let serial = serial.to_asn1_integer()
        .chain_err(|| "failed to get asn1 integer from bignum")?;
    x509builder.set_serial_number(&serial)
        .chain_err(|| "failed to set x509 serial number")?;

    // call fails without expiration dates
    // I guess they are important anyway, but still
    x509builder.set_not_before(&Asn1Time::days_from_now(0).unwrap())
        .chain_err(|| "failed to set x509 start date")?;
    x509builder.set_not_after(&Asn1Time::days_from_now(config.expire_in_days).unwrap())
        .chain_err(|| "failed to set x509 expiration date")?;

    // add the issuer and subject name
    // it's set to "/CN=LinuxTransport"
    // if we want we can make that configurable later
    let mut x509namebuilder = X509Name::builder()
        .chain_err(|| "failed to get x509name builder")?;
    x509namebuilder.append_entry_by_text("CN", "LinuxTransport")
        .chain_err(|| "failed to append /CN=LinuxTransport to x509name builder")?;
    let x509name = x509namebuilder.build();
    x509builder.set_issuer_name(&x509name)
        .chain_err(|| "failed to set x509 issuer name")?;
    x509builder.set_subject_name(&x509name)
        .chain_err(|| "failed to set x509 subject name")?;

    // set the public key
    x509builder.set_pubkey(&pkey)
        .chain_err(|| "failed to set x509 pubkey")?;

    // it also needs several extensions
    // in the openssl configuration file, these are set when generating certs
    //     basicConstraints=CA:true
    //     subjectKeyIdentifier=hash
    //     authorityKeyIdentifier=keyid:always,issuer
    // that means these extensions get added to certs generated using the
    // command line tool automatically. but since we are constructing it, we
    // need to add them manually.
    // we need to do them one at a time, and they need to be in this order
    let conf = Conf::new(ConfMethod::default())
        .chain_err(|| "failed to make new conf struct")?;
    // it seems like everything depends on the basic constraints, so let's do
    // that first.
    let bc = extension::BasicConstraints::new()
        .ca()
        .build()
        .chain_err(|| "failed to build BasicConstraints extension")?;
    x509builder.append_extension(bc)
        .chain_err(|| "failed to append BasicConstraints extension")?;

    // the akid depends on the skid. I guess it copies the skid when the cert is
    // self-signed or something, I'm not really sure.
    let skid = {
        // we need to wrap these in a block because the builder gets borrowed away
        // from us
        let ext_con = x509builder.x509v3_context(None, Some(&conf));
        extension::SubjectKeyIdentifier::new()
            .build(&ext_con)
            .chain_err(|| "failed to build SubjectKeyIdentifier extention")?
    };
    x509builder.append_extension(skid)
        .chain_err(|| "failed to append SubjectKeyIdentifier extention")?;

    // now that the skid is added we can add the akid
    let akid = {
        let ext_con = x509builder.x509v3_context(None, Some(&conf));
        extension::AuthorityKeyIdentifier::new()
            .keyid(true)
            .issuer(false)
            .build(&ext_con)
            .chain_err(|| "failed to build AuthorityKeyIdentifier extention")?
    };
    x509builder.append_extension(akid)
        .chain_err(|| "failed to append AuthorityKeyIdentifier extention")?;

    // self-sign the certificate
    x509builder.sign(&pkey, MessageDigest::sha256())
        .chain_err(|| "failed to self-sign x509 cert")?;

    let x509 = x509builder.build();

    Ok((x509, pkey))
}
