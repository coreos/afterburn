use crate::providers::gcp;
use crate::providers::MetadataProvider;
use std::collections::HashMap;
use mockito;

#[cfg(not(feature = "cl-legacy"))]
static ENV_PREFIX: &str = "GCP";
#[cfg(feature = "cl-legacy")]
static ENV_PREFIX: &str = "GCE";

#[test]
fn basic_hostname() {
    let ep = "/instance/hostname";
    let hostname = "test-hostname";

    let mut provider = gcp::GcpProvider::try_new().unwrap();
    provider.client = provider.client.max_attempts(1);

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

    mockito::reset();
    provider.hostname().unwrap_err();
}

#[test]
fn basic_attributes() {
    let ep_hostname = "/instance/hostname";
    let hostname = "test-hostname";

    let ep_ip_external = "/instance/network-interfaces/0/access-configs/0/external-ip";
    let ip_external = "test-ip-external";

    let ep_ip_local = "/instance/network-interfaces/0/ip";
    let ip_local = "test-ip-local";

    let ep_machine_type = "/instance/machine-type";
    let machine_type = "test-machine-type";

    let mut attributes:HashMap<String, String> = HashMap::new();
    attributes.insert(format!("{}_HOSTNAME", ENV_PREFIX), String::from(hostname));
    attributes.insert(format!("{}_IP_EXTERNAL_0", ENV_PREFIX), String::from(ip_external));
    attributes.insert(format!("{}_IP_LOCAL_0", ENV_PREFIX), String::from(ip_local));
    attributes.insert(format!("{}_MACHINE_TYPE", ENV_PREFIX), String::from(machine_type));

    let mut provider = gcp::GcpProvider::try_new().unwrap();
    provider.client = provider.client.max_attempts(1);

    let _m = mockito::mock("GET", ep_hostname)
        .with_status(200)
        .with_body(hostname)
        .create();
    let _m = mockito::mock("GET", ep_ip_external)
        .with_status(200)
        .with_body(ip_external)
        .create();
    let _m = mockito::mock("GET", ep_ip_local)
        .with_status(200)
        .with_body(ip_local)
        .create();
    let _m = mockito::mock("GET", ep_machine_type)
        .with_status(200)
        .with_body(machine_type)
        .create();

    let v = provider.attributes().unwrap();

    assert_eq!(v, attributes);
}
