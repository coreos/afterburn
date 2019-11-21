//! openstack metadata fetcher

use std::collections::HashMap;

use openssh_keys::PublicKey;
use slog_scope::warn;

use crate::errors::*;
use crate::network;
use crate::providers::MetadataProvider;
use crate::retry;

const URL: &str = "http://169.254.169.254/latest/meta-data";

#[derive(Clone, Debug)]
pub struct OpenstackProvider {
    client: retry::Client,
}

impl OpenstackProvider {
    pub fn try_new() -> Result<OpenstackProvider> {
        let client = retry::Client::try_new()?;
        Ok(OpenstackProvider { client })
    }

    fn endpoint_for(key: &str) -> String {
        format!("{}/{}", URL, key)
    }

    fn fetch_keys(&self) -> Result<Vec<String>> {
        let keys_list: Option<String> = self
            .client
            .get(retry::Raw, OpenstackProvider::endpoint_for("public-keys"))
            .send()?;
        let mut keys = Vec::new();
        if let Some(keys_list) = keys_list {
            for l in keys_list.lines() {
                let tokens: Vec<&str> = l.split('=').collect();
                if tokens.len() != 2 {
                    return Err("error parsing keyID".into());
                }
                let key: String = self
                    .client
                    .get(
                        retry::Raw,
                        OpenstackProvider::endpoint_for(&format!(
                            "public-keys/{}/openssh-key",
                            tokens[0]
                        )),
                    )
                    .send()?
                    .ok_or("missing ssh key")?;
                keys.push(key);
            }
        }
        Ok(keys)
    }
}

impl MetadataProvider for OpenstackProvider {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        let mut out = HashMap::with_capacity(4);

        let add_value = |map: &mut HashMap<_, _>, key: &str, name| -> Result<()> {
            let value = self
                .client
                .get(retry::Raw, OpenstackProvider::endpoint_for(name))
                .send()?;
            if let Some(value) = value {
                map.insert(key.to_string(), value);
            }
            Ok(())
        };

        add_value(&mut out, "OPENSTACK_HOSTNAME", "hostname")?;
        add_value(&mut out, "OPENSTACK_INSTANCE_ID", "instance-id")?;
        add_value(&mut out, "OPENSTACK_INSTANCE_TYPE", "instance-type")?;
        add_value(&mut out, "OPENSTACK_IPV4_LOCAL", "local-ipv4")?;
        add_value(&mut out, "OPENSTACK_IPV4_PUBLIC", "public-ipv4")?;

        Ok(out)
    }

    fn hostname(&self) -> Result<Option<String>> {
        self.client
            .get(retry::Raw, OpenstackProvider::endpoint_for("hostname"))
            .send()
    }

    fn ssh_keys(&self) -> Result<Vec<PublicKey>> {
        let mut out = Vec::new();

        for key in &self.fetch_keys()? {
            let key = PublicKey::parse(&key)?;
            out.push(key);
        }

        Ok(out)
    }

    fn networks(&self) -> Result<Vec<network::Interface>> {
        Ok(vec![])
    }

    fn virtual_network_devices(&self) -> Result<Vec<network::VirtualNetDev>> {
        warn!("virtual network devices metadata requested, but not supported on this platform");
        Ok(vec![])
    }

    fn boot_checkin(&self) -> Result<()> {
        warn!("boot check-in requested, but not supported on this platform");
        Ok(())
    }
}
