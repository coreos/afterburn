use mockito;
use errors::*;
use providers::ec2;

pub(crate) const URL: &'static str = ::mockito::SERVER_URL;

#[test]
fn test_ec2_basic() {
    let ep = "/meta-data/public-keys";
    let client = ::retry::Client::new()
        .chain_err(|| "failed to create http client")
        .unwrap()
        .max_attempts(1)
        .return_on_404(true);

    ec2::fetch_ssh_keys(&client).unwrap_err();

    let _m = mockito::mock("GET", ep)
        .with_status(503)
        .create();
    ec2::fetch_ssh_keys(&client).unwrap_err();

    let _m = mockito::mock("GET", ep)
        .with_status(200)
        .create();
    let v = ec2::fetch_ssh_keys(&client).unwrap();
    assert_eq!(v.len(), 0);

    let _m = mockito::mock("GET", ep)
        .with_status(404)
        .create();
    let v = ec2::fetch_ssh_keys(&client).unwrap();
    assert_eq!(v.len(), 0);
}
