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

use std::borrow::Cow;
use std::io::Read;
use std::time::Duration;

use reqwest::{self, Method, Request};
use reqwest::header;

use serde;
use serde_xml_rs;
use serde_json;

use retry::Retry;
use errors::*;

use retry::raw_deserializer;

pub trait Deserializer {
    fn deserialize<T, R>(&self, R) -> Result<T>
        where T: for<'de> serde::Deserialize<'de>, R: Read;
    fn content_type(&self) -> header::HeaderValue;
}

#[derive(Debug, Clone, Copy)]
pub struct Xml;

impl Deserializer for Xml {
    fn deserialize<T, R>(&self, r: R) -> Result<T>
        where T: for<'de> serde::Deserialize<'de>, R: Read
    {
        serde_xml_rs::deserialize(r)
            .chain_err(|| "failed xml deserialization")
    }
    fn content_type(&self) -> header::HeaderValue {
        header::HeaderValue::from_static("text/xml; charset=utf-8")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Json;

impl Deserializer for Json {
    fn deserialize<T, R>(&self, r: R) -> Result<T>
        where T: serde::de::DeserializeOwned, R: Read
    {
        serde_json::from_reader(r)
            .chain_err(|| "failed json deserialization")
    }
    fn content_type(&self) -> header::HeaderValue {
        header::HeaderValue::from_static("text/json; charset=utf-8")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Raw;

impl Deserializer for Raw {
    fn deserialize<T, R>(&self, r: R) -> Result<T>
        where T: for<'de> serde::Deserialize<'de>, R: Read
    {
        raw_deserializer::from_reader(r)
            .chain_err(|| "failed raw deserialization")
    }
    fn content_type(&self) -> header::HeaderValue {
        header::HeaderValue::from_static("text/plain; charset=utf-8")
    }
}

#[derive(Debug, Clone)]
pub struct Client {
    client: reqwest::Client,
    headers: header::HeaderMap,
    retry: Retry,
    return_on_404: bool,
}

impl Client {
    pub fn try_new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .build()
            .chain_err(|| "failed to initialize client")?;
        Ok(Client{
            client,
            headers: header::HeaderMap::new(),
            retry: Retry::new(),
            return_on_404: false,
        })
    }

    pub fn header(mut self, k: header::HeaderName, v: header::HeaderValue) -> Self
    {
        self.headers.append(k, v);
        self
    }

    pub fn initial_backoff(mut self, initial_backoff: Duration) -> Self {
        self.retry = self.retry.initial_backoff(initial_backoff);
        self
    }

    pub fn max_backoff(mut self, max_backoff: Duration) -> Self {
        self.retry = self.retry.max_backoff(max_backoff);
        self
    }

    /// max_attempts will panic if the argument is greater than 500
    pub fn max_attempts(mut self, max_attempts: u32) -> Self {
        self.retry = self.retry.max_attempts(max_attempts);
        self
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
            body: None,
            d,
            client: self.client.clone(),
            headers: self.headers.clone(),
            retry: self.retry.clone(),
            return_on_404: self.return_on_404,
        }
    }

    pub fn post<D>(&self, d: D, url: String, body: Option<Cow<str>>) -> RequestBuilder<D>
        where D: Deserializer
    {
        RequestBuilder{
            url,
            body: body.map(|b| b.into_owned()),
            d,
            client: self.client.clone(),
            headers: self.headers.clone(),
            retry: self.retry.clone(),
            return_on_404: self.return_on_404,
        }
    }
}

pub struct RequestBuilder<D>
    where D: Deserializer
{
    url: String,
    body: Option<String>,
    d: D,
    client: reqwest::Client,
    headers: header::HeaderMap,
    retry: Retry,
    return_on_404: bool,
}

impl<D> RequestBuilder<D>
    where D: Deserializer
{

    pub fn header(mut self, k: header::HeaderName, v: header::HeaderValue) -> Self
    {
        self.headers.append(k, v);
        self
    }

    pub fn send<T>(self) -> Result<Option<T>>
        where T: for<'de> serde::Deserialize<'de>
    {
        let url = reqwest::Url::parse(self.url.as_str())
            .chain_err(|| "failed to parse uri")?;
        let mut req = Request::new(Method::GET, url);
        req.headers_mut().extend(self.headers.clone().into_iter());

        self.retry.clone().retry(|attempt| {
            info!("Fetching {}: Attempt #{}", req.url(), attempt + 1);
            self.dispatch_request(&req)
        })
    }

    pub fn dispatch_post(self) -> Result<reqwest::StatusCode>
    {
        let url = reqwest::Url::parse(self.url.as_str())
            .chain_err(|| "failed to parse uri")?;

        self.retry.clone().retry(|attempt| {
            let mut builder = reqwest::Client::new()
                .post(url.clone())
                .headers(self.headers.clone())
                .header(header::CONTENT_TYPE, self.d.content_type());
            if let Some(ref content) = self.body {
                builder = builder.body(content.clone());
            };
            let req = builder.build()
                .chain_err(|| "failed to build POST request")?;

            info!("Posting {}: Attempt #{}", req.url(), attempt + 1);
            let status = self.client.execute(req)
                .chain_err(|| "failed to POST request")?
                .status();
            if status.is_success() {
                Ok(status)
            } else {
                Err(format!("POST failed: {}", status).into())
            }
        })
    }

    fn dispatch_request<T>(&self, req: &Request) -> Result<Option<T>>
        where T: for<'de> serde::Deserialize<'de>
    {
        match self.client.execute(clone_request(req)) {
            Ok(resp) => {
                match (resp.status(), self.return_on_404) {
                    (reqwest::StatusCode::OK,_) => {
                        info!("Fetch successful");
                        self.d.deserialize(resp)
                            .map(Some)
                            .chain_err(|| "failed to deserialize data")
                    }
                    (reqwest::StatusCode::NOT_FOUND,true) => {
                        info!("Fetch failed with 404: resource not found");
                        Ok(None)
                    }
                    (s,_) => {
                        info!("Failed to fetch: {}", s);
                        Err(format!("failed to fetch: {}", s).into())
                    }
                }
            }
            Err(e) => {
                info!("Failed to fetch: {}", e);
                Err(Error::with_chain(e, "failed to fetch"))
            }
        }
    }
}

/// Reqwests Request struct doesn't implement `Clone`,
/// so we have to do it here.
fn clone_request(req: &Request) -> Request {
    let mut newreq = Request::new(req.method().clone(), req.url().clone());
    newreq.headers_mut().extend(req.headers().clone().into_iter());
    newreq
}
