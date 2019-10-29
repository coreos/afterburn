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
    let client = crate::retry::Client::try_new().unwrap().max_attempts(1);
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
            "addresses": [],
            "bonding": { "mode": 0 }
        },
        "phone_home_url": "test-url"
    }"#;

    let attributes = maplit::hashmap! {
        "PACKET_HOSTNAME".to_string() => "test-hostname".to_string(),
        "PACKET_PHONE_HOME_URL".to_string() => "test-url".to_string(),
        "PACKET_PLAN".to_string() => "test-plan".to_string(),
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
    let client = crate::retry::Client::try_new().unwrap().max_attempts(1);
    packet::PacketProvider::fetch_content(Some(client)).unwrap_err();
}
