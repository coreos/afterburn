use crate::providers::gcp;
use crate::providers::MetadataProvider;
use mockito;

#[test]
fn basic_hostname() {
    let ep = "/computeMetadata/v1/instance/hostname";
    let hostname = "test-hostname";

    let mut server = mockito::Server::new();
    let mut provider = gcp::GcpProvider::try_new().unwrap();
    provider.client = provider.client.max_retries(0).mock_base_url(server.url());

    server.mock("GET", ep).with_status(503).create();
    provider.hostname().unwrap_err();

    server
        .mock("GET", ep)
        .with_status(200)
        .with_body(hostname)
        .create();
    let v = provider.hostname().unwrap();
    assert_eq!(v, Some(hostname.to_string()));

    server.mock("GET", ep).with_status(404).create();
    let v = provider.hostname().unwrap();
    assert_eq!(v, None);

    server.reset();
    provider.hostname().unwrap_err();
}

#[test]
fn basic_attributes() {
    let hostname = "test-hostname";
    let ip_external = "test-ip-external";
    let ip_local = "test-ip-local";
    let machine_type = "test-machine-type";

    let endpoints = maplit::btreemap! {
        "/computeMetadata/v1/instance/hostname" => hostname,
        "/computeMetadata/v1/instance/network-interfaces/0/access-configs/0/external-ip" => ip_external,
        "/computeMetadata/v1/instance/network-interfaces/0/ip" => ip_local,
        "/computeMetadata/v1/instance/machine-type" => machine_type,
    };
    let mut server = mockito::Server::new();
    for (endpoint, body) in endpoints {
        server
            .mock("GET", endpoint)
            .with_status(200)
            .with_body(body)
            .create();
    }

    let attributes = maplit::hashmap! {
        "GCP_HOSTNAME".to_string() => hostname.to_string(),
        "GCP_IP_EXTERNAL_0".to_string() => ip_external.to_string(),
        "GCP_IP_LOCAL_0".to_string() => ip_local.to_string(),
        "GCP_MACHINE_TYPE".to_string() => machine_type.to_string(),
    };

    let client = crate::retry::Client::try_new()
        .unwrap()
        .max_retries(0)
        .return_on_404(true)
        .mock_base_url(server.url());
    let provider = gcp::GcpProvider { client };

    let v = provider.attributes().unwrap();
    assert_eq!(v, attributes);

    server.reset();
    provider.attributes().unwrap_err();
}
