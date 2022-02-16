use crate::providers::aliyun;
use crate::providers::MetadataProvider;
use mockito;

#[test]
fn basic_hostname() {
    let ep = "/hostname";
    let hostname = "test-hostname";

    let mut provider = aliyun::AliyunProvider::try_new().unwrap();
    provider.client = provider.client.max_retries(0);

    {
        let _m503 = mockito::mock("GET", ep).with_status(503).create();
        provider.hostname().unwrap_err();
    }

    {
        let _m200 = mockito::mock("GET", ep)
            .with_status(200)
            .with_body(hostname)
            .create();
        let v = provider.hostname().unwrap();
        assert_eq!(v, Some(hostname.to_string()));
    }

    {
        let _m404 = mockito::mock("GET", ep).with_status(404).create();
        let v = provider.hostname().unwrap();
        assert_eq!(v, None);
    }

    {
        let _m200_empty = mockito::mock("GET", ep)
            .with_status(200)
            .with_body("")
            .create();
        let v = provider.hostname().unwrap();
        assert_eq!(v, None);
    }

    mockito::reset();
    provider.hostname().unwrap_err();
}

#[test]
fn basic_pubkeys() {
    let mut provider = aliyun::AliyunProvider::try_new().unwrap();
    provider.client = provider.client.max_retries(0);

    // Setup two entries with identical content, in order to test de-dup.
    let _m_keys = mockito::mock("GET", "/public-keys/")
        .with_status(200)
        .with_body("0/\ntest/\n")
        .create();
    let _m_key0 = mockito::mock("GET", "/public-keys/0/openssh-key")
        .with_status(200)
        .with_body("ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIIadOopfaOOAdFWRkCoOimvDyOftqphtnIeiECJuhkdq test-comment")
        .create();
    let _m_keytest = mockito::mock("GET", "/public-keys/test/openssh-key")
        .with_status(200)
        .with_body("ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIIadOopfaOOAdFWRkCoOimvDyOftqphtnIeiECJuhkdq test-comment")
        .create();

    let keys = provider.ssh_keys().unwrap();
    assert_ne!(keys, vec![]);
    assert_eq!(keys.len(), 1);

    assert_eq!(keys[0].options, None);
    assert_eq!(keys[0].comment, Some("test-comment".to_string()));

    mockito::reset();
    provider.ssh_keys().unwrap_err();
}

#[test]
fn basic_attributes() {
    let eipv4 = "test-eipv4";
    let hostname = "test-hostname";
    let image_id = "test-image-id";
    let instance_id = "test-instance-id";
    let instance_type = "test-instance-type";
    let private_ipv4 = "test-private-ipv4";
    let public_ipv4 = "test-public-ipv4";
    let region_id = "test-region-id";
    let vpc_id = "test-vpc-id";
    let zone_id = "test-zone-id";

    let endpoints = maplit::btreemap! {
        "/eipv4" => eipv4,
        "/hostname" => hostname,
        "/image-id" => image_id,
        "/instance-id" => instance_id,
        "/instance/instance-type" => instance_type,
        "/private-ipv4" => private_ipv4,
        "/public-ipv4" => public_ipv4,
        "/region-id" => region_id,
        "/vpc-id" => vpc_id,
        "/zone-id" => zone_id,
    };
    let mut mocks = Vec::with_capacity(endpoints.len());
    for (endpoint, body) in endpoints {
        let m = mockito::mock("GET", endpoint)
            .with_status(200)
            .with_body(body)
            .create();
        mocks.push(m);
    }

    let attributes = maplit::hashmap! {
        "ALIYUN_EIPV4".to_string() => eipv4.to_string(),
        "ALIYUN_HOSTNAME".to_string() => hostname.to_string(),
        "ALIYUN_IMAGE_ID".to_string() => image_id.to_string(),
        "ALIYUN_INSTANCE_ID".to_string() => instance_id.to_string(),
        "ALIYUN_INSTANCE_TYPE".to_string() => instance_type.to_string(),
        "ALIYUN_IPV4_PRIVATE".to_string() => private_ipv4.to_string(),
        "ALIYUN_IPV4_PUBLIC".to_string() => public_ipv4.to_string(),
        "ALIYUN_REGION_ID".to_string()=> region_id.to_string(),
        "ALIYUN_VPC_ID".to_string() => vpc_id.to_string(),
        "ALIYUN_ZONE_ID".to_string() => zone_id.to_string(),
    };

    let client = crate::retry::Client::try_new()
        .unwrap()
        .max_retries(0)
        .return_on_404(true);
    let provider = aliyun::AliyunProvider { client };

    let v = provider.attributes().unwrap();
    assert_eq!(v, attributes);

    mockito::reset();
    provider.attributes().unwrap_err();
}
