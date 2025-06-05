use crate::providers::oraclecloud;
use crate::providers::MetadataProvider;
use crate::retry;
use mockito;

const INSTANCE_METADATA_ENDPOINT: &str = "/opc/v2/instance";

#[test]
fn test_hostname() {
    let metadata = r#"{
    "availabilityDomain": "",
    "canonicalRegionName": "",
    "compartmentId": "",
    "faultDomain": "",
    "id": "",
    "hostname": "example-1",
    "shape": ""
}"#;

    let mut server = mockito::Server::new();
    let client = retry::Client::try_new()
        .unwrap()
        .max_retries(0)
        .mock_base_url(server.url());

    server
        .mock("GET", INSTANCE_METADATA_ENDPOINT)
        .match_header("Authorization", "Bearer Oracle")
        .with_status(200)
        .with_body(metadata)
        .create();
    let provider = oraclecloud::OracleCloudProvider::try_new_with_client(&client).unwrap();
    let v = provider.hostname().unwrap();
    assert_eq!(v, Some("example-1".to_string()));
}

#[test]
fn test_pubkeys() {
    let metadata = r#"{
    "availabilityDomain": "",
    "canonicalRegionName": "",
    "compartmentId": "",
    "faultDomain": "",
    "id": "",
    "hostname": "",
    "shape": "",
    "metadata": {
        "ssh_authorized_keys": "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQCsXe6CfHl45kCIzMF92VhDf2NpBWUyS1+IiTtxm5a83mT9730Hb8xim7GYeJu47kiESw2DAN8vNJ/Irg0apZ217ah2rXXjPQuWYSXuEuap8yLBSjqw8exgqVj/kzW+YqmnHASxI13eoFDxTQQGzyqbqowvxu/5gQmDwBmNAa9bT809ziB/qmpS1mD6qyyFDpR23kUwu3TkgAbwMXBDoqK+pdwfaF9uo9XaLHNEH8lD5BZuG2BeDafm2o76DhNSo83MvcCPNXKLxu3BbX/FCMFO6O8RRqony4i91fEV1b8TbXrbJz1bwEYEnJRvmjnqI/389tQFeYvplXR2WdT9PCKyEAG+j8y6XgecIcdTqV/7gFfak1mp2S7mYHZDnXixsn3MjCP/cIxxJVDitKusnj1TdFqtSXl4tqGccbg/5Sqnt/EVSK4bGwwBxv/YmE0P9cbXLxuEVI0JYzgrQvC8TtUgd8kUu2jqi1/Yj9IWm3aFsl/hhh8YwYrv/gm8PV0TxkM= root@example1\nssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDj6FBVgkTt7/DB93VVLk6304Nx7WUjLBJDSCh38zjCimHUpeo9uYDxflfu2N1CLtrSImIKBVP/JRy9g7K4zmRAH/wXw2UxYziX+hZoFIpbW3GmYQqhjx2lDvIRXJI7blhHhTUNWX5f10lFAYOLqA9J859AB1w7ND09+MS3jQgSazCx17h+QZ0qQ6kLSfnXw9PMUOE1Xba9hD1nYj14ryTVj9jrFPMFuUfXdb/G9lsDJ+cGvdE2/RMuPfDmEdo04zvZ5fQJJKvS7OyAuYev4Y+JC8MhEr756ITDZ17yq4BEMo/8rNPxZ5Von/8xnvry+8/2C3ep9rZyHtCwpRb6WT6TndV2ddXKhEIneyd1XiOcWPJguHj5vSoMN3mo8k2PvznGauvxBstvpjUSFLQu869/ZQwyMnbQi3wnkJk5CpLXePXn1J9njocJjt8+SKGijmmIAsmYosx8gmmu3H1mvq9Wi0qqWDITMm+J24AZBEPBhwVrjhLZb5MKxylF6JFJJBs= root@example2"
    }
}"#;

    let metadata_no_key = r#"{
    "availabilityDomain": "",
    "canonicalRegionName": "",
    "compartmentId": "",
    "faultDomain": "",
    "id": "",
    "hostname": "",
    "shape": ""
}"#;

    let mut server = mockito::Server::new();
    let client = retry::Client::try_new()
        .unwrap()
        .max_retries(0)
        .mock_base_url(server.url());

    server
        .mock("GET", INSTANCE_METADATA_ENDPOINT)
        .match_header("Authorization", "Bearer Oracle")
        .with_status(200)
        .with_body(metadata)
        .create();

    let provider = oraclecloud::OracleCloudProvider::try_new_with_client(&client).unwrap();
    let keys = provider.ssh_keys().unwrap();
    assert_ne!(keys, vec![]);
    assert_eq!(keys.len(), 2);

    assert_eq!(keys[0].options, None);
    assert_eq!(keys[0].comment, Some("root@example1".to_string()));

    assert_eq!(keys[1].options, None);
    assert_eq!(keys[1].comment, Some("root@example2".to_string()));

    server.reset();
    server
        .mock("GET", INSTANCE_METADATA_ENDPOINT)
        .match_header("Authorization", "Bearer Oracle")
        .with_status(200)
        .with_body(metadata_no_key)
        .create();
    let provider = oraclecloud::OracleCloudProvider::try_new_with_client(&client).unwrap();
    let keys = provider.ssh_keys().unwrap();
    assert_eq!(keys, vec![]);
}

