use std::{collections::HashMap, fs, net::IpAddr, str::FromStr};

use ipnetwork::IpNetwork;
use mockito;
use openssh_keys::Data;
use pnet_base::MacAddr;

use crate::{
    network::{Interface, NetworkRoute},
    providers::MetadataProvider,
};

use super::HetznerProvider;

fn setup() -> (mockito::ServerGuard, HetznerProvider) {
    let server = mockito::Server::new();
    let mut provider = HetznerProvider::try_new().expect("create provider under test");
    provider.client = provider.client.max_retries(0).mock_base_url(server.url());
    (server, provider)
}

#[test]
fn test_attributes() {
    let endpoint_metadata = "/hetzner/v1/metadata";
    let endpoint_networks = "/hetzner/v1/metadata/private-networks";
    let (mut server, provider) = setup();

    let availability_zone = "fsn1-dc14";
    let hostname = "some-hostname";
    let instance_id = "12345678";
    let public_ipv4 = "192.0.2.10";
    let region = "eu-central";

    let body_metadata = format!(
        r#"availability-zone: {availability_zone}
hostname: {hostname}
instance-id: {instance_id}
public-ipv4: {public_ipv4}
region: {region}
local-ipv4: ''
public-keys: []
vendor_data: "blah blah blah""#
    );

    let ip_0 = "10.0.0.2";
    let ip_1 = "10.128.0.2";

    let body_networks = format!(
        r#"- ip: {ip_0}
- ip: {ip_1}"#
    );

    let expected = maplit::hashmap! {
        "HETZNER_AVAILABILITY_ZONE".to_string() => availability_zone.to_string(),
        "HETZNER_HOSTNAME".to_string() => hostname.to_string(),
        "HETZNER_INSTANCE_ID".to_string() => instance_id.to_string(),
        "HETZNER_PUBLIC_IPV4".to_string() => public_ipv4.to_string(),
        "HETZNER_REGION".to_string() => region.to_string(),
        "HETZNER_PRIVATE_IPV4_0".to_string() => ip_0.to_string(),
        "HETZNER_PRIVATE_IPV4_1".to_string() => ip_1.to_string(),
    };

    // Fail on not found
    provider.attributes().unwrap_err();

    // Fail on internal server errors (metadata endpoint)
    let mock_metadata = server
        .mock("GET", endpoint_metadata)
        .with_status(503)
        .create();
    provider.attributes().unwrap_err();
    mock_metadata.assert();

    let mock_metadata = server
        .mock("GET", endpoint_metadata)
        .with_status(200)
        .with_body(body_metadata)
        .expect(2) // Once for the private-networks error test and once to compare the result
        .create();

    // Fail on internal server errors (networks endpoint)
    let mock_networks = server
        .mock("GET", endpoint_networks)
        .with_status(503)
        .create();
    provider.attributes().unwrap_err();
    mock_networks.assert();

    // Fetch metadata
    let mock_networks = server
        .mock("GET", endpoint_networks)
        .with_status(200)
        .with_body(body_networks)
        .create();

    let actual = provider.attributes().unwrap();
    mock_metadata.assert();
    mock_networks.assert();
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

#[test]
fn test_networks() {
    let endpoint_network_config = "/hetzner/v1/metadata/network-config";
    let (mut server, provider) = setup();

    let name = "interface_name";
    let mac_addr = "11:11:11:11:11:11";
    let ipv6_network = "2a01:4f9:c013:f7c7::1/64";
    let gateway = "fa11::1";
    let dns_nameserver_1 = "1a01:0ff:ff00::add:1";
    let dns_nameserver_2 = "1a01:0ff:ff00::add:2";

    let body_network_config = format!(
        r#"version: 1
config:
  - type: physical
    name: {name}
    mac_address: {mac_addr}
    subnets:
      - type: dhcp
        ipv4: true
      - type: static
        address: {ipv6_network}
        ipv6: true
        gateway: {gateway}
        dns_nameservers:
          - {dns_nameserver_1}
          - {dns_nameserver_2}"#
    );

    let expected = vec![Interface {
        name: Some(name.into()),
        mac_address: Some(MacAddr::from_str(mac_addr).unwrap()),
        nameservers: vec![
            IpAddr::from_str(dns_nameserver_1).unwrap(),
            IpAddr::from_str(dns_nameserver_2).unwrap(),
        ],
        ip_addresses: vec![IpNetwork::from_str(ipv6_network).unwrap()],
        dhcp: Some(crate::network::DhcpSetting::V4),
        routes: vec![NetworkRoute {
            destination: IpNetwork::from_str("::/0").unwrap(),
            gateway: IpAddr::from_str(gateway).unwrap(),
        }],
        unmanaged: false,
        priority: 20,
        bond: None,
        required_for_online: None,
        path: None,
    }];

    assert!(provider.networks().is_err(), "Should fail on not found");

    let mock_metadata = server
        .mock("GET", endpoint_network_config)
        .with_status(503)
        .create();
    assert!(
        provider.networks().is_err(),
        "Should fail on internal server error"
    );
    mock_metadata.assert();

    let mock_metadata = server
        .mock("GET", endpoint_network_config)
        .with_status(200)
        .with_body(body_network_config)
        .expect(1)
        .create();

    let actual = provider.networks().unwrap();
    mock_metadata.assert();

    assert_eq!(actual, expected);
}

#[test]
fn test_netplan_config() {
    let endpoint_network_config = "/hetzner/v1/metadata/network-config";
    let (mut server, provider) = setup();

    let body = fs::read("./tests/fixtures/hetzner/network-config.yaml")
        .expect("Unable to read network-config fixture");
    let expected = String::from_utf8(
        fs::read("./tests/fixtures/hetzner/netplan-config.yaml")
            .expect("Unable to read network-config fixture"),
    )
    .unwrap();

    let mock_metadata = server
        .mock("GET", endpoint_network_config)
        .with_status(200)
        .with_body(body)
        .expect(1)
        .create();

    let actual = provider.netplan_config().unwrap().unwrap();
    mock_metadata.assert();

    assert_eq!(actual, expected);
}
