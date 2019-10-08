use crate::providers::aliyun;
use crate::providers::MetadataProvider;
use mockito;

#[test]
fn basic_hostname() {
    let ep = "/hostname";
    let hostname = "test-hostname";

    let mut provider = aliyun::AliyunProvider::try_new().unwrap();
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

    let _m = mockito::mock("GET", ep)
        .with_status(200)
        .with_body("")
        .create();
    let v = provider.hostname().unwrap();
    assert_eq!(v, None);
}

#[test]
fn basic_pubkeys() {
    let mut provider = aliyun::AliyunProvider::try_new().unwrap();
    provider.client = provider.client.max_attempts(1);

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
}