#[test]
fn test_attributes() {
    let metadata = r#"{
  "availabilityDomain" : "EMIr:PHX-AD-1",
  "faultDomain" : "FAULT-DOMAIN-3",
  "compartmentId" : "ocid1.tenancy.oc1..exampleuniqueID",
  "displayName" : "my-example-instance",
  "hostname" : "my-hostname",
  "id" : "ocid1.instance.oc1.phx.exampleuniqueID",
  "image" : "ocid1.image.oc1.phx.exampleuniqueID",
  "metadata" : {
    "ssh_authorized_keys" : "example-ssh-key"
  },
  "region" : "phx",
  "canonicalRegionName" : "us-phoenix-1",
  "ociAdName" : "phx-ad-1",
  "regionInfo" : {
    "realmKey" : "oc1",
    "realmDomainComponent" : "oraclecloud.com",
    "regionKey" : "PHX",
    "regionIdentifier" : "us-phoenix-1"
  },
  "shape" : "VM.Standard.E3.Flex",
  "state" : "Running",
  "timeCreated" : 1600381928581,
  "agentConfig" : {
    "monitoringDisabled" : false,
    "managementDisabled" : false,
    "allPluginsDisabled" : false,
    "pluginsConfig" : [ {
      "name" : "OS Management Service Agent",
      "desiredState" : "ENABLED"
    }, {
      "name" : "Custom Logs Monitoring",
      "desiredState" : "ENABLED"
    }, {
      "name" : "Compute Instance Run Command",
      "desiredState" : "ENABLED"
    }, {
      "name" : "Compute Instance Monitoring",
      "desiredState" : "ENABLED"
    } ]
  },
  "freeformTags": {
    "Department": "Finance"
  },
  "definedTags": {
    "Operations": {
      "CostCenter": "42"
    }
  }
}"#;

    let mut server = mockito::Server::new();
    let client = retry::Client::try_new()
        .unwrap()
        .max_retries(0)
        .mock_base_url(server.url());

    server
        .mock("GET", INSTANCE_METADATA_ENDPOINT)
        .match_header("Authorization", "Bearer Oracle")
        .with_status(200)
        .with_body(metadata)
        .create();

    let attributes = maplit::hashmap! {
        "ORACLECLOUD_AVAILABILITY_DOMAIN".to_string() => "EMIr:PHX-AD-1".to_string(),
        "ORACLECLOUD_COMPARTMENT_ID".to_string() => "ocid1.tenancy.oc1..exampleuniqueID".to_string(),
        "ORACLECLOUD_FAULT_DOMAIN".to_string() => "FAULT-DOMAIN-3".to_string(),
        "ORACLECLOUD_HOSTNAME".to_string() => "my-hostname".to_string(),
        "ORACLECLOUD_INSTANCE_ID".to_string() => "ocid1.instance.oc1.phx.exampleuniqueID".to_string(),
        "ORACLECLOUD_INSTANCE_SHAPE".to_string() => "VM.Standard.E3.Flex".to_string(),
        "ORACLECLOUD_REGION_ID".to_string() => "us-phoenix-1".to_string(),
    };

    let provider = oraclecloud::OracleCloudProvider::try_new_with_client(&client).unwrap();
    let v = provider.attributes().unwrap();
    assert_eq!(v, attributes);
}
