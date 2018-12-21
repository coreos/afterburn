use mockito::{self, Matcher};
use providers::{packet, MetadataProvider};

#[test]
fn test_boot_checkin() {
    let ep = "/events";
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

    let json_body = json!({
        "state": "succeeded",
        "code": 1042,
        "message": "coreos-metadata: boot check-in",
    });
    let mock = mockito::mock("POST", ep)
        .match_header("content-type", Matcher::Regex("text/json.*".to_string()))
        .match_body(Matcher::Json(json_body))
        .with_status(200)
        .create();

    let r = provider.boot_checkin();
    mock.assert();
    r.unwrap();
}
