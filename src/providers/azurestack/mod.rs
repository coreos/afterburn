// Copyright 2020 Red Hat, Inc.
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

// This is a stub provider. Right now the AzureStack provider does
// asbolutely nothing interesting.

//! azurestack/azurestack metadata fetcher
use crate::providers::MetadataProvider;

use slog_scope::debug;

#[derive(Clone, Copy, Debug)]
pub struct AzureStack;

impl AzureStack {
    pub fn new() -> Self {
        debug!("azure stack provider is a noop stub");
        Self
    }
}

impl MetadataProvider for AzureStack{}