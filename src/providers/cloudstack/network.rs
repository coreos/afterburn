//! network metadata fetcher for the cloudstack provider

use std::collections::HashMap;
use std::net::IpAddr;

use anyhow::{Context, Result};
use openssh_keys::PublicKey;

use crate::providers::MetadataProvider;
use crate::retry;
use crate::util::DhcpOption;

#[derive(Clone, Debug)]
pub struct CloudstackNetwork {
    server_base_url: String,
    pub(crate) client: retry::Client,
}

impl CloudstackNetwork {
    pub fn try_new() -> Result<CloudstackNetwork> {
        let server_base_url = CloudstackNetwork::get_server_base_url_from_dhcp()?;
        let client = retry::Client::try_new()?.return_on_404(true);

        Ok(CloudstackNetwork {
            server_base_url,
            client,
        })
    }

    fn endpoint_for(&self, key: &str) -> String {
        format!("{}/latest/meta-data/{}", self.server_base_url, key)
    }

    fn get_server_base_url_from_dhcp() -> Result<String> {
        if cfg!(test) {
            // will be ignored by retry.Client
            return Ok("http://localhost".into());
        }
        let server = DhcpOption::DhcpServerId.get_value()?;
        let ip = server
            .parse::<IpAddr>()
            .with_context(|| format!("failed to parse server ip address: {server}"))?;
        Ok(format!("http://{ip}"))
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
