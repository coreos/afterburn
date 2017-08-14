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

//! Providers
//!
//! These are the providers which coreos-metadata knows how to retrieve metadata
//! from. Internally, they handle the ins and outs of each providers metadata
//! services, and externally, they provide a function to fetch that metadata in
//! a regular format.
//!
//! To add a provider, put a `pub mod provider;` line in this file, export a
//! function to fetch the metadata, and then add a match line in the top-level
//! `fetch_metadata()` function in metadata.rs.

pub mod azure;
pub mod digitalocean;
pub mod ec2;
pub mod gce;
pub mod openstack;
pub mod packet;
pub mod vagrant_virtualbox;
