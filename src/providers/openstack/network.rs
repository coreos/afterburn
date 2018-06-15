//! openstack metadata fetcher

use std::collections::HashMap;

use openssh_keys::PublicKey;
use update_ssh_keys::AuthorizedKeyEntry;

use errors::*;
use metadata::Metadata;
use network;
use providers::MetadataProvider;
use retry;

const URL: &'static str = "http://169.254.169.254/latest/meta-data";

#[derive(Clone, Debug)]
pub struct OpenstackProvider {
    client: retry::Client,
}

impl OpenstackProvider {
    pub fn new() -> Result<OpenstackProvider> {
        let client = retry::Client::new()?;
        Ok(OpenstackProvider { client })
    }

    fn endpoint_for(key: &str) -> String {
        format!("{}/{}", URL, key)
    }

    fn fetch_keys(&self) -> Result<Vec<String>> {
        let keys_list: Option<String> = self.client
            .get(retry::Raw, OpenstackProvider::endpoint_for("public-keys"))
            .send()?;
        let mut keys = Vec::new();
        if let Some(keys_list) = keys_list {
            for l in keys_list.lines() {
                let tokens: Vec<&str> = l.split('=').collect();
                if tokens.len() != 2 {
                    return Err("error parsing keyID".into());
                }
                let key: String = self.client
                    .get(retry::Raw, OpenstackProvider::endpoint_for(&format!("public-keys/{}/openssh-key", tokens[0])))
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
            let value = self.client.get(retry::Raw, OpenstackProvider::endpoint_for(name)).send()?;
            if let Some(value) = value {
                map.insert(key.to_string(), value);
            }
            Ok(())
        };

        add_value(&mut out, "OPENSTACK_HOSTNAME", "hostname")?;
        add_value(&mut out, "OPENSTACK_INSTANCE_ID", "instance-id")?;
        add_value(&mut out, "OPENSTACK_IPV4_LOCAL", "local-ipv4")?;
        add_value(&mut out, "OPENSTACK_IPV4_PUBLIC", "public-ipv4")?;

        Ok(out)
    }

    fn hostname(&self) -> Result<Option<String>> {
        self.client.get(retry::Raw, OpenstackProvider::endpoint_for("hostname")).send()
    }

    fn ssh_keys(&self) -> Result<Vec<AuthorizedKeyEntry>> {
        let mut out = Vec::new();

        for key in &self.fetch_keys()? {
            let key = PublicKey::parse(&key)?;
            out.push(AuthorizedKeyEntry::Valid{key});
        }

        Ok(out)
    }

    fn networks(&self) -> Result<Vec<network::Interface>> {
        Ok(vec![])
    }

    fn network_devices(&self) -> Result<Vec<network::Device>> {
        Ok(vec![])
    }
}

pub fn fetch_metadata() -> Result<Metadata> {
    let provider = OpenstackProvider::new()
        .chain_err(|| "openstack: failed to create http client")?;

    let hostname: Option<String> = provider.client.get(retry::Raw, OpenstackProvider::endpoint_for("hostname")).send()?;

    Ok(Metadata::builder()
        .add_attribute_if_exists("OPENSTACK_INSTANCE_ID".to_owned(), provider.client.get(retry::Raw, OpenstackProvider::endpoint_for("instance-id")).send()?)
        .add_attribute_if_exists("OPENSTACK_IPV4_LOCAL".to_owned(), provider.client.get(retry::Raw, OpenstackProvider::endpoint_for("local-ipv4")).send()?)
        .add_attribute_if_exists("OPENSTACK_IPV4_PUBLIC".to_owned(), provider.client.get(retry::Raw, OpenstackProvider::endpoint_for("public-ipv4")).send()?)
        .add_attribute_if_exists("OPENSTACK_HOSTNAME".to_owned(), hostname.clone())
        .set_hostname_if_exists(hostname)
        .add_ssh_keys(provider.fetch_keys()?)?
        .build())
}

