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

use anyhow::{anyhow, Context, Result};
use reqwest::{self, blocking, header, Method};
use slog_scope::info;

use crate::retry::Retry;

use crate::retry::raw_deserializer;

pub trait Deserializer {
    fn deserialize<T, R>(&self, r: R) -> Result<T>
    where
        T: for<'de> serde::Deserialize<'de>,
        R: Read;
    fn content_type(&self) -> header::HeaderValue;
}

#[derive(Debug, Clone, Copy)]
pub struct Xml;

impl Deserializer for Xml {
    fn deserialize<T, R>(&self, r: R) -> Result<T>
    where
        T: for<'de> serde::Deserialize<'de>,
        R: Read,
    {
        serde_xml_rs::de::from_reader(r).context("failed xml deserialization")
    }

    fn content_type(&self) -> header::HeaderValue {
        header::HeaderValue::from_static("text/xml; charset=utf-8")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Json;

impl Deserializer for Json {
    fn deserialize<T, R>(&self, r: R) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
        R: Read,
    {
        serde_json::from_reader(r).context("failed json deserialization")
    }
    fn content_type(&self) -> header::HeaderValue {
        header::HeaderValue::from_static("application/json")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Raw;

impl Deserializer for Raw {
    fn deserialize<T, R>(&self, r: R) -> Result<T>
    where
        T: for<'de> serde::Deserialize<'de>,
        R: Read,
    {
        raw_deserializer::from_reader(r).context("failed raw deserialization")
    }
    fn content_type(&self) -> header::HeaderValue {
        header::HeaderValue::from_static("text/plain; charset=utf-8")
    }
}

#[derive(Debug, Clone)]
pub struct Client {
    client: blocking::Client,
    headers: header::HeaderMap,
    retry: Retry,
    return_on_404: bool,
    #[cfg(test)]
    mock_base_url: Option<String>,
}

impl Client {
    pub fn try_new() -> Result<Self> {
        let client = blocking::Client::builder()
            .build()
            .context("failed to initialize client")?;
        Ok(Client {
            client,
            headers: header::HeaderMap::new(),
            retry: Retry::new(),
            return_on_404: false,
            #[cfg(test)]
            mock_base_url: None,
        })
    }

    pub fn header(mut self, k: header::HeaderName, v: header::HeaderValue) -> Self {
        self.headers.append(k, v);
        self
    }

    #[allow(dead_code)]
    pub fn initial_backoff(mut self, initial_backoff: Duration) -> Self {
        self.retry = self.retry.initial_backoff(initial_backoff);
        self
    }

    #[allow(dead_code)]
    pub fn max_backoff(mut self, max_backoff: Duration) -> Self {
        self.retry = self.retry.max_backoff(max_backoff);
        self
    }

    /// Maximum number of retries to attempt.
    ///
    /// If zero, only the initial request will be performed, with no
    /// additional retries.
    #[allow(dead_code)]
    pub fn max_retries(mut self, retries: u8) -> Self {
        self.retry = self.retry.max_retries(retries);
        self
    }

    pub fn return_on_404(mut self, return_on_404: bool) -> Self {
        self.return_on_404 = return_on_404;
        self
    }

    #[cfg(test)]
    pub fn mock_base_url(mut self, base_url: String) -> Self {
        self.mock_base_url = Some(base_url);
        self
    }

    pub fn get<D>(&self, d: D, url: String) -> RequestBuilder<D>
    where
        D: Deserializer,
    {
        RequestBuilder {
            url,
            body: None,
            d,
            client: self.client.clone(),
            headers: self.headers.clone(),
            retry: self.retry.clone(),
            return_on_404: self.return_on_404,
            #[cfg(test)]
            mock_base_url: self.mock_base_url.clone(),
        }
    }

    pub fn post<D>(&self, d: D, url: String, body: Option<Cow<str>>) -> RequestBuilder<D>
    where
        D: Deserializer,
    {
        RequestBuilder {
            url,
            body: body.map(Cow::into_owned),
            d,
            client: self.client.clone(),
            headers: self.headers.clone(),
            retry: self.retry.clone(),
            return_on_404: self.return_on_404,
            #[cfg(test)]
            mock_base_url: self.mock_base_url.clone(),
        }
    }

    pub fn put<D>(&self, d: D, url: String, body: Option<Cow<str>>) -> RequestBuilder<D>
    where
        D: Deserializer,
    {
        RequestBuilder {
            url,
            body: body.map(Cow::into_owned),
            d,
            client: self.client.clone(),
            headers: self.headers.clone(),
            retry: self.retry.clone(),
            return_on_404: self.return_on_404,
            #[cfg(test)]
            mock_base_url: self.mock_base_url.clone(),
        }
    }
}

pub struct RequestBuilder<D>
where
    D: Deserializer,
{
    url: String,
    body: Option<String>,
    d: D,
    client: blocking::Client,
    headers: header::HeaderMap,
    retry: Retry,
    return_on_404: bool,
    #[cfg(test)]
    mock_base_url: Option<String>,
}

impl<D> RequestBuilder<D>
where
    D: Deserializer,
{
    pub fn header(mut self, k: header::HeaderName, v: header::HeaderValue) -> Self {
        self.headers.append(k, v);
        self
    }

    pub fn send<T>(self) -> Result<Option<T>>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let url = self.parse_url()?;
        let mut req = blocking::Request::new(Method::GET, url);
        req.headers_mut().extend(self.headers.clone());

        self.retry.clone().retry(|attempt| {
            info!("Fetching {}: Attempt #{}", req.url(), attempt + 1);
            self.dispatch_request(&req)
        })
    }

    pub fn dispatch_put<T>(self) -> Result<Option<T>>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let url = self.parse_url()?;

        self.retry.clone().retry(|attempt| {
            let mut builder = blocking::Client::new()
                .put(url.clone())
                .headers(self.headers.clone())
                .header(header::CONTENT_TYPE, self.d.content_type());
            if let Some(ref content) = self.body {
                builder = builder.body(content.clone());
            };
            let req = builder.build().context("failed to build PUT request")?;

            info!("Putting {}: Attempt #{}", req.url(), attempt + 1);
            let response = self.client.execute(req).context("failed to PUT request")?;
            let status = response.status();
            if status.is_success() {
                self.d
                    .deserialize(response)
                    .map(Some)
                    .context("failed to deserialize data")
            } else {
                Err(anyhow!("PUT failed: {}", status))
            }
        })
    }

    pub fn dispatch_post(self) -> Result<reqwest::StatusCode> {
        let url = self.parse_url()?;

        self.retry.clone().retry(|attempt| {
            let mut builder = blocking::Client::new()
                .post(url.clone())
                .headers(self.headers.clone())
                .header(header::CONTENT_TYPE, self.d.content_type());
            if let Some(ref content) = self.body {
                builder = builder.body(content.clone());
            };
            let req = builder.build().context("failed to build POST request")?;

            info!("Posting {}: Attempt #{}", req.url(), attempt + 1);
            let status = self
                .client
                .execute(req)
                .context("failed to POST request")?
                .status();
            if status.is_success() {
                Ok(status)
            } else {
                Err(anyhow!("POST failed: {}", status))
            }
        })
    }

    fn dispatch_request<T>(&self, req: &blocking::Request) -> Result<Option<T>>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        match self.client.execute(clone_request(req)) {
            Ok(resp) => match (resp.status(), self.return_on_404) {
                (reqwest::StatusCode::OK, _) => {
                    info!("Fetch successful");
                    self.d
                        .deserialize(resp)
                        .map(Some)
                        .context("failed to deserialize data")
                }
                (reqwest::StatusCode::NOT_FOUND, true) => {
                    info!("Fetch failed with 404: resource not found");
                    Ok(None)
                }
                (s, _) => {
                    info!("Failed to fetch: {}", s);
                    Err(anyhow!("failed to fetch: {}", s))
                }
            },
            Err(e) => {
                info!("Failed to fetch: {}", e);
                Err(anyhow!(e).context("failed to fetch"))
            }
        }
    }

    fn parse_url(&self) -> Result<reqwest::Url> {
        #[allow(unused_mut)]
        let mut url = reqwest::Url::parse(self.url.as_str()).context("failed to parse uri")?;
        #[cfg(test)]
        if let Some(mock_base_url) = &self.mock_base_url {
            let base_url =
                reqwest::Url::parse(mock_base_url).context("failed to parse mock base URL")?;
            url.set_scheme(base_url.scheme())
                .map_err(|_| anyhow!("failed to update URL scheme"))?;
            let host = base_url
                .host()
                .context("mock base URL doesn't have a host")?
                .to_string();
            url.set_host(Some(&host))
                .context("failed to update URL host")?;
            url.set_port(base_url.port())
                .map_err(|_| anyhow!("failed to update URL port"))?;
        }
        Ok(url)
    }
}

/// Reqwests Request struct doesn't implement `Clone`,
/// so we have to do it here.
fn clone_request(req: &blocking::Request) -> blocking::Request {
    let mut newreq = blocking::Request::new(req.method().clone(), req.url().clone());
    newreq.headers_mut().extend(req.headers().clone());
    newreq
}
