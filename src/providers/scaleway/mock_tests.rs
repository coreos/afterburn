use mockito::{self, Matcher};

use crate::providers::scaleway::ScalewayProvider;
use crate::providers::MetadataProvider;

#[test]
fn test_attributes() {
    let metadata = r#"{
        "commercial_type": "GP1-M",
        "hostname": "frontend-0",
        "id": "11111111-1111-1111-1111-111111111111",
        "ipv6": {
            "address": "2001:db8::1"
        },
        "location": {
            "zone_id": "par1"
        },
        "private_ip": "10.0.0.2",
        "public_ip": {
            "address": "93.184.216.34"
        },
        "ssh_public_keys": []
    }"#;

    let want = maplit::hashmap! {
        "SCALEWAY_INSTANCE_ID".to_string() => "11111111-1111-1111-1111-111111111111".to_string(),
        "SCALEWAY_INSTANCE_TYPE".to_string() => "GP1-M".to_string(),
        "SCALEWAY_HOSTNAME".to_string() => "frontend-0".to_string(),
        "SCALEWAY_IPV4_PRIVATE".to_string() => "10.0.0.2".to_string(),
        "SCALEWAY_IPV4_PUBLIC".to_string() => "93.184.216.34".to_string(),
        "SCALEWAY_IPV6_PUBLIC".to_string() => "2001:db8::1".to_string(),
        "SCALEWAY_ZONE_ID".to_string() => "par1".to_string(),
    };

    let mut server = mockito::Server::new();
    server
        .mock("GET", "/conf?format=json")
        .with_status(200)
        .with_body(metadata)
        .create();

    let mut provider = ScalewayProvider::try_new().unwrap();
    provider.client = provider.client.max_retries(0).mock_base_url(server.url());
    let got = provider.attributes().unwrap();

    assert_eq!(got, want);

    server.reset();
}

#[test]
fn test_boot_checkin() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("PATCH", "/state")
        .match_header(
            "content-type",
            Matcher::Regex("application/json".to_string()),
        )
        .match_body(r#"{"state_detail":"booted"}"#)
        .with_status(200)
        .create();

    let mut provider = ScalewayProvider::try_new().unwrap();
    provider.client = provider.client.max_retries(0).mock_base_url(server.url());

    provider.boot_checkin().unwrap();
    mock.assert();

    server.reset();
}
