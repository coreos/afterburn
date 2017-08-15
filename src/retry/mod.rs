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

//! retry
//!
//! this is an abstraction over the regular http get request. it allows you to
//! have a request retry until it succeeds, with a configurable number of
//! of attempts and a backoff strategy. It also takes care of automatically
//! deserializing responses and handles headers in a sane way.

use hyper;
use hyper::header;

use serde;
use serde_xml_rs;

use std::io::Read;
use std::marker::Copy;

#[derive(Debug)]
pub struct Client<D>
    where D: Deserializer
{
    headers: header::Headers,
    deserializer: D,
    client: hyper::client::Client,
}

pub trait Deserializer {
    fn deserialize<'de, T>(&self, &str) -> Result<T, String>
        where T: serde::Deserialize<'de>;
}

#[derive(Debug, Clone, Copy)]
pub struct Xml;

impl Deserializer for Xml {
    fn deserialize<'de, T>(&self, input: &str) -> Result<T, String>
        where T: serde::Deserialize<'de>
    {
        serde_xml_rs::deserialize(input.as_bytes())
            .map_err(wrap_error!("failed xml deserialization"))
    }
}

pub struct RequestBuilder<'a, D>
    where D: Deserializer
{
    req: hyper::client::RequestBuilder<'a>,
    deserializer: D,
    uri: String,
}

impl<'a, D> RequestBuilder<'a, D>
    where D: Deserializer
{
    pub fn header<H>(mut self, header: H) -> Self
        where H: header::Header + header::HeaderFormat
    {
        self.req = self.req.header(header);
        self
    }

    pub fn send<'de, T>(self) -> Result<T, String>
        where T: serde::Deserialize<'de> + 'de
    {
        // save uri for logging
        let uri = self.uri;

        let mut res = self.req.send()
            .map_err(wrap_error!("failed to request from uri '{}'", uri))?;

        let mut body = String::new();
        res.read_to_string(&mut body)
            .map_err(wrap_error!("failed to read response body"))?;

        trace!("http get response from '{}': {}", uri, body);

        self.deserializer.deserialize(&body)
            .map_err(wrap_error!("failed to deserialize xml into versions struct"))
    }
}

impl<D> Client<D>
    where D: Deserializer + Copy
{
    pub fn new(deserializer: D) -> Self {
        Client {
            deserializer: deserializer,
            headers: header::Headers::new(),
            client: hyper::client::Client::new(),
        }
    }

    pub fn header<H>(mut self, header: H) -> Self
        where H: header::Header + header::HeaderFormat
    {
        self.headers.set(header);
        self
    }

    pub fn get(&self, uri: String) -> RequestBuilder<D> {
        trace!("http get request to '{}' with headers '{:?}'", uri, self.headers);
        let req = self.client.get(&uri).headers(self.headers.clone());
        RequestBuilder {
            req: req,
            uri: uri,
            deserializer: self.deserializer,
        }
    }
}
