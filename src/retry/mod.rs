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

//! retry is a generic function that retrys functions until they succeed.

use errors::*;
use std::time::Duration;
use std::thread;

pub mod raw_deserializer;
mod client;
pub use self::client::*;

#[derive(Clone, Debug)]
pub struct Retry {
    initial_backoff: Duration,
    max_backoff: Duration,
    max_attempts: u32,
}

impl ::std::default::Default for Retry {
    fn default() -> Self {
        Retry {
            initial_backoff: Duration::new(1,0),
            max_backoff: Duration::new(5,0),
            max_attempts: 10,
        }
    }
}

impl Retry {
    pub fn new() -> Self {
        Retry::default()
    }

    pub fn initial_backoff(mut self, initial_backoff: Duration) -> Self {
        self.initial_backoff = initial_backoff;
        self
    }

    pub fn max_backoff(mut self, max_backoff: Duration) -> Self {
        self.max_backoff = max_backoff;
        self
    }

    pub fn max_attempts(mut self, max_attempts: u32) -> Self {
        self.max_attempts = max_attempts;
        self
    }

    pub fn retry<F, R>(self, try: F) -> Result<R>
        where F: Fn(u32) -> Result<R>
    {
        let mut delay = self.initial_backoff;
        let mut attempts = 0;

        loop {
            let res = try(attempts);

            // if the result is ok, we don't need to try again
            if res.is_ok() {
                break res;
            }

            // otherwise, perform the retry-backoff logic
            attempts += 1;
            if attempts == self.max_attempts {
                break res.map_err(|e| Error::with_chain(e, "timed out"));
            }

            thread::sleep(delay);

            delay = if self.max_backoff != Duration::new(0,0) && delay * 2 > self.max_backoff {
                self.max_backoff
            } else {
                delay * 2
            };
        }
    }
}
