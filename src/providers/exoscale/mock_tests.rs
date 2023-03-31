use crate::providers::exoscale;
use crate::providers::MetadataProvider;
use mockito;

#[test]
fn basic_hostname() {
    let ep = "/1.0/meta-data/local-hostname";
    let hostname = "test-hostname";

    let mut server = mockito::Server::new();
    let mut provider = exoscale::ExoscaleProvider::try_new().unwrap();
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

    server
        .mock("GET", ep)
        .with_status(200)
        .with_body("")
        .create();
    let v = provider.hostname().unwrap();
    assert_eq!(v, None);

    server.reset();
    provider.hostname().unwrap_err();
}

#[test]
fn basic_pubkeys() {
    let mut server = mockito::Server::new();
    let mut provider = exoscale::ExoscaleProvider::try_new().unwrap();
    provider.client = provider.client.max_retries(0).mock_base_url(server.url());

    server.mock("GET", "/1.0/meta-data/public-keys")
        .with_status(200)
        .with_body("ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQC+bqdi18/+JfjrqmOEtVKyCU0bsIc6tBqqU7p9mesJkALocLddDU6d97w2zwERhzaqReDyg4msvQQohgtncb4afKKWQjCCCWlcwtP0nAeg9GFtUfmLeYcP2KAjxblabncluuAnvMHyBixKAjr5eWD4B1HjOmpMRmycwmy85QhGTYhF+AkiHGCPPUDrVy2cIvrPSDXEEa7bz5aQUime0Eold56n3O7E5BJuAozf+oeiWCERRRt9ATlLkMvwVItzBHN25YoMOd0KfgYMtBVAw86TErYFx4Tu98blYNUQTthf9VxcU8xy0rFacXmuS7LHbp+CKDY0X5dNHuhqz0wFto4J test-comment")
        .create();

    let keys = provider.ssh_keys().unwrap();
    assert_ne!(keys, vec![]);
    assert_eq!(keys.len(), 1);

    assert_eq!(keys[0].options, None);
    assert_eq!(keys[0].comment, Some("test-comment".to_string()));

    server.reset();
    provider.ssh_keys().unwrap_err();
}

#[test]
fn basic_attributes() {
    let public_hostname = "test-hostname";
    let local_hostname = "test-hostname";
    let instance_id = "test-instance-id";
    let service_offering = "test-instance-service-offering";
    let local_ipv4 = "test-local-ipv4";
    let public_ipv4 = "test-public-ipv4";
    let availability_zone = "availability-zone";
    let cloud_identifier = "test-cloud-identifier";
    let vm_id = "test-vm-id";

    let endpoints = maplit::btreemap! {
        "/1.0/meta-data/local-hostname" => local_hostname,
        "/1.0/meta-data/public-hostname" => public_hostname,
        "/1.0/meta-data/instance-id" => instance_id,
        "/1.0/meta-data/service-offering" => service_offering,
        "/1.0/meta-data/local-ipv4" => local_ipv4,
        "/1.0/meta-data/public-ipv4" => public_ipv4,
        "/1.0/meta-data/availability-zone" => availability_zone,
        "/1.0/meta-data/cloud-identifier" => cloud_identifier,
        "/1.0/meta-data/vm-id" => vm_id,
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
        "EXOSCALE_INSTANCE_ID".to_string() => instance_id.to_string(),
        "EXOSCALE_LOCAL_HOSTNAME".to_string() => local_hostname.to_string(),
        "EXOSCALE_PUBLIC_HOSTNAME".to_string() => public_hostname.to_string(),
        "EXOSCALE_AVAILABILITY_ZONE".to_string() => availability_zone.to_string(),
        "EXOSCALE_PUBLIC_IPV4".to_string() => public_ipv4.to_string(),
        "EXOSCALE_LOCAL_IPV4".to_string() => local_ipv4.to_string(),
        "EXOSCALE_SERVICE_OFFERING".to_string() => service_offering.to_string(),
        "EXOSCALE_CLOUD_IDENTIFIER".to_string()=> cloud_identifier.to_string(),
        "EXOSCALE_VM_ID".to_string() => vm_id.to_string(),
    };

    let client = crate::retry::Client::try_new()
        .unwrap()
        .max_retries(0)
        .return_on_404(true)
        .mock_base_url(server.url());
    let provider = exoscale::ExoscaleProvider { client };

    let v = provider.attributes().unwrap();
    assert_eq!(v, attributes);

    server.reset();
    provider.attributes().unwrap_err();
}
