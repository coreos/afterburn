// Copyright 2026 CoreOS, Inc.
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

use std::collections::HashMap;
use std::time::Duration;

use serde_json::json;

use super::StackitProvider;
use crate::providers::MetadataProvider;

const METADATA_PATH: &str = "/openstack/latest/meta_data.json";
const INSTANCE_ID: &str = "e1c3b0a2-81e6-4af6-8df7-74ca59b02468";
const PUBLIC_KEY_1: &str =
    "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIBjYTHGYkNK7DZ4Gn0NGN1sjFUVapus4GXybEYg/ylcA first-key";
const PUBLIC_KEY_2: &str =
    "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIOPAmN/ccWtKFlCPOwjAMXxrbKBE4cxypTLKgARZF8W1 second-key";

fn setup() -> (mockito::ServerGuard, StackitProvider) {
    let server = mockito::Server::new();
    let mut provider = StackitProvider::try_new().unwrap();
    provider.client = provider.client.max_retries(0).mock_base_url(server.url());
    (server, provider)
}

#[test]
fn test_attributes() {
    let (mut server, provider) = setup();
    // Representative OpenStack metadata, including fields Afterburn does not use.
    let body = json!({
        "uuid": INSTANCE_ID,
        "hostname": "coreos.example.internal",
        "name": "CoreOS server",
        "availability_zone": "eu01-1",
        "project_id": "33730fee-c38b-4312-973b-59eaed7df0f1",
        "public_keys": {"keypair": PUBLIC_KEY_1},
        "meta": {"role": "worker"}
    });
    let mock = server
        .mock("GET", METADATA_PATH)
        .with_body(body.to_string())
        .create();

    assert_eq!(
        provider.attributes().unwrap(),
        HashMap::from([
            ("STACKIT_INSTANCE_ID".to_owned(), INSTANCE_ID.to_owned()),
            (
                "STACKIT_HOSTNAME".to_owned(),
                "coreos.example.internal".to_owned()
            ),
            ("STACKIT_AVAILABILITY_ZONE".to_owned(), "eu01-1".to_owned()),
        ])
    );
    mock.assert();
}

#[test]
fn test_optional_attributes() {
    let (mut server, provider) = setup();
    for body in [
        json!({"uuid": INSTANCE_ID}),
        json!({"uuid": INSTANCE_ID, "hostname": null, "availability_zone": null}),
        json!({"uuid": INSTANCE_ID, "hostname": "", "availability_zone": ""}),
    ] {
        let mock = server
            .mock("GET", METADATA_PATH)
            .with_body(body.to_string())
            .create();
        assert_eq!(
            provider.attributes().unwrap(),
            HashMap::from([("STACKIT_INSTANCE_ID".to_owned(), INSTANCE_ID.to_owned())])
        );
        mock.assert();
        server.reset();
    }
}

#[test]
fn test_hostname() {
    let (mut server, provider) = setup();
    let mock = server
        .mock("GET", METADATA_PATH)
        .with_body(
            json!({
                "uuid": INSTANCE_ID,
                "hostname": "coreos.example.internal",
                "name": "CoreOS server"
            })
            .to_string(),
        )
        .create();
    assert_eq!(
        provider.hostname().unwrap(),
        Some("coreos.example.internal".to_owned())
    );
    mock.assert();
}

#[test]
fn test_missing_hostname() {
    let (mut server, provider) = setup();
    for body in [
        json!({"uuid": INSTANCE_ID, "name": "CoreOS server"}),
        json!({"uuid": INSTANCE_ID, "hostname": null}),
        json!({"uuid": INSTANCE_ID, "hostname": ""}),
    ] {
        let mock = server
            .mock("GET", METADATA_PATH)
            .with_body(body.to_string())
            .create();
        assert_eq!(provider.hostname().unwrap(), None);
        mock.assert();
        server.reset();
    }
}

