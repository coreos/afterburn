use mockito;
use openssh_keys::Data;

use crate::providers::MetadataProvider;

use super::HetznerProvider;

fn setup() -> (mockito::ServerGuard, HetznerProvider) {
    let server = mockito::Server::new();
    let mut provider = HetznerProvider::try_new().expect("create provider under test");
    provider.client = provider.client.max_retries(0).mock_base_url(server.url());
    (server, provider)
}

#[test]
fn test_attributes() {
    let endpoint = "/hetzner/v1/metadata";
    let (mut server, provider) = setup();

    let availability_zone = "fsn1-dc14";
    let hostname = "some-hostname";
    let instance_id = "12345678";
    let public_ipv4 = "192.0.2.10";
    let region = "eu-central";

    let body = format!(
        r#"availability-zone: {availability_zone}
hostname: {hostname}
instance-id: {instance_id}
public-ipv4: {public_ipv4}
region: {region}
local-ipv4: ''
public-keys: []
vendor_data: "blah blah blah""#
    );

    let expected = maplit::hashmap! {
        "HETZNER_AVAILABILITY_ZONE".to_string() => availability_zone.to_string(),
        "HETZNER_HOSTNAME".to_string() => hostname.to_string(),
        "HETZNER_INSTANCE_ID".to_string() => instance_id.to_string(),
        "HETZNER_PUBLIC_IPV4".to_string() => public_ipv4.to_string(),
        "HETZNER_REGION".to_string() => region.to_string(),
    };

    // Fail on not found
    provider.attributes().unwrap_err();

    // Fail on internal server errors
    let mock = server.mock("GET", endpoint).with_status(503).create();
    provider.attributes().unwrap_err();
    mock.assert();

    // Fetch metadata
    let mock = server
        .mock("GET", endpoint)
        .with_status(200)
        .with_body(body)
        .create();
    let actual = provider.attributes().unwrap();
    mock.assert();
    assert_eq!(actual, expected);
}

#[test]
fn test_hostname() {
    let endpoint = "/hetzner/v1/metadata/hostname";
    let hostname = "some-hostname";

    let (mut server, provider) = setup();

    // Fail on not found
    provider.hostname().unwrap_err();

    // Fail on internal server errors
    server.mock("GET", endpoint).with_status(503).create();
    provider.hostname().unwrap_err();

    // Return hostname on success
    server
        .mock("GET", endpoint)
        .with_status(200)
        .with_body(hostname)
        .create();
    assert_eq!(provider.hostname().unwrap(), Some(hostname.to_string()));

    // Return `None` if response is empty
    server
        .mock("GET", endpoint)
        .with_status(200)
        .with_body("")
        .create();
    assert_eq!(provider.hostname().unwrap(), None);
}

#[test]
fn test_pubkeys() {
    let endpoint = "/hetzner/v1/metadata/public-keys";
    let pubkey1 =
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIBjYTHGYkNK7DZ4Gn0NGN1sjFUVapus4GXybEYg/ylcA some-key";
    let pubkey2 =
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIOPAmN/ccWtKFlCPOwjAMXxrbKBE4cxypTLKgARZF8W1 some-other-key";

    let (mut server, provider) = setup();

    // Fail on not found
    provider.ssh_keys().unwrap_err();

    // Fail on internal server errors
    server.mock("GET", endpoint).with_status(503).create();
    provider.ssh_keys().unwrap_err();

    // No keys
    server
        .mock("GET", endpoint)
        .with_status(200)
        .with_body("[]")
        .create();
    let keys = provider.ssh_keys().unwrap();
    assert!(keys.is_empty());

    // Fetch single key
    server
        .mock("GET", endpoint)
        .with_status(200)
        .with_body(serde_json::to_string(&[pubkey1]).unwrap())
        .create();
    let keys = provider.ssh_keys().unwrap();
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0].comment, Some("some-key".to_string()));
    assert_eq!(
        keys[0].data,
        Data::Ed25519 {
            key: vec![
                24, 216, 76, 113, 152, 144, 210, 187, 13, 158, 6, 159, 67, 70, 55, 91, 35, 21, 69,
                90, 166, 235, 56, 25, 124, 155, 17, 136, 63, 202, 87, 0
            ]
        }
    );
    assert_eq!(keys[0].options, None);

    // Fetch multiple keys
    server
        .mock("GET", endpoint)
        .with_status(200)
        .with_body(serde_json::to_string(&[pubkey1, pubkey2]).unwrap())
        .create();
    let keys = provider.ssh_keys().unwrap();
    assert_eq!(keys.len(), 2);
}
