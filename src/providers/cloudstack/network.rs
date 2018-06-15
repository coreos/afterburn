//! network metadata fetcher for the cloudstack provider

use std::collections::HashMap;
use std::net::IpAddr;
use std::time::Duration;

use openssh_keys::PublicKey;
use update_ssh_keys::AuthorizedKeyEntry;

use errors::*;
use metadata::Metadata;
use network;
use providers::MetadataProvider;
use retry;
use util;

const SERVER_ADDRESS: &'static str = "SERVER_ADDRESS";

#[derive(Clone, Debug)]
pub struct CloudstackNetwork {
    server_address: IpAddr,
    client: retry::Client,
}

impl CloudstackNetwork {
    pub fn new() -> Result<CloudstackNetwork> {
        let server_address = CloudstackNetwork::get_dhcp_server_address()?;
        let client = retry::Client::new()?
            .initial_backoff(Duration::from_secs(1))
            .max_backoff(Duration::from_secs(5))
            .max_attempts(10);

        Ok(CloudstackNetwork {
            server_address,
            client,
        })
    }

    fn endpoint_for(&self, key: &str) -> String {
        format!("http://{}/latest/meta-data/{}", self.server_address, key)
    }

    fn get_dhcp_server_address() -> Result<IpAddr> {
        let server = util::dns_lease_key_lookup(SERVER_ADDRESS)?;
        let ip = server.parse()
            .chain_err(|| format!("failed to parse server ip address: {}", server))?;
        Ok(IpAddr::V4(ip))
    }
}

impl MetadataProvider for CloudstackNetwork {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        let mut out = HashMap::with_capacity(9);
        let add_value = |map: &mut HashMap<_, _>, key: &str, name| -> Result<()> {
            let value = self.client.get(retry::Raw, self.endpoint_for(name)).send()?;

            if let Some(value) = value {
                map.insert(key.to_string(), value);
            }

            Ok(())
        };

        add_value(&mut out, "CLOUDSTACK_INSTANCE_ID", "instance-id")?;
        add_value(&mut out, "CLOUDSTACK_LOCAL_HOSTNAME", "local-hostname")?;
        add_value(&mut out, "CLOUDSTACK_PUBLIC_HOSTNAME", "public-hostname")?;
        add_value(&mut out, "CLOUDSTACK_AVAILABILITY_ZONE", "availability-zone")?;
        add_value(&mut out, "CLOUDSTACK_IPV4_PUBLIC", "public-ipv4")?;
        add_value(&mut out, "CLOUDSTACK_IPV4_LOCAL", "local-ipv4")?;
        add_value(&mut out, "CLOUDSTACK_SERVICE_OFFERING", "service-offering")?;
        add_value(&mut out, "CLOUDSTACK_CLOUD_IDENTIFIER", "cloud-identifier")?;
        add_value(&mut out, "CLOUDSTACK_VM_ID", "vm-id")?;

        Ok(out)
    }

    fn hostname(&self) -> Result<Option<String>> {
        Ok(None)
    }

    fn ssh_keys(&self) -> Result<Vec<AuthorizedKeyEntry>> {
        let keys: Option<String> = self.client
            .get(retry::Raw, self.endpoint_for("public-keys"))
            .send()?;

        if let Some(keys) = keys {
            let keys = PublicKey::read_keys(keys.as_bytes())?
                .into_iter()
                .map(|key| AuthorizedKeyEntry::Valid{key})
                .collect::<Vec<_>>();

            Ok(keys)
        } else {
            Ok(vec![])
        }
    }

    fn networks(&self) -> Result<Vec<network::Interface>> {
        Ok(vec![])
    }

    fn network_devices(&self) -> Result<Vec<network::Device>> {
        Ok(vec![])
    }
}

pub fn fetch_metadata() -> Result<Metadata> {
    // first, find the address of the metadata service
    let server = CloudstackNetwork::get_dhcp_server_address()?;
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
