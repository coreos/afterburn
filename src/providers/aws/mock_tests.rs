use crate::errors::*;
use crate::providers::aws;
use mockito;
use std::collections::HashMap;
use crate::providers::MetadataProvider;

#[cfg(not(feature = "cl-legacy"))]
static ENV_PREFIX: &str = "AWS";
#[cfg(feature = "cl-legacy")]
static ENV_PREFIX: &str = "EC2";

#[test]
fn test_aws_basic() {
    let ep = "/meta-data/public-keys";
    let client = crate::retry::Client::try_new()
        .chain_err(|| "failed to create http client")
        .unwrap()
        .max_attempts(1)
        .return_on_404(true);
    let provider = aws::AwsProvider { client };

    provider.fetch_ssh_keys().unwrap_err();

    let _m = mockito::mock("GET", ep).with_status(503).create();
    provider.fetch_ssh_keys().unwrap_err();

    let _m = mockito::mock("GET", ep).with_status(200).create();
    let v = provider.fetch_ssh_keys().unwrap();
    assert_eq!(v.len(), 0);

    let _m = mockito::mock("GET", ep).with_status(404).create();
    let v = provider.fetch_ssh_keys().unwrap();
    assert_eq!(v.len(), 0);
}

#[test]
fn test_attributes_fetching() {
    let ep_instance_id = "/meta-data/instance-id";
    let instance_id = "test-instance-id";

    let ep_instance_type = "/meta-data/instance-type";
    let instance_type = "test-instance-type";

    let ep_ipv4_local = "/meta-data/local-ipv4";
    let ipv4_local = "test-ipv4-local";

    let ep_ipv4_public = "/meta-data/public-ipv4";
    let ipv4_public = "test-ipv4-public";

    let ep_availability_zone = "/meta-data/placement/availability-zone";
    let availability_zone = "test-availability-zone";

    let ep_hostname = "/meta-data/hostname";
    let hostname = "test-hostname";

    let ep_public_hostname = "/meta-data/public-hostname";
    let public_hostname = "test-public-hostname";

    let ep_region = "/dynamic/instance-identity/document";
    let instance_id_doc = "{\"region\": \"test-region\"}";
    let region = "test-region";

    let mut attributes:HashMap<String, String> = HashMap::new();
    attributes.insert(format!("{}_INSTANCE_ID", ENV_PREFIX), String::from(instance_id));
    attributes.insert(format!("{}_INSTANCE_TYPE", ENV_PREFIX), String::from(instance_type));
    attributes.insert(format!("{}_IPV4_LOCAL", ENV_PREFIX), String::from(ipv4_local));
    attributes.insert(format!("{}_IPV4_PUBLIC", ENV_PREFIX), String::from(ipv4_public));
    attributes.insert(format!("{}_AVAILABILITY_ZONE", ENV_PREFIX), String::from(availability_zone));
    attributes.insert(format!("{}_HOSTNAME", ENV_PREFIX), String::from(hostname));
    attributes.insert(format!("{}_PUBLIC_HOSTNAME", ENV_PREFIX), String::from(public_hostname));
    attributes.insert(format!("{}_REGION", ENV_PREFIX), String::from(region));

    let client = crate::retry::Client::try_new()
        .chain_err(|| "failed to create http client")
        .unwrap()
        .max_attempts(1)
        .return_on_404(true);
    let provider = aws::AwsProvider { client };

    let _m = mockito::mock("GET", ep_instance_id)
        .with_status(200)
        .with_body(instance_id)
        .create();
    let _m = mockito::mock("GET", ep_instance_type)
        .with_status(200)
        .with_body(instance_type)
        .create();
    let _m = mockito::mock("GET", ep_ipv4_local)
        .with_status(200)
        .with_body(ipv4_local)
        .create();
    let _m = mockito::mock("GET", ep_ipv4_public)
        .with_status(200)
        .with_body(ipv4_public)
        .create();
    let _m = mockito::mock("GET", ep_availability_zone)
        .with_status(200)
        .with_body(availability_zone)
        .create();
    let _m = mockito::mock("GET", ep_hostname)
        .with_status(200)
        .with_body(hostname)
        .create();
    let _m = mockito::mock("GET", ep_public_hostname)
        .with_status(200)
        .with_body(public_hostname)
        .create();
    let _m = mockito::mock("GET", ep_region)
        .with_status(200)
        .with_body(instance_id_doc)
        .create();

    let v = provider.attributes().unwrap();

    assert_eq!(v, attributes);
}