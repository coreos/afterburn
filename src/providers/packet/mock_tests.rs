use crate::providers::{packet, MetadataProvider};
use mockito::{self, Matcher};

#[test]
fn test_boot_checkin() {
    let data = packet::PacketData {
        id: String::new(),
        hostname: String::new(),
        iqn: String::new(),
        plan: String::new(),
        facility: String::new(),
        tags: vec![],
        ssh_keys: vec![],
        network: packet::PacketNetworkInfo {
            interfaces: vec![],
            addresses: vec![],
            bonding: packet::PacketBondingMode { mode: 0 },
        },
        error: None,
        phone_home_url: mockito::server_url(),
    };
    let provider = packet::PacketProvider { data };

    let mock = mockito::mock("POST", "/")
        .match_header(
            "content-type",
            Matcher::Regex("application/json".to_string()),
        )
        .match_body("")
        .with_status(200)
        .create();

    let r = provider.boot_checkin();
    mock.assert();
    r.unwrap();

    mockito::reset();

    // Check error logic, but fail fast without re-trying.
    let client = crate::retry::Client::try_new().unwrap().max_retries(0);
    packet::PacketProvider::fetch_content(Some(client)).unwrap_err();
}

#[test]
fn test_packet_attributes() {
    let metadata = r#"{
        "id": "test-id",
        "hostname": "test-hostname",
        "iqn": "test-iqn",
        "plan": "test-plan",
        "facility": "test-facility",
        "tags": [],
        "ssh_keys": [],
        "network": {
            "interfaces": [],
            "addresses": [
              {
                "id": "fde74ec8-bc24-43ca-a852-875bd6e10bee",
                "address_family": 4,
                "netmask": "255.255.255.254",
                "public": true,
                "management": true,
                "address": "147.0.0.1",
                "gateway": "147.0.0.0"
              },
              {
                "id": "3ae01206-b03a-4353-b04d-94747d36457e",
                "address_family": 6,
                "netmask": "ffff:ffff:ffff:ffff:ffff:ffff:ffff:fffe",
                "public": true,
                "management": true,
                "address": "2604:1380::1",
                "gateway": "2604:1380::0"
              },
              {
                "id": "bec04697-31fd-4a99-b1a9-cd1c7b88c810",
                "address_family": 4,
                "netmask": "255.255.255.254",
                "public": false,
                "management": true,
                "address": "10.0.0.1",
                "gateway": "10.0.0.0"
              },
              {
                "id": "38475859-389f-48ac-a855-f3ebdb82c565",
                "address_family": 6,
                "netmask": "ffff:ffff:ffff:ffff:ffff:ffff:ffff:fffe",
                "public": false,
                "management": true,
                "address": "fd00::1",
                "gateway": "fd00::0"
              }
            ],
            "bonding": { "mode": 0 }
        },
        "phone_home_url": "test-url"
    }"#;

    let attributes = maplit::hashmap! {
        "PACKET_HOSTNAME".to_string() => "test-hostname".to_string(),
        "PACKET_PHONE_HOME_URL".to_string() => "test-url".to_string(),
        "PACKET_PLAN".to_string() => "test-plan".to_string(),
        "PACKET_IPV4_PUBLIC_0".to_string() => "147.0.0.1".to_string(),
        "PACKET_IPV4_PRIVATE_0".to_string() => "10.0.0.1".to_string(),
        "PACKET_IPV6_PUBLIC_0".to_string() => "2604:1380::1".to_string(),
        "PACKET_IPV6_PRIVATE_0".to_string() => "fd00::1".to_string(),
    };

    let _m = mockito::mock("GET", "/metadata")
        .with_status(200)
        .with_body(metadata)
        .create();

    let provider = packet::PacketProvider::try_new().unwrap();
    let v = provider.attributes().unwrap();

    assert_eq!(v, attributes);

    mockito::reset();

    // Check error logic, but fail fast without re-trying.
    let client = crate::retry::Client::try_new().unwrap().max_retries(0);
    packet::PacketProvider::fetch_content(Some(client)).unwrap_err();
}
