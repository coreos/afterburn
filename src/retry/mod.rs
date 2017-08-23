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

use std::io::Read;
use std::time::Duration;
use std::thread;

use reqwest;
use reqwest::header;
use reqwest::header::ContentType;
use reqwest::{Method,Request};

use serde;
use serde_xml_rs;

use errors::*;

#[inline(always)]
pub fn default_initial_backoff() -> Duration { Duration::new(1,0) }
#[inline(always)]
pub fn default_max_backoff() -> Duration { Duration::new(5,0) }
#[inline(always)]
pub fn default_max_attempts() -> u32 { 0 }

pub trait Deserializer {
    fn deserialize<'de, T>(&self, &str) -> Result<T>
        where T: serde::Deserialize<'de>;
    fn content_type(&self) -> ContentType;
}

#[derive(Debug, Clone, Copy)]
pub struct Xml;

impl Deserializer for Xml {
    fn deserialize<'de, T>(&self, input: &str) -> Result<T>
        where T: serde::Deserialize<'de>
    {
        serde_xml_rs::deserialize(input.as_bytes())
            .chain_err(|| format!("failed xml deserialization"))
    }
    fn content_type(&self) -> ContentType {
        ContentType("text/xml; charset=utf-8".parse().unwrap())
    }
}

#[derive(Debug, Clone)]
pub struct Client {
    client: reqwest::Client,
    headers: header::Headers,
    initial_backoff: Duration,
    max_backoff: Duration,
    max_attempts: u32,
    return_on_404: bool,
}

impl Client {
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::new()
            .chain_err(|| format!("failed to initialize client"))?;
        Ok(Client{
            client,
            headers: header::Headers::new(),
            initial_backoff: default_initial_backoff(),
            max_backoff:  default_max_backoff(),
            max_attempts: default_max_attempts(),
            return_on_404: false,
        })
    }

    pub fn header<H>(mut self, h: H) -> Self
        where H: header::Header
    {
        self.headers.set(h);
        self
    }

    pub fn initial_backoff(mut self, initial_backoff: Duration) -> Self {
        self.initial_backoff = initial_backoff;
        self
    }

    pub fn max_backoff(mut self, max_backoff: Duration) -> Self {
        self.max_backoff = max_backoff;
        self
    }

    /// max_attempts will panic if the argument is greater than 500
    pub fn max_attempts(mut self, max_attempts: u32) -> Self {
        if max_attempts > 500 {
            // Picking 500 as a max_attempts number arbitrarily, to prevent the mutually recursive
            // functions dispatch_request and wait_then_retry from blowing up the stack
            panic!("max_attempts cannot be greater than 500")
        } else {
            self.max_attempts = max_attempts;
            self
        }
    }

    pub fn return_on_404(mut self, return_on_404: bool) -> Self {
        self.return_on_404 = return_on_404;
        self
    }

    pub fn get<D>(&self, d: D, url: String) -> RequestBuilder<D>
        where D: Deserializer
    {
        RequestBuilder{
            url,
            d,
            client: self.client.clone(),
            headers: self.headers.clone(),
            initial_backoff: self.initial_backoff.clone(),
            max_backoff: self.max_backoff.clone(),
            max_attempts: self.max_attempts.clone(),
            return_on_404: self.return_on_404.clone(),
        }
    }
}

pub struct RequestBuilder<D>
    where D: Deserializer
{
    url: String,
    d: D,
    client: reqwest::Client,
    headers: header::Headers,
    initial_backoff: Duration,
    max_backoff: Duration,
    max_attempts: u32,
    return_on_404: bool,
}

impl<D> RequestBuilder<D>
    where D: Deserializer
{

    pub fn header<H>(mut self, h: H) -> Self
        where H: header::Header
    {
        self.headers.set(h);
        self
    }

    pub fn send<'de, T>(&self) -> Result<T>
        where T: serde::Deserialize<'de> + 'de
    {
        let url = reqwest::Url::parse(self.url.as_str())
            .chain_err(|| format!("failed to parse uri"))?;
        let mut req = Request::new(Method::Get, url);
        req.headers_mut().extend(self.headers.iter());
        req.headers_mut().set(self.d.content_type());
        self.dispatch_request(req, self.initial_backoff, 0)
    }

    fn dispatch_request<'de, T>(&self, req: Request, delay: Duration, num_attempts: u32) -> Result<T>
        where T: serde::Deserialize<'de> + 'de
    {
        info!("Fetching {}: Attempt#{}", req.url(), num_attempts + 1);

        match self.client.execute(clone_request(&req)) {
            Ok(mut resp) => {
                match (resp.status(), self.return_on_404) {
                    (reqwest::StatusCode::Ok,_) => {
                        let mut buf = String::new();
                        match resp.read_to_string(&mut buf) {
                            Ok(_) => {
                                self.d.deserialize(buf.as_str())
                                    .chain_err(|| format!("failed to deserialize data"))
                            }
                            Err(e) => {
                                info!("error reading body: {}", e);
                                self.wait_then_retry(req, delay, num_attempts)
                            }
                        }
                    }
                    (reqwest::StatusCode::NotFound,true) => {
                        // TODO: return empty (failed?)
                        error!("Failed to fetch: should return!");
                        self.wait_then_retry(req, delay, num_attempts)
                    }
                    (s,_) => {
                        info!("Failed to fetch: {}", s);
                        self.wait_then_retry(req, delay, num_attempts)
                    }
                }
            }
            Err(e) => {
                info!("Failed to fetch: {}", e);
                self.wait_then_retry(req, delay, num_attempts)
            }
        }
    }

    fn wait_then_retry<'de, T>(&self, req: Request, delay: Duration, num_attempts: u32) -> Result<T>
        where T: serde::Deserialize<'de> + 'de
    {
        thread::sleep(delay);
        let delay = if self.max_backoff != Duration::new(0,0) && delay * 2 > self.max_backoff {
                self.max_backoff
            } else {
                delay * 2
            };
        let num_attempts = num_attempts + 1;
        if self.max_attempts != 0 && num_attempts == self.max_attempts {
            Err(format!("Timed out while fetching {}", req.url()).into())
        } else {
            self.dispatch_request(req, delay, num_attempts)
        }
    }
}

fn clone_request(req: &Request) -> Request {
    let mut newreq = Request::new(req.method().clone(), req.url().clone());
    newreq.headers_mut().extend(req.headers().iter());
    newreq
}
