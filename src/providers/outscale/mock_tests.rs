use std::collections::{BTreeMap, HashMap};

use anyhow::Context;

use crate::providers::outscale;
use crate::providers::MetadataProvider;

#[test]
fn basic_hostname() {
    let endpoint = "/latest/meta-data/hostname";
    let hostname = "test-hostname";

    let mut server = mockito::Server::new();
    let client = crate::retry::Client::try_new()
        .context("failed to create http client")
        .unwrap()
        .max_retries(0)
        .return_on_404(true)
        .mock_base_url(server.url());

    let provider = outscale::OutscaleProvider { client };

    server.mock("GET", endpoint).with_status(503).create();
    provider.hostname().unwrap_err();

    server
        .mock("GET", endpoint)
        .with_status(200)
        .with_body(hostname)
        .create();

    assert_eq!(provider.hostname().unwrap(), Some(hostname.to_string()));

    server.reset();
    provider.hostname().unwrap_err();
}

#[test]
fn basic_pubkeys() {
    let key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQC+bqdi18/+JfjrqmOEtVKyCU0bsIc6tBqqU7p9mesJkALocLddDU6d97w2zwERhzaqReDyg4msvQQohgtncb4afKKWQjCCCWlcwtP0nAeg9GFtUfmLeYcP2KAjxblabncluuAnvMHyBixKAjr5eWD4B1HjOmpMRmycwmy85QhGTYhF+AkiHGCPPUDrVy2cIvrPSDXEEa7bz5aQUime0Eold56n3O7E5BJuAozf+oeiWCERRRt9ATlLkMvwVItzBHN25YoMOd0KfgYMtBVAw86TErYFx4Tu98blYNUQTthf9VxcU8xy0rFacXmuS7LHbp+CKDY0X5dNHuhqz0wFto4J test-comment";

    let mut server = mockito::Server::new();
    let client = crate::retry::Client::try_new()
        .context("failed to create http client")
        .unwrap()
        .max_retries(0)
        .return_on_404(true)
        .mock_base_url(server.url());

    let provider = outscale::OutscaleProvider { client };

    server
        .mock("GET", "/latest/meta-data/public-keys/0/openssh-key")
        .with_status(200)
        .with_body(key)
        .create();

    let keys = provider.ssh_keys().unwrap();

    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0].comment, Some("test-comment".to_string()));

    server.reset();
    provider.ssh_keys().unwrap_err();
}

#[test]
fn basic_attributes() {
    let endpoints = BTreeMap::from([
        ("/latest/meta-data/ami-id", "test-ami-id"),
        ("/latest/meta-data/instance-id", "test-instance-id"),
        ("/latest/meta-data/instance-type", "test-instance-type"),
        ("/latest/meta-data/local-ipv4", "10.0.0.10"),
        ("/latest/meta-data/hostname", "test-hostname"),
        ("/latest/meta-data/public-hostname", "test-public-hostname"),
        (
            "/latest/meta-data/placement/availability-zone",
            "eu-west-2a",
        ),
    ]);

    let expected = HashMap::from([
        ("OUTSCALE_AMI_ID".to_string(), "test-ami-id".to_string()),
        (
            "OUTSCALE_INSTANCE_ID".to_string(),
            "test-instance-id".to_string(),
        ),
        (
            "OUTSCALE_INSTANCE_TYPE".to_string(),
            "test-instance-type".to_string(),
        ),
        ("OUTSCALE_IPV4_LOCAL".to_string(), "10.0.0.10".to_string()),
        ("OUTSCALE_HOSTNAME".to_string(), "test-hostname".to_string()),
        (
            "OUTSCALE_PUBLIC_HOSTNAME".to_string(),
            "test-public-hostname".to_string(),
        ),
        (
            "OUTSCALE_AVAILABILITY_ZONE".to_string(),
            "eu-west-2a".to_string(),
        ),
    ]);

    let mut server = mockito::Server::new();

    for (endpoint, body) in endpoints {
        server
            .mock("GET", endpoint)
            .with_status(200)
            .with_body(body)
            .create();
    }

    let client = crate::retry::Client::try_new()
        .unwrap()
        .max_retries(0)
        .return_on_404(true)
        .mock_base_url(server.url());

    let provider = outscale::OutscaleProvider { client };

    assert_eq!(provider.attributes().unwrap(), expected);

    server.reset();
    provider.attributes().unwrap_err();
}
