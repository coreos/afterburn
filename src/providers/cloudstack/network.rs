//! network metadata fetcher for the cloudstack provider

use errors::*;
use metadata::Metadata;
use std::net::IpAddr;
use std::time::Duration;
use retry;
use openssh_keys::PublicKey;
use util;

const SERVER_ADDRESS: &'static str = "SERVER_ADDRESS";

pub fn fetch_metadata() -> Result<Metadata> {
    // first, find the address of the metadata service
    let server = get_dhcp_server_address()?;
    let client = retry::Client::new()?
        .initial_backoff(Duration::from_secs(1))
        .max_backoff(Duration::from_secs(5))
        .max_attempts(10);
    let endpoint_for = |key| format!("http://{}/latest/meta-data/{}", server, key);

    // then get the ssh keys and parse them
    let keys: Option<String> = client.get(retry::Raw, endpoint_for("public-keys")).send()
        .chain_err(|| "failed to get public keys")?;

    let mut builder = Metadata::builder();
    if let Some(k) = keys {
        let keys = PublicKey::read_keys(k.as_bytes())?;
        builder = builder.add_publickeys(keys);
    }

    Ok(builder
       .add_attribute_if_exists("CLOUDSTACK_INSTANCE_ID".into(), client.get(retry::Raw, endpoint_for("instance-id")).send()?)
       .add_attribute_if_exists("CLOUDSTACK_LOCAL_HOSTNAME".into(), client.get(retry::Raw, endpoint_for("local-hostname")).send()?)
       .add_attribute_if_exists("CLOUDSTACK_PUBLIC_HOSTNAME".into(), client.get(retry::Raw, endpoint_for("public-hostname")).send()?)
       .add_attribute_if_exists("CLOUDSTACK_AVAILABILITY_ZONE".into(), client.get(retry::Raw, endpoint_for("availability-zone")).send()?)
       .add_attribute_if_exists("CLOUDSTACK_IPV4_PUBLIC".into(), client.get(retry::Raw, endpoint_for("public-ipv4")).send()?)
       .add_attribute_if_exists("CLOUDSTACK_IPV4_LOCAL".into(), client.get(retry::Raw, endpoint_for("local-ipv4")).send()?)
       .add_attribute_if_exists("CLOUDSTACK_SERVICE_OFFERING".into(), client.get(retry::Raw, endpoint_for("service-offering")).send()?)
       .add_attribute_if_exists("CLOUDSTACK_CLOUD_IDENTIFIER".into(), client.get(retry::Raw, endpoint_for("cloud-identifier")).send()?)
       .add_attribute_if_exists("CLOUDSTACK_VM_ID".into(), client.get(retry::Raw, endpoint_for("vm-id")).send()?)
       .build())
}

fn get_dhcp_server_address() -> Result<IpAddr> {
    let server = util::dns_lease_key_lookup(SERVER_ADDRESS)?;
    let ip = server.parse()
        .chain_err(|| format!("failed to parse server ip address: {}", server))?;
    Ok(IpAddr::V4(ip))
}
