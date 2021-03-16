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

mod cli;
mod initrd;
mod metadata;
mod network;
mod providers;
mod retry;
mod util;

use anyhow::{Context, Result};
use slog::{slog_o, Drain};
use slog_scope::debug;
use std::env;

fn main() -> Result<()> {
    // Setup logging.
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let log = slog::Logger::root(drain, slog_o!());
    let _guard = slog_scope::set_global_logger(log);
    debug!("logging initialized");

    // Parse command-line arguments.
    let cli_cmd = cli::parse_args(env::args()).context("failed to parse command-line arguments")?;
    debug!("command-line arguments parsed");

    // Run core logic.
    cli_cmd.run().context("failed to run")?;
    debug!("all tasks completed");

    Ok(())
}
