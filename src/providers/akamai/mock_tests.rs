use crate::providers::akamai::{AkamaiProvider, TOKEN_TTL};
use crate::providers::MetadataProvider;
use mockito::{self};

#[test]
fn test_attributes() {
    let mut server = mockito::Server::new();
    let token = "deadbeefcafebabe";

    // Mock the PUT /v1/token endpoint.
    let put_v1_token = server
        .mock("PUT", "/v1/token")
        .match_header("metadata-token-expiry-seconds", TOKEN_TTL)
        .with_body(token)
        .expect_at_least(1)
        .create();

    // Mock the GET /v1/instance endpoint.
    let instance_metadata = r#"{
        "id": 12345678,
        "label": "my-linode",
        "region": "us-ord",
        "type": "g6-nanode-1",
        "specs": {
            "vcpus": 1,
            "memory": 1024,
            "gpus": 0,
            "transfer": 1000,
            "disk": 25600
        },
        "backups": {
            "enabled": false,
            "status": null
        },
        "host_uuid": "a631b16d14534d84e2830da16d1b28e1d08d24df",
        "tags": ["foo", "bar", "baz"]
    }"#;

    let get_v1_instance = server
        .mock("GET", "/v1/instance")
        .match_header("Accept", "application/json")
        .match_header("metadata-token", token)
        .with_body(instance_metadata)
        .create();

    // Mock the /v1/network endpoint.
    let network_metadata = r#"{
        "interfaces": [
          {
            "id": 12345678,
            "purpose": "public",
            "label": null,
            "ipam_address": null
          }
        ],
        "ipv4": {
            "public": [
                "1.2.3.4/32"
            ],
            "private": [
                "192.168.1.1/32"
            ],
            "shared": []
        },
        "ipv6": {
            "slaac": "2600:3c06::f03c:94ff:fecb:c10b/128",
            "ranges": [],
            "link_local": "fe80::f03c:94ff:fecb:c10b/128",
            "shared_ranges": []
        }
    }"#;

    let get_v1_network = server
        .mock("GET", "/v1/network")
        .match_header("Accept", "application/json")
        .match_header("metadata-token", token)
        .with_body(network_metadata)
        .create();

    let provider = AkamaiProvider::with_base_url(server.url()).unwrap();
    let attrs = provider.attributes();

    // Assert that our endpoints were called.
    put_v1_token.assert();
    get_v1_instance.assert();
    get_v1_network.assert();

    let actual = attrs.unwrap();
    let expected = maplit::hashmap! {
        "AKAMAI_INSTANCE_ID".to_string() => "12345678".to_string(),
        "AKAMAI_INSTANCE_HOST_UUID".to_string() => "a631b16d14534d84e2830da16d1b28e1d08d24df".to_string(),
        "AKAMAI_INSTANCE_LABEL".to_string() => "my-linode".to_string(),
        "AKAMAI_INSTANCE_REGION".to_string() => "us-ord".to_string(),
        "AKAMAI_INSTANCE_TYPE".to_string() => "g6-nanode-1".to_string(),
        "AKAMAI_INSTANCE_TAGS".to_string() => "foo:bar:baz".to_string(),
        "AKAMAI_PUBLIC_IPV4_0".to_string() => "1.2.3.4/32".to_string(),
        "AKAMAI_PRIVATE_IPV4_0".to_string() => "192.168.1.1/32".to_string(),
        "AKAMAI_IPV6_SLAAC".to_string() => "2600:3c06::f03c:94ff:fecb:c10b/128".to_string(),
        "AKAMAI_IPV6_LINK_LOCAL".to_string() => "fe80::f03c:94ff:fecb:c10b/128".to_string(),
    };
    assert_eq!(expected, actual);

    server.reset();
}
