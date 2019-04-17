use crate::errors::*;
use mockito;
use crate::providers::aws;

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
