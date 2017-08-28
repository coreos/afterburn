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

//! utility functions

pub fn key_lookup(delim: char, key: &str, contents: &str) -> Option<String> {
    for l in contents.clone().lines() {
        match l.find(delim) {
            Some(index) => {
                let l = l.to_owned();
                let (k, val) = l.split_at(index+1);
                if k != format!("{}{}", key, delim) {
                    continue
                }
                return Some(val.to_owned())
            }
            None => continue,
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn key_lookup_test() {
        let tests = vec![
            ('=', "DNS", "foo=bar\nbaz=bax\nDNS=8.8.8.8\n", Some("8.8.8.8".to_owned())),
            (':', "foo", "foo:bar", Some("bar".to_owned())),
            (' ', "foo", "", None),
            (':', "bar", "foo:bar\nbaz:bar", None),
            (' ', "baz", "foo foo\nbaz bar", Some("bar".to_owned())),
            (' ', "foo", "\n\n\n\n\n\n\n \n", None),
        ];
        for (delim, key, contents, expected_val) in tests {
            let val = key_lookup(delim, key, contents);
            assert_eq!(val, expected_val);
        }
    }
}
