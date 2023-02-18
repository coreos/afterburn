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

//! Drive a functions through a finite number of retries until it succeeds.

use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};

mod client;
pub mod raw_deserializer;
pub use self::client::*;

#[derive(Clone, Debug)]
pub struct Retry {
    initial_backoff: Duration,
    max_backoff: Duration,
    max_retries: u8,
}

impl Default for Retry {
    fn default() -> Self {
        Retry {
            initial_backoff: Duration::new(1, 0),
            max_backoff: Duration::new(5, 0),
            max_retries: 10,
        }
    }
}

impl Retry {
    /// Build a new retrying driver.
    ///
    /// This defaults to 10 retries with 5 seconds maximum backoff.
    pub fn new() -> Self {
        Retry::default()
    }

    /// Set the initial backoff.
    #[allow(dead_code)]
    pub fn initial_backoff(mut self, initial_backoff: Duration) -> Self {
        self.initial_backoff = initial_backoff;
        self
    }

    /// Set the maximum backoff.
    #[allow(dead_code)]
    pub fn max_backoff(mut self, max_backoff: Duration) -> Self {
        self.max_backoff = max_backoff;
        self
    }

    /// Maximum number of retries to attempt.
    ///
    /// If zero, only the initial run will be performed, with no
    /// additional retries.
    pub fn max_retries(mut self, retries: u8) -> Self {
        self.max_retries = retries;
        self
    }

    /// Retry a function until it either succeeds once or fails all the time.
    pub fn retry<F, R>(self, try_fn: F) -> Result<R>
    where
        F: Fn(u8) -> Result<R>,
    {
        let mut delay = self.initial_backoff;
        let mut attempts = 0;

        loop {
            let res = try_fn(attempts);

            // If the result is ok, there is no need to try again.
            if res.is_ok() {
                break res;
            }

            // Otherwise, perform "the retry with backoff" logic.
            if attempts >= self.max_retries {
                break res.with_context(|| {
                    format!("maximum number of retries ({}) reached", self.max_retries)
                });
            }
            attempts = attempts.saturating_add(1);

            thread::sleep(delay);

            delay = if self.max_backoff != Duration::new(0, 0) && delay * 2 > self.max_backoff {
                self.max_backoff
            } else {
                delay * 2
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::bail;

    type AttemptResult = Result<u8>;

    #[test]
    fn test_no_retries() {
        let retries = 0;
        let timings = Duration::from_millis(100);
        let driver = Retry::new()
            .initial_backoff(timings)
            .max_backoff(timings)
            .max_retries(retries);

        let final_res: AttemptResult = driver.retry(|attempt| {
            if attempt != 0 {
                panic!("unreachable attempt {attempt}");
            }

            bail!("expected error")
        });
        final_res.unwrap_err();
    }

    #[test]
    fn test_max_retries() {
        let retries = 3;
        let timings = Duration::from_millis(100);
        let driver = Retry::new()
            .initial_backoff(timings)
            .max_backoff(timings)
            .max_retries(retries);

        let final_res = driver.retry(|attempt| {
            if attempt == retries {
                return AttemptResult::Ok(attempt);
            }
            if attempt > retries {
                panic!("unreachable attempt {attempt}");
            }

            bail!("expected error #{}", attempt)
        });
        let total = final_res.unwrap();
        assert_eq!(total, retries);
    }
}
