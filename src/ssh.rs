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

//! ssh manipulates the authorized_keys directory and file
//! TODO(sdemos):
//! right now this doesn't do the file locking expected by the other tools which manipulate this directory
//! for testing purposes, I'll leave it that way for now. If this ever gets used in real life, fix this somehow.
//! the real fix is https://bugzilla.mindrot.org/show_bug.cgi?id=2755 but if that doesn't make it in, the
//! second best fix is to rewrite `update-ssh-keys` in rust as a library/binary combo

use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::io::prelude::*;
use users::os::unix::UserExt;
use users;

use errors::*;

pub fn create_authorized_keys_dir(user: users::User) -> Result<PathBuf> {
    // construct the path to the authorized keys directory
    let ssh_dir = user.home_dir().join(".ssh");
    let authorized_keys_dir = ssh_dir.join("authorized_keys.d");
    // check if the authorized keys directory exists
    if authorized_keys_dir.is_dir() {
        // if it does, just return
        return Ok(authorized_keys_dir);
    }
    // if it doesn't, create it
    fs::create_dir_all(&authorized_keys_dir)
        .chain_err(|| format!("failed to create directory {:?}", authorized_keys_dir))?;
    // check if there is an authorized keys file
    let authorized_keys_file = ssh_dir.join("authorized_keys");
    if authorized_keys_file.is_file() {
        // if there is, copy it into the authorized keys directory
        let preserved_keys_file = authorized_keys_dir.join("orig_authorzied_keys");
        fs::copy(&authorized_keys_file, preserved_keys_file)
            .chain_err(|| format!("failed to copy old authorzied keys file"))?;
    }
    // then we are done
    Ok(authorized_keys_dir)
}

pub fn sync_authorized_keys(authorized_keys_dir: PathBuf) -> Result<()> {
    let ssh_dir = authorized_keys_dir.parent()
        .ok_or(format!("could not get parent directory of {:?}", authorized_keys_dir))?;
    let mut authorized_keys_file = File::create(ssh_dir.join("authorized_keys"))
        .chain_err(|| format!("failed to create file {:?}", ssh_dir.join("authorized_keys")))?;
    flatten_dir(&mut authorized_keys_file, &authorized_keys_dir)
}

fn flatten_dir(mut file: &mut File, dir: &PathBuf) -> Result<()> {
    let dir_contents = fs::read_dir(&dir)
        .chain_err(|| format!("failed to read from directory {:?}", dir))?;
    for entry in dir_contents {
        let entry = entry.chain_err(|| format!("failed to read entry in directory {:?}", dir))?;
        let path = entry.path();
        if path.is_dir() {
            // if it's a directory, recurse into it
            flatten_dir(&mut file, &path)?;
        } else {
            let mut from = File::open(&path)
                .chain_err(|| format!("failed to open file {:?}", path))?;
            let mut contents = String::new();
            from.read_to_string(&mut contents)
                .chain_err(|| format!("failed to read file {:?}", path))?;
            write!(&mut file, "{}\n", contents)
                .chain_err(|| format!("failed to write to file {:?}", file))?;
        }
    }
    Ok(())
}
