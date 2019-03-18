use errors::*;
use mockito;
use providers::ec2;

#[test]
fn test_ec2_basic() {
    let ep = "/meta-data/public-keys";
    let client = ::retry::Client::try_new()
        .chain_err(|| "failed to create http client")
        .unwrap()
        .max_attempts(1)
        .return_on_404(true);
    let provider = ec2::Ec2Provider { client };

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