#[test]
fn test_ssh_keys() {
    let (mut server, provider) = setup();
    for public_keys in [
        json!({"keypair-a": PUBLIC_KEY_1}),
        json!({"keypair-b": PUBLIC_KEY_2, "keypair-a": PUBLIC_KEY_1}),
    ] {
        let mock = server
            .mock("GET", METADATA_PATH)
            .with_body(json!({"uuid": INSTANCE_ID, "public_keys": public_keys}).to_string())
            .create();
        let keys = provider.ssh_keys().unwrap();
        let expected = [PUBLIC_KEY_1, PUBLIC_KEY_2][..public_keys.as_object().unwrap().len()]
            .iter()
            .map(|key| (*key).to_owned())
            .collect::<Vec<_>>();
        assert_eq!(
            keys.iter().map(ToString::to_string).collect::<Vec<_>>(),
            expected
        );
        mock.assert();
        server.reset();
    }
}

#[test]
fn test_no_ssh_keys() {
    let (mut server, provider) = setup();
    for body in [
        json!({"uuid": INSTANCE_ID}),
        json!({"uuid": INSTANCE_ID, "public_keys": null}),
        json!({"uuid": INSTANCE_ID, "public_keys": {}}),
    ] {
        let mock = server
            .mock("GET", METADATA_PATH)
            .with_body(body.to_string())
            .create();
        assert!(provider.ssh_keys().unwrap().is_empty());
        mock.assert();
        server.reset();
    }
}

#[test]
fn test_invalid_ssh_key() {
    let (mut server, provider) = setup();
    let mock = server
        .mock("GET", METADATA_PATH)
        .with_body(
            json!({
                "uuid": INSTANCE_ID,
                "public_keys": {"keypair-a": PUBLIC_KEY_1, "keypair-b": "invalid key"}
            })
            .to_string(),
        )
        .create();
    let err = provider.ssh_keys().unwrap_err();
    assert!(err
        .to_string()
        .contains("failed to parse STACKIT SSH public key"));
    mock.assert();
}

#[test]
fn test_http_errors() {
    let (mut server, provider) = setup();
    for status in [404, 500, 503] {
        let mock = server
            .mock("GET", METADATA_PATH)
            .with_status(status)
            .expect(3)
            .create();
        assert!(provider.attributes().is_err());
        assert!(provider.hostname().is_err());
        assert!(provider.ssh_keys().is_err());
        mock.assert();
        server.reset();
    }
}

#[test]
fn test_invalid_metadata() {
    let (mut server, provider) = setup();
    for body in [
        "",
        "not json",
        "null",
        "[]",
        "{}",
        r#"{"uuid": ""}"#,
        r#"{"uuid": 123}"#,
        r#"{"uuid": "instance", "hostname": []}"#,
        r#"{"uuid": "instance", "public_keys": []}"#,
        r#"{"uuid": "instance", "public_keys": {"keypair": null}}"#,
    ] {
        let mock = server
            .mock("GET", METADATA_PATH)
            .with_body(body)
            .expect(3)
            .create();
        assert!(provider.attributes().is_err(), "accepted {body}");
        assert!(provider.hostname().is_err(), "accepted {body}");
        assert!(provider.ssh_keys().is_err(), "accepted {body}");
        mock.assert();
        server.reset();
    }
}

#[test]
fn test_retry_metadata() {
    let (mut server, mut provider) = setup();
    provider.client = provider
        .client
        .max_retries(1)
        .initial_backoff(Duration::ZERO);
    let unavailable = server.mock("GET", METADATA_PATH).with_status(503).create();
    let available = server
        .mock("GET", METADATA_PATH)
        .with_body(json!({"uuid": INSTANCE_ID, "hostname": "coreos"}).to_string())
        .create();
    assert_eq!(provider.hostname().unwrap(), Some("coreos".to_owned()));
    unavailable.assert();
    available.assert();
}

#[test]
fn test_write_attributes() {
    let (mut server, provider) = setup();
    let mock = server
        .mock("GET", METADATA_PATH)
        .with_body(json!({"uuid": INSTANCE_ID}).to_string())
        .create();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("afterburn");
    provider
        .write_attributes(path.to_str().unwrap().to_owned())
        .unwrap();
    assert_eq!(
        std::fs::read_to_string(path).unwrap(),
        format!("AFTERBURN_STACKIT_INSTANCE_ID={INSTANCE_ID}\n")
    );
    mock.assert();
}
