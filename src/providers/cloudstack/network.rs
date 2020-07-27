//! network metadata fetcher for the cloudstack provider

use std::collections::HashMap;
use std::net::IpAddr;

use openssh_keys::PublicKey;

use crate::errors::*;
use crate::providers::MetadataProvider;
use crate::retry;
use crate::util;

const SERVER_ADDRESS: &str = "SERVER_ADDRESS";

#[derive(Clone, Debug)]
pub struct CloudstackNetwork {
    server_address: IpAddr,
    client: retry::Client,
}

impl CloudstackNetwork {
    pub fn try_new() -> Result<CloudstackNetwork> {
        let server_address = CloudstackNetwork::get_dhcp_server_address()?;
        let client = retry::Client::try_new()?;

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
        let ip = server
            .parse()
            .chain_err(|| format!("failed to parse server ip address: {}", server))?;
        Ok(IpAddr::V4(ip))
    }
}

impl MetadataProvider for CloudstackNetwork {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        let mut out = HashMap::with_capacity(9);
        let add_value = |map: &mut HashMap<_, _>, key: &str, name| -> Result<()> {
            let value = self
                .client
                .get(retry::Raw, self.endpoint_for(name))
                .send()?;

            if let Some(value) = value {
                map.insert(key.to_string(), value);
            }

            Ok(())
        };

        add_value(&mut out, "CLOUDSTACK_INSTANCE_ID", "instance-id")?;
        add_value(&mut out, "CLOUDSTACK_LOCAL_HOSTNAME", "local-hostname")?;
        add_value(&mut out, "CLOUDSTACK_PUBLIC_HOSTNAME", "public-hostname")?;
        add_value(
            &mut out,
            "CLOUDSTACK_AVAILABILITY_ZONE",
            "availability-zone",
        )?;
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

    fn ssh_keys(&self) -> Result<Vec<PublicKey>> {
        let keys: Option<String> = self
            .client
            .get(retry::Raw, self.endpoint_for("public-keys"))
            .send()?;

        if let Some(keys) = keys {
            let keys = PublicKey::read_keys(keys.as_bytes())?;
            Ok(keys)
        } else {
            Ok(vec![])
        }
    }
}
