use mockito;
use crate::providers::gcp;
use crate::providers::MetadataProvider;

#[test]
fn basic_hostname() {
    let ep = "/instance/hostname";
    let hostname = "test-hostname";

    let mut provider = gcp::GcpProvider::try_new().unwrap();
    provider.client = provider.client.max_attempts(1);

    provider.hostname().unwrap_err();

    let _m = mockito::mock("GET", ep).with_status(503).create();
    provider.hostname().unwrap_err();

    let _m = mockito::mock("GET", ep)
        .with_status(200)
        .with_body(hostname)
        .create();
    let v = provider.hostname().unwrap();
    assert_eq!(v, Some(hostname.to_string()));

    let _m = mockito::mock("GET", ep).with_status(404).create();
    let v = provider.hostname().unwrap();
    assert_eq!(v, None);
}
