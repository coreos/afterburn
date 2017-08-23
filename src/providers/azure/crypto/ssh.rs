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

//! OpenSSH Public Keys
//!
//! This crate converts to and from OpenSSH Public Keys. Right now, there is
//! just decoding from generic RSA to OpenSSH Public Key format.
//!
//! To use this library:
//!
//! ```
//! extern crate openssl;
//! extern crate ssh;
//!
//! use openssl::rsa::Rsa;
//!
//! let rsa = Rsa::generate(2048).unwrap();
//! let ssh_pubkey = ssh::rsa_to_ssh(rsa, "user@host");
//! println!("my ssh public key: {}", ssh_pubkey);
//! ```

use base64;

use byteorder::{WriteBytesExt, BigEndian};

use openssl::rsa::Rsa;
use openssl::bn;

// an ssh key consists of three pieces:
//    ssh-keytype data comment
// our ssh-keytype is ssh-rsa.
// the data also consists of three pieces (for an rsa key):
//    ssh-rsa public-exponent modulus
// each of those is encoded as big-endian bytes preceeded by four bytes
// representing their length.
// for details:
// see ssh-rsa format in https://tools.ietf.org/html/rfc4253#section-6.6
pub fn rsa_to_ssh(rsa: &Rsa, comment: &str) -> String {
    // first it has the keytype
    let keytype = "ssh-rsa";
    let mut key = encode_ssh(keytype.as_bytes().to_vec());

    // then it has the encoded public exponent
    let e = rsa.e().unwrap();
    key.append(&mut encode_mpint(e));

    // last it has the endcoded modulus
    let n = rsa.n().unwrap();
    key.append(&mut encode_mpint(n));

    // the data section is base64 encoded
    let data = base64::encode(&key);

    format!("{} {} {}", keytype, data, comment)
}

// according to RFC 4251, the mpint datatype representation is a big-endian
// arbitrary-precision integer stored in two's compliment and stored as a
// string with the minimum possible number of characters.
// see mpint definition in https://tools.ietf.org/html/rfc4251#section-5
fn encode_mpint(num: &bn::BigNumRef) -> Vec<u8> {
    // bignum.to_vec() gives big-endian results
    let mut buf = num.to_vec();

    // If the number is positive (which ours are going to be because of the rsa
    // spec), then we are required to guarentee that the most significant bit is
    // set to zero if the first bit in the first byte is going to be one.
    if buf.get(0).unwrap() & 0x80 != 0 {
        buf.insert(0, 0);
    }

    // other than that it's just normal ssh encoding
    encode_ssh(buf)
}

// a datatype in ssh is encoded as 4 bytes representing the size, followed by
// the data itself, all in big-endian. Since we don't really the endian-ness of
// the provided Vec<u8>, we just assume it's correct and worry about the length.
fn encode_ssh(mut buf: Vec<u8>) -> Vec<u8> {
    let mut encoded: Vec<u8> = Vec::new();

    // The first four bytes represent the length of the encoded data.
    // BigEndian::write_u32(&mut encoded, buf.len() as u32);
    encoded.write_u32::<BigEndian>(buf.len() as u32).unwrap();

    // the rest of the bytes are the data itself
    encoded.append(&mut buf);

    encoded
}
